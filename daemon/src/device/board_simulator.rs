//! A board, in software, on the far end of a pty.
//!
//! **This is the firmware's stand-in, not the daemon's.** It speaks
//! `docs/reference/protocol.md` over a real file descriptor: it reads COBS
//! frames, checks their CRCs, decodes the protobuf, refuses what it does not understand, keeps counts
//! in a 64-bit accumulator, evaluates zones **in counts** on its own scan, and
//! emits `sample` and `zone_hit` at the rate it was asked for. The daemon on
//! the other side runs exactly the code it will run against a Teensy.
//!
//! What it is not is a reference implementation. The real firmware
//! (`firmware/`, and `mousewheeld_native_device` built from it for this
//! machine) has an interrupt, rings and no allocator; this has a thread and a
//! `Vec`. Where it is faithful is the **wire and the semantics** —
//! which is what the daemon can be wrong about.
//!
//! It also does the one thing a happy path never does: **it drops frames**. Every
//! few seconds a sample is not written while its `seq` is still consumed, which
//! is what a full ring on the device looks like from the host. A consumer that
//! has never seen a gap has never been tested.

use std::io;
use std::os::fd::OwnedFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use prost::Message;

use crate::link::framing::{check, seal};
use crate::link::serial::{duplicate, FrameReader, FrameWriter};
use crate::wire::link::{
    device_message::Body as Reply, host_message::Body as Command, Armed, DeviceMessage,
    Error as DeviceError, FlashedSet, HelloAck, HostMessage, Ok as Acknowledgement, Origin, Pong,
    Sample, StateReport, Zone, ZoneHit,
};

const SCAN_HZ: f64 = 1000.0;
const MAX_FRAME: usize = 256;
const MAX_ZONES: usize = 16;

struct ZoneRuntime {
    zone: Zone,
    inside: bool,
    armed: bool,
}

struct Board {
    counts: f64,
    origin: i64,
    distance: f64,
    seq: u64,
    message_id: u16,
    rate_hz: u32,
    velocity: bool,
    device_velocity: i64,
    /// Zones being uploaded, and the committed set. The live one is untouched
    /// until `zones_end`, so a link that dies mid-upload leaves the board
    /// running what it was running.
    staging: Vec<Zone>,
    staged_version: u32,
    staged_name: String,
    live: Vec<Zone>,
    live_version: u32,
    live_name: String,
    armed: Option<(u32, Vec<ZoneRuntime>)>,
    flashed: Option<(String, u32)>,
    ring_drops: u64,
    /// The animal.
    resting: bool,
    bout_ends_at: Instant,
    speed_counts_s: f64,
    next_drop_at: Instant,
}

pub struct BoardSimulator {
    board: Mutex<Board>,
    writer: Mutex<FrameWriter>,
    running: Arc<AtomicBool>,
    counts_per_cm: f64,
    started: Instant,
}

impl BoardSimulator {
    /// Take the pty's board end and start answering on it.
    ///
    /// `counts_per_cm` is the wheel this board is bolted to — the simulation
    /// needs it only to move at speeds an animal could, since everything it
    /// says on the wire is counts.
    pub fn spawn(fd: OwnedFd, counts_per_cm: f64) -> io::Result<Arc<AtomicBool>> {
        let reader_fd = duplicate(&fd)?;
        let now = Instant::now();
        let simulator = Arc::new(Self {
            board: Mutex::new(Board {
                counts: 0.0,
                origin: 0,
                distance: 0.0,
                seq: 0,
                message_id: 0,
                rate_hz: 0,
                velocity: true,
                device_velocity: 0,
                staging: Vec::new(),
                staged_version: 0,
                staged_name: String::new(),
                live: Vec::new(),
                live_version: 0,
                live_name: String::new(),
                armed: None,
                flashed: None,
                ring_drops: 0,
                resting: true,
                bout_ends_at: now,
                speed_counts_s: 0.0,
                next_drop_at: now + Duration::from_secs(5),
            }),
            writer: Mutex::new(FrameWriter::new(fd)),
            running: Arc::new(AtomicBool::new(true)),
            counts_per_cm,
            started: now,
        });

        let running = simulator.running.clone();

        let commands = simulator.clone();
        std::thread::Builder::new()
            .name("board-commands".into())
            .spawn(move || commands.serve_commands(FrameReader::new(reader_fd, MAX_FRAME)))?;

        let scan = simulator.clone();
        std::thread::Builder::new()
            .name("board-scan".into())
            .spawn(move || scan.run_scan())?;

        Ok(running)
    }

    fn t_us(&self) -> u64 {
        self.started.elapsed().as_micros() as u64
    }

    /// Seal and write one message, numbered with this board's own counter.
    fn send(&self, body: Reply) {
        let message_id = {
            let mut board = self.board.lock().unwrap();
            board.message_id = board.message_id.wrapping_add(1);
            u32::from(board.message_id)
        };
        let frame = seal(&DeviceMessage { message_id, body: Some(body) }.encode_to_vec());
        let _ = self.writer.lock().unwrap().write_frame(&frame);
    }

