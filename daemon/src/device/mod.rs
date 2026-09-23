//! The board, as the rest of the daemon sees it.
//!
//! Everything above this module talks to a device through [`Device`]: read the
//! counts, arm a compiled set, move the origin, watch the wire. What is under
//! it is a **serial port** — `docs/reference/protocol.md` over a file
//! descriptor — and on a host with no board, a pty with [`board_simulator`] on
//! the far end. The daemon runs the same code either way; only the descriptor
//! differs.
//!
//! ## What this module owns, and what the board owns
//!
//! The board owns what must be decided in the scan that sees the count: its own
//! origin, its own odometer, and **zone evaluation**. The host mirrors those
//! numbers from the samples it receives, because the API reports them — and a
//! mirror computed from the same stream is right whenever the stream is
//! complete. When it is not, `seq_gaps` says so, rather than a distance quietly
//! coming up short.
//!
//! The host owns two things the board cannot: the mapping from the device clock
//! to `CLOCK_MONOTONIC`, and the **continuity offset** that keeps the published
//! accumulator from stepping backwards across a board reset.
//!
//! ## Threads
//!
//! One reader thread per link, blocking on `read`. It parses, updates state
//! under a mutex and broadcasts; it never waits on an HTTP handler. Commands go
//! out through a second descriptor, from whichever thread asked.

pub mod board_simulator;

use std::collections::VecDeque;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use tokio::sync::broadcast;

use crate::link::continuity::{ClockCorrelation, Continuity};
use crate::link::framing::check;
use crate::link::messages::{
    commands, decode, describe_device, Action, Command, Fire, FromDevice, Interval, Metric,
    Sample as WireSample, Zone as WireZone,
};
use crate::link::serial::{duplicate, open_port, pty_pair, FrameReader, FrameWriter};
use crate::model::device::{
    Capacities, DeviceInfo, FlashedZoneSet, LinkStats, WireDirection, WireLevel, WireLine,
};
use crate::model::state::{
    AxisState, LinkHealth, LinkState, RigState, Sample, StreamFrame, ZoneHitEvent,
};
use crate::model::zone_set::{ArmOrigin, FireRule, ZoneMetric, ZoneStatus};
use crate::publish::SegmentPublisher;
use crate::zones::{CompiledZone, CompiledZoneSet};

/// The conversation — greetings, uploads, arms, refusals — and nothing else.
/// Hours of it on a rig, because these are the lines somebody reads when the
/// layers stop agreeing.
const CONVERSATION_LINES: usize = 2000;
/// Samples are kept separately and briefly. **Not tidiness:** at 500 Hz they
/// fill any shared ring in seconds, and the greeting and the arm a person is
/// actually looking for are gone before they open the panel. The monitor hides
/// them by default and shows the recent ones on request; this is what "recent"
/// can honestly mean on a stream that fast.
const SAMPLE_LINES: usize = 200;
const STALE_AFTER: Duration = Duration::from_millis(500);
/// How long an arm waits for the board to say `armed`. Long enough for a link
/// that is merely busy; short enough that an HTTP handler does not hang on a
/// board that has gone away.
const ARM_ACK_TIMEOUT: Duration = Duration::from_millis(750);
/// The span the host's own velocity is measured over. Not the gap between two
/// samples: a wheel publishes whole counts, so a slope across one 2 ms step is
/// mostly quantisation.
const VELOCITY_SPAN_NS: u64 = 250_000_000;
/// What this daemon sends in `hello`, and the range it will talk to.
///
/// **A floor, not an equality.** Firmware is flashed separately and will be
/// older than the daemon that talks to it; refusing anything but an exact match
/// would make every daemon release a reflash. Below the floor is refused by
/// name instead of half-understood, because a protocol this daemon only mostly
/// speaks is a zone armed at a distance nobody chose.
const PROTOCOL_VERSION: u32 = 1;
/// The frame limit before a board has said its own, in encoded bytes including
/// the delimiter. The ESP32 firmware's, and room for any message this protocol
/// has.
const DEFAULT_MAX_FRAME: usize = 256;
/// How long a zone's pulse is. The authored set has no width yet, so it is the
/// width statemachined's lines use.
const ZONE_PULSE_MS: u32 = 10;
const OLDEST_PROTOCOL_SPOKEN: u32 = 1;
/// The rule this arrangement exists for, as a compile error rather than a test:
/// firmware is flashed separately and will be older than the daemon, so a floor
/// above what this daemon speaks would make every release a reflash.
const _: () = assert!(OLDEST_PROTOCOL_SPOKEN <= PROTOCOL_VERSION);

/// One axis as the rig config describes it.
#[derive(Clone)]
pub struct AxisSetup {
    pub name: String,
    pub counts_per_cm: f64,
    pub invert: bool,
}

/// What the daemon is attached to.
pub enum Backend {
    /// No board. Every route that needs one refuses with `no_device` rather
    /// than inventing a reading.
    Absent,
    /// A real serial port, from the rig config.
    Port { path: String, baud: u32 },
    /// A pty with a simulated board on the far end — the same link code, and a
    /// board that can be wrong in the ways a real one is.
    Simulated,
}

struct AxisRuntime {
    name: String,
    counts_per_cm: f64,
    invert: bool,
    /// The published accumulator: the device's counts plus the continuity
    /// offset. The number a corridor is driven from.
    counts: i64,
    /// The host's mirror of the board's origin.
    origin: i64,
    distance: i64,
    host_velocity_cm_s: f64,
    device_velocity_cm_s: f64,
    /// Added after the sign, so that turning `invert` over while the daemon
    /// runs reverses the direction from then on without moving anything: see
    /// [`Device::set_invert`].
    bias: i64,
}

impl AxisRuntime {
    /// Counts in the board's frame — offset for continuity, not inverted — in
    /// the host's.
    fn host(&self, board: i64) -> i64 {
        self.sign() * board + self.bias
    }

    fn sign(&self) -> i64 {
        if self.invert {
            -1
        } else {
            1
        }
    }
}

struct ArmedSet {
    set: CompiledZoneSet,
    arm_id: u32,
    label: Option<String>,
    /// Per zone, in the set's own order: where it fired, if it has.
    fired_at: Vec<Option<i64>>,
    /// The board said `armed`.
    confirmed: bool,
}

struct Inner {
    axes: Vec<AxisRuntime>,
    connected: bool,
    board: String,
    firmware: String,
    protocol_version: u32,
    max_frame: usize,
    capacities: Capacities,
    flashed: Option<FlashedZoneSet>,
    connection_count: u64,
    last_error: Option<String>,
    message_id: u16,

    seq: u64,
    seq_gaps: u64,
    ring_drops: u64,
    last_sample_at: Option<Instant>,
    last_device_us: u64,
    measured_rate_hz: f64,
    history: VecDeque<(u64, i64)>,

    clock: ClockCorrelation,
    continuity: Continuity,
    armed: Option<ArmedSet>,
    /// The board's running count of late scans, as last reported. The board
    /// sends the total; what is news is the difference.
    scan_overruns: u64,
    /// Between a `hello_ack` and the `state_report` that finishes the connect.
    taking_over: bool,
    /// An arm the board had when this daemon connected and did not make: a
    /// flashed set, or an earlier daemon's. Kept so that it can be disarmed.
    foreign_arm_id: Option<u32>,
}

/// One output line, as the wire carries it: an index, a pin and a safe level.
/// Names stay host-side — the board holds no strings but the zone set's name.
pub type WireLine2 = (u8, u8, bool);

pub struct Device {
    backend: Backend,
    /// The rig's output line map, uploaded at every connect. The board comes up
    /// with its outputs at compile-time defaults and has no memory of a wiring
    /// it was told about before it was unplugged.
    lines: Vec<WireLine2>,
    inner: Mutex<Inner>,
    /// Signalled when the board acknowledges an arm.
    armed_ack: Condvar,
    writer: Mutex<Option<FrameWriter>>,
    conversation: Mutex<VecDeque<WireLine>>,
    samples: Mutex<VecDeque<WireLine>>,
    frames: broadcast::Sender<StreamFrame>,
    wire: broadcast::Sender<WireLine>,
    running: Arc<AtomicBool>,
    stream_rate_hz: u32,
    /// The segment vstimd reads every frame. `None` when it could not be
    /// created, which is a warning and not a reason to stop.
    publisher: Option<SegmentPublisher>,
}

/// Which set is armed, and under what.
pub struct ArmSummary {
    pub zone_set: String,
    pub version: u32,
    pub arm_id: u32,
    pub label: Option<String>,
}

impl Device {
    pub fn new(
        backend: Backend,
        axes: Vec<AxisSetup>,
        lines: Vec<WireLine2>,
        stream_rate_hz: u32,
        publisher: Option<SegmentPublisher>,
    ) -> Arc<Self> {
        let count = axes.len();
        let device = Arc::new(Self {
            backend,
            lines,
            inner: Mutex::new(Inner {
                axes: axes
                    .into_iter()
                    .map(|axis| AxisRuntime {
                        name: axis.name,
                        counts_per_cm: axis.counts_per_cm,
                        invert: axis.invert,
                        counts: 0,
                        origin: 0,
                        distance: 0,
                        host_velocity_cm_s: 0.0,
                        device_velocity_cm_s: 0.0,
                        bias: 0,
                    })
                    .collect(),
                connected: false,
                board: String::new(),
                firmware: String::new(),
                protocol_version: 0,
                max_frame: DEFAULT_MAX_FRAME,
                capacities: Capacities { n_axes: 0, max_zones: 0, max_lines: 0, scan_hz: 0 },
                flashed: None,
                connection_count: 0,
                last_error: None,
                message_id: 0,
                seq: 0,
                seq_gaps: 0,
                ring_drops: 0,
                last_sample_at: None,
                last_device_us: 0,
                measured_rate_hz: 0.0,
                history: VecDeque::new(),
                clock: ClockCorrelation::new(),
                continuity: Continuity::new(count),
                armed: None,
                scan_overruns: 0,
                taking_over: false,
                foreign_arm_id: None,
            }),
            armed_ack: Condvar::new(),
            writer: Mutex::new(None),
            conversation: Mutex::new(VecDeque::new()),
            samples: Mutex::new(VecDeque::new()),
            frames: broadcast::channel(4096).0,
            wire: broadcast::channel(1024).0,
            running: Arc::new(AtomicBool::new(true)),
            stream_rate_hz,
            publisher,
        });
        if let Err(problem) = device.clone().open_link() {
            log::warn!("the device link did not open: {problem}");
        }
        device
    }

    /// Open the port — or make a pty and put a simulated board on it — and
    /// start the reader.
    fn open_link(self: Arc<Self>) -> io::Result<()> {
        let path = match &self.backend {
            Backend::Absent => return Ok(()),
            Backend::Port { path, baud } => {
                let fd = open_port(path, *baud)?;
                *self.writer.lock().unwrap() = Some(FrameWriter::new(duplicate(&fd)?));
                self.spawn_reader(FrameReader::new(fd, DEFAULT_MAX_FRAME));
                path.clone()
            }
            Backend::Simulated => {
                let counts_per_cm = self
                    .inner
                    .lock()
                    .unwrap()
                    .axes
                    .first()
                    .map(|axis| axis.counts_per_cm)
                    .unwrap_or(86.92);
                let (board_side, host_path) = pty_pair()?;
                board_simulator::BoardSimulator::spawn(board_side, counts_per_cm)?;
                let fd = open_port(host_path.to_str().unwrap_or_default(), 0)?;
                *self.writer.lock().unwrap() = Some(FrameWriter::new(duplicate(&fd)?));
                self.spawn_reader(FrameReader::new(fd, DEFAULT_MAX_FRAME));
                host_path.to_string_lossy().into_owned()
            }
        };
        log::info!("device link open on {path}");
        self.spawn_housekeeping();
        // The greeting starts the conversation; everything else follows the
        // `hello_ack` that answers it.
        self.send(commands::hello(self.next_message_id(), PROTOCOL_VERSION));
        Ok(())
    }