    fn refuse(&self, code: &str, detail: impl Into<String>, answers: Option<u32>) {
        self.send(Reply::Error(DeviceError {
            code: code.to_string(),
            detail: detail.into(),
            answers,
        }));
    }

    fn acknowledge(&self, answers: u32) {
        self.send(Reply::Ok(Acknowledgement { answers }));
    }

    // ------------------------------------------------------- commands ---

    fn serve_commands(&self, mut reader: FrameReader) {
        while self.running.load(Ordering::Relaxed) {
            let frames = match reader.read_frames() {
                Ok(frames) if frames.is_empty() => return, // the host closed the port
                Ok(frames) => frames,
                Err(_) => return,
            };
            for frame in frames {
                match check(&frame, MAX_FRAME) {
                    Ok(bytes) => self.handle(&bytes),
                    // Refused by name, never acted on — not even partially.
                    Err(problem) => self.refuse("bad_crc", problem.to_string(), None),
                }
            }
        }
    }

    fn handle(&self, bytes: &[u8]) {
        let Ok(message) = HostMessage::decode(bytes) else {
            self.refuse("bad_message", "not a host message", None);
            return;
        };
        let id = message.message_id;
        let Some(command) = message.body else {
            self.refuse("unknown_type", "a body this board does not know", Some(id));
            return;
        };

        match command {
            Command::Hello(_) => self.send_hello_ack(),
            Command::Ping(_) => self.send(Reply::Pong(Pong { answers: id })),
            Command::Stream(stream) => {
                let mut board = self.board.lock().unwrap();
                board.rate_hz = stream.rate_hz;
                board.velocity = stream.velocity;
                drop(board);
                self.acknowledge(id);
            }
            Command::Axes(_) | Command::Lines(_) | Command::Analog(_) | Command::Debug(_) => {
                self.acknowledge(id)
            }
            Command::ZonesBegin(begin) => {
                let mut board = self.board.lock().unwrap();
                board.staging.clear();
                board.staged_version = begin.zone_set_version;
                board.staged_name = begin.name;
            }
            Command::Zone(zone) => {
                let mut board = self.board.lock().unwrap();
                if board.staging.len() >= MAX_ZONES {
                    drop(board);
                    self.refuse("zone_table_full", format!("this board holds {MAX_ZONES} zones"), Some(id));
                    return;
                }
                board.staging.push(zone);
            }
            Command::ZonesEnd(_) => {
                let mut board = self.board.lock().unwrap();
                board.live = std::mem::take(&mut board.staging);
                board.live_version = board.staged_version;
                board.live_name = board.staged_name.clone();
                drop(board);
                self.acknowledge(id);
            }
            Command::Arm(arm) => {
                let origin_is_current = arm.origin != Origin::Absolute as i32;
                let (version, origin) = {
                    let mut board = self.board.lock().unwrap();
                    if origin_is_current {
                        board.origin = board.counts as i64;
                        board.distance = 0.0;
                    }
                    let zones = board
                        .live
                        .iter()
                        .cloned()
                        .map(|zone| ZoneRuntime { zone, inside: false, armed: true })
                        .collect();
                    board.armed = Some((arm.arm_id, zones));
                    (board.live_version, board.origin)
                };
                self.send(Reply::Armed(Armed {
                    arm_id: arm.arm_id,
                    zone_set_version: version,
                    origin: vec![origin],
                }));
            }
            Command::Disarm(_) => {
                self.board.lock().unwrap().armed = None;
                self.acknowledge(id);
            }
            Command::Zero(_) => {
                let mut board = self.board.lock().unwrap();
                board.origin = board.counts as i64;
                board.distance = 0.0;
                drop(board);
                self.acknowledge(id);
            }
            Command::Save(_) => {
                let mut board = self.board.lock().unwrap();
                if board.live.is_empty() {
                    drop(board);
                    self.refuse("nothing_to_save", "no zone set is committed", Some(id));
                    return;
                }
                board.flashed = Some((board.live_name.clone(), board.live_version));
                drop(board);
                self.acknowledge(id);
            }
            Command::State(_) => self.send_state_report(),
        }
    }

    fn send_hello_ack(&self) {
        let flashed = self.board.lock().unwrap().flashed.clone();
        self.send(Reply::HelloAck(HelloAck {
            board: "simulated".into(),
            firmware: "0.0.0+simulated".into(),
            protocol_version: 1,
            n_axes: 1,
            max_zones: MAX_ZONES as u32,
            max_lines: 8,
            max_frame: MAX_FRAME as u32,
            scan_hz: SCAN_HZ as u32,
            flashed: flashed.map(|(name, version)| FlashedSet { name, version }),
        }));
    }