    /// Every few seconds: a `state` for the counters only the board knows, and
    /// a `ping` when no sample has arrived to prove the link on its own.
    fn spawn_housekeeping(self: &Arc<Self>) {
        let device = self.clone();
        std::thread::Builder::new()
            .name("device-housekeeping".into())
            .spawn(move || {
                while device.running.load(Ordering::Relaxed) {
                    std::thread::sleep(Duration::from_secs(5));
                    if !device.connected() {
                        continue;
                    }
                    device.send(commands::state(device.next_message_id()));
                    let quiet = device
                        .inner
                        .lock()
                        .unwrap()
                        .last_sample_at
                        .map(|at| at.elapsed() > Duration::from_secs(2))
                        .unwrap_or(true);
                    if quiet {
                        device.send(commands::ping(device.next_message_id()));
                    }
                }
            })
            .expect("housekeeping needs a thread");
    }

    fn spawn_reader(self: &Arc<Self>, mut reader: FrameReader) {
        let device = self.clone();
        std::thread::Builder::new()
            .name("device-link".into())
            .spawn(move || {
                while device.running.load(Ordering::Relaxed) {
                    let frames = match reader.read_frames() {
                        Ok(frames) if frames.is_empty() => break,
                        Ok(frames) => frames,
                        Err(problem) => {
                            log::warn!("device link read failed: {problem}");
                            break;
                        }
                    };
                    for frame in frames {
                        device.receive(&frame);
                    }
                }
                log::warn!("the device link closed");
                device.inner.lock().unwrap().connected = false;
            })
            .expect("the device link needs a thread");
    }

    // ------------------------------------------------------- receiving ---

    fn receive(&self, frame: &[u8]) {
        let max_frame = self.inner.lock().unwrap().max_frame;
        let bytes = match check(frame, max_frame) {
            Ok(bytes) => bytes,
            Err(problem) => {
                // Never acted on, not even partially.
                self.note_error(&format!("{problem}"));
                self.log_wire(WireDirection::In, hex(frame), WireLevel::Error);
                return;
            }
        };
        let (message_id, message) = match decode(&bytes) {
            Ok(decoded) => decoded,
            Err(problem) => {
                self.note_error(&format!("{problem}"));
                self.log_wire(WireDirection::In, hex(&bytes), WireLevel::Error);
                return;
            }
        };

        let level = match &message {
            FromDevice::Error(_) => WireLevel::Error,
            _ => WireLevel::Info,
        };
        self.log_wire(WireDirection::In, describe_device(message_id, &message), level);

        match message {
            FromDevice::HelloAck(ack) => {
                if !speaks_protocol(ack.protocol_version) {
                    // Said once, plainly, and then nothing else is done with
                    // this board: no stream, no line map, no zones. A daemon
                    // that carried on here would be guessing.
                    self.note_error(&format!(
                        "the board speaks protocol {} and this daemon needs at least {} — \
                         reflash the board, or run the daemon that matches it",
                        ack.protocol_version, OLDEST_PROTOCOL_SPOKEN
                    ));
                    let mut inner = self.inner.lock().unwrap();
                    inner.connected = false;
                    inner.board = ack.board;
                    inner.firmware = ack.firmware;
                    inner.protocol_version = ack.protocol_version;
                    return;
                }
                {
                    let mut inner = self.inner.lock().unwrap();
                    // A greeting nobody asked for is a board that came back:
                    // its counters restarted, and the published accumulator
                    // must not.
                    if inner.connected {
                        inner.continuity.device_restarted();
                    }
                    inner.connected = true;
                    inner.connection_count += 1;
                    inner.board = ack.board;
                    inner.firmware = ack.firmware;
                    inner.protocol_version = ack.protocol_version;
                    if ack.max_frame != 0 {
                        inner.max_frame = ack.max_frame as usize;
                    }
                    inner.capacities = Capacities {
                        n_axes: u8::try_from(ack.n_axes).unwrap_or(u8::MAX),
                        max_zones: u16::try_from(ack.max_zones).unwrap_or(u16::MAX),
                        max_lines: u8::try_from(ack.max_lines).unwrap_or(u8::MAX),
                        scan_hz: ack.scan_hz,
                    };
                    inner.flashed = ack
                        .flashed
                        .map(|set| FlashedZoneSet { name: set.name, version: set.version });
                    inner.taking_over = true;
                }
                // Before configuring, ask what the board is doing: one left armed
                // by a daemon that has since gone refuses the configuration as
                // busy. The `state_report` that answers finishes the connect.
                self.send(commands::state(self.next_message_id()));
            }
            FromDevice::Sample(sample) => self.absorb_sample(&sample),
            FromDevice::ZoneHit(hit) => {
                let event = {
                    let mut inner = self.inner.lock().unwrap();
                    let counts_per_cm = inner.axes.first().map(|a| a.counts_per_cm).unwrap_or(1.0);
                    let zone = hit.zone as usize;
                    let name = inner
                        .armed
                        .as_ref()
                        .and_then(|armed| armed.set.zones.get(zone))
                        .map(|zone| zone.name.clone());
                    // The board reports its raw counts; where the zone fired is
                    // the displacement in the host's frame, the one the bounds
                    // were compiled in.
                    let displacement = hit.c.first().map(|raw| {
                        let axis = &inner.axes[0];
                        axis.host(raw + inner.continuity.offset(0)) - axis.origin
                    });
                    if let Some(armed) = inner.armed.as_mut() {
                        if let Some(slot) = armed.fired_at.get_mut(zone) {
                            *slot = displacement;
                        }
                    }
                    let host_ns = inner.clock.observe(hit.t_us, monotonic_ns());
                    name.map(|zone| ZoneHitEvent {
                        zone,
                        arm_id: hit.arm_id,
                        seq: hit.seq,
                        host_monotonic_ns: host_ns,
                        position_cm: displacement.unwrap_or(0) as f64 / counts_per_cm,
                    })
                };
                if let Some(event) = event {
                    let _ = self.frames.send(StreamFrame::ZoneHit(event));
                }
            }
            FromDevice::Armed(armed) => {
                let mismatch = {
                    let mut inner = self.inner.lock().unwrap();
                    // The origin the board set when it read the arm, which is
                    // later than the sample the host's mirror was set from.
                    for (index, origin) in armed.origin.iter().enumerate() {
                        let origin = origin + inner.continuity.offset(index);
                        if let Some(axis) = inner.axes.get_mut(index) {
                            axis.origin = axis.host(origin);
                        }
                    }
                    match inner.armed.as_mut() {
                        Some(set) if set.arm_id == armed.arm_id => {
                            // The board says which version it armed. If that is
                            // not the one just uploaded, the board is running a
                            // set nobody on this host has seen — worth a word,
                            // because every zone in it is a line that fires
                            // somewhere unexpected.
                            let expected = set.set.version;
                            set.confirmed = true;
                            (expected != armed.zone_set_version)
                                .then_some((expected, armed.zone_set_version))
                        }
                        _ => None,
                    }
                };
                if let Some((expected, armed_version)) = mismatch {
                    self.note_error(&format!(
                        "the board armed zone-set version {armed_version}, not the {expected} just uploaded"
                    ));
                }
                self.armed_ack.notify_all();
            }
            FromDevice::StateReport(report) => {
                if std::mem::take(&mut self.inner.lock().unwrap().taking_over) {
                    self.take_over(report.armed.as_ref().map(|armed| armed.arm_id));
                }
                let mut inner = self.inner.lock().unwrap();
                inner.ring_drops = report.ring_drops;
                // The board's origin is the one the zones are evaluated
                // against, so where the host's mirror has drifted — a `zero`
                // that crossed a reset, a sample lost at the wrong moment — the
                // board wins.
                // The board's raw counts, against what the host is publishing.
                // They differ by the continuity offset, so a *change* in the
                // difference is the interesting thing — it means the host and
                // the board disagree about how far the wheel has gone, which no
                // other number would show.
                for (index, counts) in report.c.iter().enumerate() {
                    if let Some(axis) = inner.axes.get(index) {
                        log::trace!(
                            "{}: board {counts} counts, published {}",
                            axis.name,
                            axis.counts
                        );
                    }
                }
                for (index, origin) in report.origin.iter().enumerate() {
                    // In the host's frame: offset and sign, as the samples are.
                    let origin = origin + inner.continuity.offset(index);
                    if let Some(axis) = inner.axes.get_mut(index) {
                        let origin = &axis.host(origin);
                        if axis.origin != *origin {
                            log::debug!(
                                "{}: adopting the board's origin {origin} (mirror had {})",
                                axis.name,
                                axis.origin
                            );
                            axis.origin = *origin;
                        }
                    }
                }
                // A total that went down is a board that reset and counts afresh.
                let before = if report.scan_overruns < inner.scan_overruns {
                    0
                } else {
                    inner.scan_overruns
                };
                inner.scan_overruns = report.scan_overruns;
                if report.scan_overruns > before {
                    drop(inner);
                    // A scan was late. The device's timing is what this whole
                    // system is for, so this is a finding and not a statistic.
                    self.note_error(&format!(
                        "the board missed {} scans ({} since it started)",
                        report.scan_overruns - before,
                        report.scan_overruns
                    ));
                }
            }
            FromDevice::Error(error) => {
                // The refusal names the line it refused where it could read one,
                // which is the difference between "the board is unhappy" and
                // "the board refused the third zone of the set you just sent".
                match error.answers {
                    Some(message_id) => self.note_error(&format!(
                        "{}: {} (refusing message {message_id})",
                        error.code, error.detail
                    )),
                    None => self.note_error(&format!("{}: {}", error.code, error.detail)),
                }
            }
            FromDevice::Log(line) => log::info!("board: {}", line.text),
            FromDevice::Ok(ack) => log::trace!("board acknowledged {}", ack.answers),
            FromDevice::Pong(ack) => log::trace!("board answered ping {}", ack.answers),
        }
    }

    /// One sample: loss, the clock, continuity, the mirrors — then the segment,
    /// and only then the broadcast.
    ///
    /// **The segment is written first of the three publications.** vstimd reads
    /// it inside the frame it is drawing and cannot wait for a socket send; the
    /// stream's subscribers can wait for anything.
    fn absorb_sample(&self, sample: &WireSample) {
        let host_now = monotonic_ns();
        let (frame, centimetres) = {
            let mut inner = self.inner.lock().unwrap();

            // `seq` is contiguous on this wire, so a jump is loss — unlike the
            // daemon's own stream, where decimation skips it by design.
            let mut lost_before = 0;
            if inner.seq != 0 && sample.seq > inner.seq + 1 {
                lost_before = sample.seq - inner.seq - 1;
                inner.seq_gaps += lost_before;
            }
            inner.seq = sample.seq;

            // A device clock that went backwards is a board that restarted
            // without saying hello.
            if sample.t_us < inner.last_device_us {
                inner.continuity.device_restarted();
            }
            inner.last_device_us = sample.t_us;
            let host_monotonic_ns = inner.clock.observe(sample.t_us, host_now);

            if let Some(previous) = inner.last_sample_at {
                let interval = previous.elapsed().as_secs_f64();
                if interval > 0.0 {
                    inner.measured_rate_hz = 0.98 * inner.measured_rate_hz + 0.02 / interval;
                }
            }
            inner.last_sample_at = Some(Instant::now());

            let published = inner.continuity.publish(&sample.c);
            for (index, counts) in published.iter().enumerate() {
                let Some(axis) = inner.axes.get_mut(index) else { continue };
                // Inversion is applied once, here, so nothing above carries a sign.
                let counts = axis.host(*counts);
                axis.distance += (counts - axis.counts).abs();
                axis.counts = counts;
                if let Some(velocity) = sample.v.get(index) {
                    let velocity = axis.sign() * velocity;
                    axis.device_velocity_cm_s = velocity as f64 / axis.counts_per_cm;
                }
            }

            let leading = inner.axes.first().map(|axis| axis.counts).unwrap_or(0);
            inner.history.push_back((host_monotonic_ns, leading));
            while inner
                .history
                .front()
                .is_some_and(|(at, _)| host_monotonic_ns.saturating_sub(*at) > VELOCITY_SPAN_NS)
            {
                inner.history.pop_front();
            }
            if let Some((first_at, first_counts)) = inner.history.front().copied() {
                let span = host_monotonic_ns.saturating_sub(first_at) as f64 / 1e9;
                if span > 0.0 {
                    if let Some(axis) = inner.axes.first_mut() {
                        axis.host_velocity_cm_s =
                            (leading - first_counts) as f64 / axis.counts_per_cm / span;
                    }
                }
            }

            // Centimetres, from the published accumulator: what every
            // consumer of this daemon is told, and the only unit in the
            // segment.
            let centimetres: Vec<f64> = inner
                .axes
                .iter()
                .map(|axis| axis.counts as f64 / axis.counts_per_cm)
                .collect();
            (
                Sample {
                    seq: sample.seq,
                    device_us: sample.t_us,
                    host_monotonic_ns,
                    lost_before,
                    axes: inner.axes.iter().map(axis_state).collect(),
                },
                centimetres,
            )
        };
        if let Some(publisher) = self.publisher.as_ref() {
            publisher.publish(&centimetres);
        }
        let _ = self.frames.send(StreamFrame::Sample(frame));
    }