    fn send_state_report(&self) {
        let board = self.board.lock().unwrap();
        let report = StateReport {
            c: vec![board.counts as i64],
            origin: vec![board.origin],
            distance: vec![board.distance as i64],
            v: vec![board.device_velocity],
            n_axes: 1,
            stream_rate_hz: board.rate_hz,
            ring_drops: board.ring_drops,
            ..Default::default()
        };
        drop(board);
        self.send(Reply::StateReport(report));
    }

    // ----------------------------------------------------------- scan ---

    /// The scan: move, evaluate zones, and emit at the stream divider.
    fn run_scan(&self) {
        let period = Duration::from_secs_f64(1.0 / SCAN_HZ);
        let mut next = Instant::now();
        let mut since_sample = 0.0_f64;
        while self.running.load(Ordering::Relaxed) {
            next += period;
            let now = Instant::now();
            if next > now {
                std::thread::sleep(next - now);
            } else {
                next = now;
            }

            let hits = self.step(now);
            for (zone, seq, counts) in hits {
                let arm_id = self
                    .board
                    .lock()
                    .unwrap()
                    .armed
                    .as_ref()
                    .map(|(id, _)| *id)
                    .unwrap_or(0);
                self.send(Reply::ZoneHit(ZoneHit {
                    seq,
                    arm_id,
                    zone: zone as u32,
                    t_us: self.t_us(),
                    c: vec![counts],
                }));
            }

            let rate_hz = self.board.lock().unwrap().rate_hz;
            if rate_hz == 0 {
                continue;
            }
            since_sample += 1.0 / SCAN_HZ;
            if since_sample < 1.0 / f64::from(rate_hz) {
                continue;
            }
            since_sample = 0.0;
            self.emit_sample(now);
        }
    }

    /// One scan's movement and zone evaluation. Returns `(zone index, seq,
    /// counts)`, the counts raw as the firmware reports them.
    fn step(&self, now: Instant) -> Vec<(usize, u64, i64)> {
        let mut board = self.board.lock().unwrap();

        if now >= board.bout_ends_at {
            board.resting = !board.resting;
            let seconds = if board.resting { 2.0 } else { 6.0 };
            board.bout_ends_at = now + Duration::from_secs_f64(seconds);
            board.speed_counts_s = if board.resting {
                0.0
            } else {
                (12.0 + f64::from(rand::random::<u8>()) / 255.0 * 26.0) * self.counts_per_cm
            };
        }
        let wobble = 1.0 + 0.12 * (board.seq as f64 / SCAN_HZ * 5.3).sin();
        let step = (board.speed_counts_s * wobble).max(0.0) / SCAN_HZ;
        board.counts += step;
        board.distance += step.abs();
        board.device_velocity = ((step * SCAN_HZ / 20.0).round() * 20.0) as i64;

        let counts = board.counts as i64;
        let displacement = counts - board.origin;
        let distance = board.distance as i64;
        let seq = board.seq;

        let Some((_, zones)) = board.armed.as_mut() else {
            return Vec::new();
        };
        let mut hits = Vec::new();
        for (index, runtime) in zones.iter_mut().enumerate() {
            let raw = if runtime.zone.metric == 1 { distance } else { displacement };
            let value = if runtime.zone.wrap > 0 {
                raw.rem_euclid(runtime.zone.wrap)
            } else {
                raw
            };
            let interval = runtime.zone.intervals.first().copied().unwrap_or_default();
            let inside = interval.lo.is_none_or(|low| value >= low)
                && interval.hi.is_none_or(|high| value <= high);

            if inside && !runtime.inside && runtime.armed {
                // The TTL is scheduled here, in the scan that saw the count.
                // The message that follows is only the record.
                if runtime.zone.fire == 0 {
                    runtime.armed = false;
                }
                hits.push((index, seq, counts));
            }
            if !inside && runtime.zone.fire == 1 {
                let left_by = interval
                    .lo
                    .map(|low| (low - value).max(0))
                    .unwrap_or(0)
                    .max(interval.hi.map(|high| (value - high).max(0)).unwrap_or(0));
                if left_by >= runtime.zone.hysteresis {
                    runtime.armed = true;
                }
            }
            runtime.inside = inside;
        }
        hits
    }

    fn emit_sample(&self, now: Instant) {
        let (seq, counts, velocity, dropped) = {
            let mut board = self.board.lock().unwrap();
            board.seq += 1;
            let dropped = now >= board.next_drop_at;
            if dropped {
                // A full ring: the line is never written, and its `seq` is gone
                // with it. The host sees the gap and says so.
                board.ring_drops += 1;
                board.next_drop_at = now + Duration::from_secs(7);
            }
            (
                board.seq,
                board.counts as i64,
                board.velocity.then_some(board.device_velocity),
                dropped,
            )
        };
        if dropped {
            return;
        }
        self.send(Reply::Sample(Sample {
            seq,
            t_us: self.t_us(),
            c: vec![counts],
            v: velocity.into_iter().collect(),
        }));
    }
}