    fn note_error(&self, problem: &str) {
        self.inner.lock().unwrap().last_error = Some(problem.to_string());
        log::warn!("device: {problem}");
    }

    // --------------------------------------------------------- sending ---

    fn next_message_id(&self) -> u16 {
        let mut inner = self.inner.lock().unwrap();
        inner.message_id = inner.message_id.wrapping_add(1);
        inner.message_id
    }

    fn send(&self, command: Command) {
        let frame = command.encode();
        self.log_wire(WireDirection::Out, command.describe(), WireLevel::Info);
        let mut writer = self.writer.lock().unwrap();
        if let Some(writer) = writer.as_mut() {
            if let Err(problem) = writer.write_frame(&frame) {
                log::warn!("device: could not write {}: {problem}", command.name());
            }
        }
    }

    // ---------------------------------------------------- the API's view ---

    pub fn connected(&self) -> bool {
        self.inner.lock().unwrap().connected
    }

    pub fn subscribe_frames(&self) -> broadcast::Receiver<StreamFrame> {
        self.frames.subscribe()
    }

    pub fn subscribe_wire(&self) -> broadcast::Receiver<WireLine> {
        self.wire.subscribe()
    }

    pub fn capacities(&self) -> Capacities {
        self.inner.lock().unwrap().capacities.clone()
    }

    /// What this daemon speaks, and what the attached board speaks.
    pub fn protocol(&self) -> (u32, u32, Option<u32>) {
        let board = self.inner.lock().unwrap().protocol_version;
        (
            PROTOCOL_VERSION,
            OLDEST_PROTOCOL_SPOKEN,
            (board > 0).then_some(board),
        )
    }

    pub fn log_wire(&self, direction: WireDirection, text: impl Into<String>, level: WireLevel) {
        let text = text.into();
        let is_sample = text.contains(r#""msg_type":"sample""#);
        let line = WireLine {
            host_monotonic_ns: monotonic_ns(),
            direction,
            text,
            level,
        };
        {
            let (mut log, limit) = if is_sample {
                (self.samples.lock().unwrap(), SAMPLE_LINES)
            } else {
                (self.conversation.lock().unwrap(), CONVERSATION_LINES)
            };
            log.push_back(line.clone());
            while log.len() > limit {
                log.pop_front();
            }
        }
        let _ = self.wire.send(line);
    }

    /// The wire's recent past, in time order: **all** of the conversation, and
    /// the last `most_samples` samples merged back into it.
    ///
    /// The asymmetry is the point. Truncating the merged list to a total would
    /// hand back a second of samples and none of the conversation, which is the
    /// opposite of what somebody opening the monitor came for.
    pub fn wire_log(&self, most_samples: usize) -> Vec<WireLine> {
        let samples = self.samples.lock().unwrap();
        let recent = samples.iter().skip(samples.len().saturating_sub(most_samples));
        let mut lines: Vec<WireLine> = self
            .conversation
            .lock()
            .unwrap()
            .iter()
            .cloned()
            .chain(recent.cloned())
            .collect();
        lines.sort_by_key(|line| line.host_monotonic_ns);
        lines
    }

    pub fn info(&self, port: String) -> DeviceInfo {
        let inner = self.inner.lock().unwrap();
        DeviceInfo {
            connected: inner.connected,
            port,
            board: if inner.board.is_empty() { "none".into() } else { inner.board.clone() },
            firmware_version: inner.firmware.clone(),
            protocol_version: inner.protocol_version,
            uptime_device_us: inner.last_device_us,
            capacities: inner.capacities.clone(),
            flashed_zone_set: inner.flashed.clone(),
            link: LinkStats {
                connection_count: inner.connection_count,
                last_error: inner.last_error.clone(),
            },
        }
    }

    pub fn state(&self) -> RigState {
        let inner = self.inner.lock().unwrap();
        let stale = inner
            .last_sample_at
            .map(|at| at.elapsed() > STALE_AFTER)
            .unwrap_or(true);
        RigState {
            axes: inner.axes.iter().map(axis_state).collect(),
            health: LinkHealth {
                seq_gaps: inner.seq_gaps,
                ring_drops: inner.ring_drops,
                measured_rate_hz: inner.measured_rate_hz,
                stale,
            },
            link: LinkState { connected: inner.connected },
        }
    }

    /// Move the origin — on the board, and on the host's mirror of it.
    pub fn zero(&self, axes: &[String]) {
        let indices: Vec<usize> = {
            let mut inner = self.inner.lock().unwrap();
            let mut indices = Vec::new();
            for index in 0..inner.axes.len() {
                let selected =
                    axes.is_empty() || axes.iter().any(|name| name == &inner.axes[index].name);
                if selected {
                    inner.axes[index].origin = inner.axes[index].counts;
                    inner.axes[index].distance = 0;
                    indices.push(index);
                }
            }
            indices
        };
        self.send(commands::zero(self.next_message_id(), &indices));
    }

    /// Set the origin so that position_cm matches the requested value.
    ///
    /// This is for vstimd corridor synchronization: when vstimd resets the
    /// camera position, mousewheeld can sync the wheel position to match.
    /// The origin is adjusted so that `position_cm = counts - origin` equals
    /// the requested value.
    pub fn set_position(&self, axes: &[String], positions_cm: &[f64]) {
        let mut inner = self.inner.lock().unwrap();
        let indices: Vec<usize> = {
            let mut indices = Vec::new();
            for index in 0..inner.axes.len() {
                let selected = axes.is_empty() || axes.iter().any(|name| name == &inner.axes[index].name);
                if selected {
                    let position_cm = positions_cm.get(index).copied().unwrap_or(0.0);
                    let counts_at_position = (position_cm * inner.axes[index].counts_per_cm) as i64;
                    // origin = counts - counts_at_position
                    // so that position_cm = counts - origin = counts_at_position
                    inner.axes[index].origin = inner.axes[index].counts - counts_at_position;
                    indices.push(index);
                }
            }
            indices
        };
        drop(inner);
        
        // The board doesn't support absolute positioning; we'll just zero on the device
        // and the host will maintain the correct origin. The board's view of origin
        // will be corrected on the next state report.
        self.send(commands::zero(self.next_message_id(), &indices));
    }

    /// Upload a compiled set, arm it, and wait for the board to say so.
    ///
    /// `None` if the board did not acknowledge. Reporting an arm that never
    /// happened is worse than refusing: a trial would run believing a line is
    /// armed that is not.
    pub fn arm(
        &self,
        set: CompiledZoneSet,
        origin: ArmOrigin,
        label: Option<String>,
    ) -> Option<u32> {
        let arm_id = u32::from(rand::random::<u16>()) + 1000;
        let version = set.version;
        let zone_count = set.zones.len();
        let inverted: Vec<bool> =
            self.inner.lock().unwrap().axes.iter().map(|axis| axis.invert).collect();

        // Chunked, and the board's live set is untouched until `zones_end`.
        self.send(commands::zones_begin(
            self.next_message_id(),
            version,
            zone_count,
            &set.name,
        ));
        for (index, zone) in set.zones.iter().enumerate() {
            self.send(commands::zone(self.next_message_id(), wire_zone(index, zone, &inverted)));
        }
        self.send(commands::zones_end(self.next_message_id()));

        {
            let mut inner = self.inner.lock().unwrap();
            if origin == ArmOrigin::Current {
                for axis in inner.axes.iter_mut() {
                    axis.origin = axis.counts;
                    axis.distance = 0;
                }
            }
            inner.armed = Some(ArmedSet {
                fired_at: vec![None; zone_count],
                set,
                arm_id,
                label,
                confirmed: false,
            });
        }
        self.send(commands::arm(
            self.next_message_id(),
            arm_id,
            version,
            origin == ArmOrigin::Current,
        ));

        let inner = self.inner.lock().unwrap();
        let (mut inner, timeout) = self
            .armed_ack
            .wait_timeout_while(inner, ARM_ACK_TIMEOUT, |inner| {
                !inner.armed.as_ref().is_some_and(|armed| armed.confirmed)
            })
            .expect("the device lock");
        if timeout.timed_out() {
            inner.armed = None;
            return None;
        }
        Some(arm_id)
    }

    pub fn disarm(&self) {
        let arm_id = {
            let mut inner = self.inner.lock().unwrap();
            let arm_id = inner
                .armed
                .take()
                .map(|armed| armed.arm_id)
                .or(inner.foreign_arm_id)
                .unwrap_or(0);
            inner.foreign_arm_id = None;
            arm_id
        };
        self.send(commands::disarm(self.next_message_id(), arm_id));
    }

    /// Finish a connect, once the board has said whether it is armed.
    ///
    /// An unarmed board gets everything it needs to be useful, in the order it
    /// needs it: how many axes to count, then the wiring, because a zone names
    /// an axis and a line by index and a board without them refuses the zone;
    /// then the stream. Zones follow only when somebody arms.
    ///
    /// An armed board keeps its axes and its lines, because it refuses both
    /// while armed and because its zones were compiled against them. The arm is
    /// left alone: it is this daemon's across a reopened link, or it is the
    /// flashed set the board came up with, and a board that runs its set with
    /// no host attached is what flashing is for.
    fn take_over(&self, board_arm: Option<u32>) {
        match board_arm {
            None => {
                let axes = {
                    let inner = self.inner.lock().unwrap();
                    inner.axes.len().min(usize::from(inner.capacities.n_axes)).max(1)
                };
                self.send(commands::axes(self.next_message_id(), axes));
                self.send(commands::lines(self.next_message_id(), &self.lines));
            }
            Some(arm_id) => {
                let mut inner = self.inner.lock().unwrap();
                if inner.armed.as_ref().map(|armed| armed.arm_id) != Some(arm_id) {
                    inner.foreign_arm_id = Some(arm_id);
                    let flashed = inner
                        .flashed
                        .as_ref()
                        .map(|set| format!(", flashed zone set {} v{}", set.name, set.version))
                        .unwrap_or_default();
                    log::warn!(
                        "the board is armed as {arm_id}, not by this daemon{flashed}: its axes, \
                         lines and zones stay as they are until it is disarmed"
                    );
                }
            }
        }
        self.send(commands::stream(self.next_message_id(), self.stream_rate_hz, true));
    }

    /// Ask the board to keep the armed set across a power cycle.
    pub fn save_to_flash(&self) -> Option<FlashedZoneSet> {
        let flashed = {
            let inner = self.inner.lock().unwrap();
            let armed = inner.armed.as_ref()?;
            FlashedZoneSet { name: armed.set.name.clone(), version: armed.set.version }
        };
        self.send(commands::save(self.next_message_id()));
        // Noted here, and settled by the board's next `hello_ack`: what is on
        // the flash is the board's to report, not the host's to assume.
        self.inner.lock().unwrap().flashed = Some(flashed.clone());
        Some(flashed)
    }

    pub fn armed_zones(&self) -> (Option<ArmSummary>, Vec<ZoneStatus>) {
        let inner = self.inner.lock().unwrap();
        let Some(armed) = inner.armed.as_ref() else {
            return (None, Vec::new());
        };
        let counts_per_cm = inner.axes.first().map(|a| a.counts_per_cm).unwrap_or(1.0);
        let zones = armed
            .set
            .zones
            .iter()
            .enumerate()
            .map(|(index, zone)| {
                let fired_at = armed.fired_at.get(index).copied().flatten();
                ZoneStatus {
                    name: zone.name.clone(),
                    // The board disarms a `once` zone when it fires; the host
                    // mirrors that from the hit rather than asking.
                    armed: !(fired_at.is_some() && zone.fire == FireRule::Once),
                    fired: fired_at.is_some(),
                    inside: false,
                    fired_at_cm: fired_at.map(|counts| counts as f64 / counts_per_cm),
                    // The bounds as armed: what the board is comparing against,
                    // in centimetres again.
                    min_cm: zone
                        .min_counts
                        .iter()
                        .map(|bound| bound.map(|counts| counts as f64 / counts_per_cm))
                        .collect(),
                    max_cm: zone
                        .max_counts
                        .iter()
                        .map(|bound| bound.map(|counts| counts as f64 / counts_per_cm))
                        .collect(),
                    wrap_cm: zone.wrap_counts.map(|counts| counts as f64 / counts_per_cm),
                    metric: zone.metric,
                }
            })
            .collect();
        (
            Some(ArmSummary {
                zone_set: armed.set.name.clone(),
                version: armed.set.version,
                arm_id: armed.arm_id,
                label: armed.label.clone(),
            }),
            zones,
        )
    }

    pub fn counts_of(&self, axis_name: &str) -> Option<i64> {
        let inner = self.inner.lock().unwrap();
        inner
            .axes
            .iter()
            .find(|axis| axis.name == axis_name)
            .map(|axis| axis.counts)
    }

    /// The segment this daemon is publishing to, and how many writes have
    /// reached it — the number a reader's own count is compared against when
    /// somebody asks why a corridor is not moving.
    pub fn publishing(&self) -> Option<(String, u64)> {
        self.publisher
            .as_ref()
            .map(|publisher| (publisher.name().to_string(), publisher.write_count()))
    }

    /// Turn an axis' direction over while the daemon runs.
    ///
    /// **Nothing moves at the flip; the direction reverses from then on.** The
    /// published accumulator is what vstimd follows, and it differences
    /// successive totals: negating it would move a corridor by twice the
    /// distance walked. So the bias is chosen to keep the accumulator where it
    /// is, and the origin is mirrored about it — the position was reached
    /// going the way that is now called backwards, so its sign changes.
    ///
    /// Refused while a set is armed: its zones went to the board in the
    /// direction they were compiled for.
    pub fn set_invert(&self, axis_name: &str, invert: bool) -> Result<(), String> {
        let mut inner = self.inner.lock().unwrap();
        let Some(index) = inner.axes.iter().position(|axis| axis.name == axis_name) else {
            return Ok(());
        };
        if inner.axes[index].invert == invert {
            return Ok(());
        }
        if inner.armed.is_some() || inner.foreign_arm_id.is_some() {
            return Err(format!("disarm before changing the direction of {axis_name}"));
        }
        let axis = &mut inner.axes[index];
        let at = axis.counts;
        axis.invert = invert;
        axis.bias = 2 * at - axis.bias;
        axis.origin = 2 * at - axis.origin;
        axis.host_velocity_cm_s = -axis.host_velocity_cm_s;
        axis.device_velocity_cm_s = -axis.device_velocity_cm_s;
        if index == 0 {
            for (_, counts) in inner.history.iter_mut() {
                *counts = 2 * at - *counts;
            }
        }
        Ok(())
    }

    pub fn recalibrate(&self, axis_name: &str, counts_per_cm: f64) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(axis) = inner.axes.iter_mut().find(|axis| axis.name == axis_name) {
            axis.counts_per_cm = counts_per_cm;
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

/// Whether this daemon will talk to a board that greeted with `board`.
///
/// A floor and no ceiling. Newer firmware is fine — the protocol gains fields
/// additively and a reader ignores what it does not know — and older firmware
/// than the floor is refused by name rather than half-understood.
fn speaks_protocol(board: u32) -> bool {
    board >= OLDEST_PROTOCOL_SPOKEN
}

/// A compiled zone, as the wire carries it: indices and counts, nothing else.
///
/// The set is compiled in the host's frame and the board counts in its own,
/// uninverted, so a displacement interval on an inverted axis is mirrored on
/// the way out: the board's intervals are closed, and `lo ≤ -x ≤ hi` is exactly
/// `-hi ≤ x ≤ -lo`. Distance has no sign and is not touched.
fn wire_zone(index: usize, zone: &CompiledZone, inverted: &[bool]) -> WireZone {
    let wrap = zone.wrap_counts.unwrap_or(0);
    WireZone {
        index: index as u32,
        intervals: zone
            .axis_indices
            .iter()
            .enumerate()
            .map(|(i, axis)| {
                let lo = zone.min_counts.get(i).copied().flatten();
                let hi = zone.max_counts.get(i).copied().flatten();
                let mirrored = zone.metric == ZoneMetric::Displacement
                    && inverted.get(*axis).copied().unwrap_or(false);
                let (lo, hi) = if mirrored { mirror(lo, hi, wrap) } else { (lo, hi) };
                Interval { axis: *axis as u32, lo, hi }
            })
            .collect(),
        metric: match zone.metric {
            ZoneMetric::Displacement => Metric::Displacement,
            ZoneMetric::Distance => Metric::Distance,
        } as i32,
        wrap: zone.wrap_counts.unwrap_or(0),
        fire: match zone.fire {
            FireRule::Once => Fire::Once,
            FireRule::Rearm => Fire::Rearm,
        } as i32,
        hysteresis: zone.hysteresis_counts,
        level_on_arm: zone.level,
        line: u32::from(zone.line_index),
        action: Action::Pulse as i32,
        pulse_ms: ZONE_PULSE_MS,
    }
}

/// `[lo, hi]` → `[-hi, -lo]`, and on a wrapped track brought back into the
/// lap, which is where the board compares a wrapped displacement.
///
/// The board has no interval that crosses the end of a lap, so the one case
/// without an exact image is a lap interval starting at 0: `[0, hi]` mirrors to
/// `[w - hi, w]`, whose `w` is the lap's 0 again. It goes out as `[w - hi, w]`,
/// which the board's `value ≤ hi` reads as "to the end of the lap" — every
/// count but the lap's first, one count short of exact.
fn mirror(lo: Option<i64>, hi: Option<i64>, wrap: i64) -> (Option<i64>, Option<i64>) {
    let (lo, hi) = (hi.map(|hi| -hi), lo.map(|lo| -lo));
    if wrap <= 0 {
        return (lo, hi);
    }
    let lo = lo.map(|lo| lo.rem_euclid(wrap));
    let hi = hi.map(|hi| match hi.rem_euclid(wrap) {
        0 => wrap,
        hi => hi,
    });
    (lo, hi)
}

/// Bytes the link could not read, for the monitor: a refused frame is shown as
/// what arrived, not as what it might have meant.
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect::<Vec<_>>().join(" ")
}

fn axis_state(axis: &AxisRuntime) -> AxisState {
    AxisState {
        name: axis.name.clone(),
        counts: axis.counts,
        position_cm: (axis.counts - axis.origin) as f64 / axis.counts_per_cm,
        distance_cm: axis.distance as f64 / axis.counts_per_cm,
        velocity_cm_s: axis.host_velocity_cm_s,
        device_velocity_cm_s: axis.device_velocity_cm_s,
    }
}

/// `CLOCK_MONOTONIC` nanoseconds — the join key with statemachined's trace and
/// vstimd's vblank timestamps.
pub fn monotonic_ns() -> u64 {
    // SAFETY: a valid out-pointer for the duration of the call.
    let mut spec = unsafe { std::mem::zeroed::<libc::timespec>() };
    // SAFETY: as above.
    unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut spec) };
    spec.tv_sec as u64 * 1_000_000_000 + spec.tv_nsec as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_board_older_than_the_floor_is_refused_and_a_newer_one_is_not() {
        assert!(!speaks_protocol(0), "a board that greeted with nothing");
        assert!(speaks_protocol(OLDEST_PROTOCOL_SPOKEN));
        assert!(
            speaks_protocol(PROTOCOL_VERSION + 5),
            "newer firmware is fine: the protocol grows additively and a reader \
             ignores what it does not know"
        );
    }

    fn sample(seq: u64, c: i64) -> WireSample {
        WireSample { seq, t_us: seq * 2000, c: vec![c], v: vec![] }
    }

    #[test]
    fn turning_invert_over_reverses_the_direction_and_moves_nothing() {
        let device = Device::new(
            Backend::Absent,
            vec![AxisSetup { name: "wheel".into(), counts_per_cm: 10.0, invert: false }],
            vec![],
            500,
            None,
        );
        device.absorb_sample(&sample(1, 0));
        device.absorb_sample(&sample(2, 300));
        assert_eq!(device.counts_of("wheel"), Some(300));

        device.set_invert("wheel", true).unwrap();
        assert_eq!(device.counts_of("wheel"), Some(300), "the accumulator does not jump");
        device.absorb_sample(&sample(3, 320));
        assert_eq!(device.counts_of("wheel"), Some(280), "the board counting up is now backwards");

        device.set_invert("wheel", false).unwrap();
        device.absorb_sample(&sample(4, 330));
        assert_eq!(device.counts_of("wheel"), Some(290));
    }

    #[test]
    fn a_displacement_zone_on_an_inverted_axis_goes_to_the_board_mirrored() {
        let zone = CompiledZone {
            name: "goal".into(),
            axis_indices: vec![0],
            metric: ZoneMetric::Displacement,
            min_counts: vec![Some(400)],
            max_counts: vec![None],
            wrap_counts: None,
            fire: FireRule::Once,
            hysteresis_counts: 0,
            level: false,
            line_index: 0,
        };
        let interval = |zone: &CompiledZone, inverted| wire_zone(0, zone, &[inverted]).intervals[0].clone();
        assert_eq!((interval(&zone, false).lo, interval(&zone, false).hi), (Some(400), None));
        assert_eq!((interval(&zone, true).lo, interval(&zone, true).hi), (None, Some(-400)));

        let distance = CompiledZone { metric: ZoneMetric::Distance, ..zone.clone() };
        assert_eq!(interval(&distance, true).lo, Some(400), "distance has no sign");

        // On a 1000-count lap: [100, 300] is [700, 900] the other way round,
        // and [0, 300] is [700, the end of the lap].
        let lap = CompiledZone {
            min_counts: vec![Some(100)],
            max_counts: vec![Some(300)],
            wrap_counts: Some(1000),
            ..zone.clone()
        };
        assert_eq!((interval(&lap, true).lo, interval(&lap, true).hi), (Some(700), Some(900)));
        let from_start = CompiledZone { min_counts: vec![Some(0)], ..lap };
        assert_eq!(
            (interval(&from_start, true).lo, interval(&from_start, true).hi),
            (Some(700), Some(1000))
        );
    }
}
