//! The board behind the API — and, until the firmware exists, a stand-in for it.
//!
//! **The seam is the point.** Everything above this module talks to a device
//! through `Device`: read the counts, arm a compiled set, move the origin,
//! watch the wire. What is under it today is a **simulated wheel** on a thread,
//! because there is no firmware yet; what goes under it at M2 is the real-time
//! thread with a serial port. The routes, the store, the compiler and the
//! console panels do not know the difference and will not change when it is
//! swapped.
//!
//! The simulation is not decoration. It evaluates zones **in counts**, off the
//! compiler's output, exactly as the firmware's scan will: that is what makes
//! the compiler's arithmetic testable before a board exists, and it is why an
//! inverted interval or a wrong line map is caught here rather than on a rig.
//! It also injects what a real link does and a happy path never shows — lost
//! samples — because a consumer that has never seen a gap has never been tested.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::sync::broadcast;

use crate::model::device::{
    Capacities, DeviceInfo, FlashedZoneSet, LinkStats, WireDirection, WireLevel, WireLine,
};
use crate::model::state::{AxisState, LinkHealth, LinkState, RigState, Sample, StreamFrame, ZoneHitEvent};
use crate::model::zone_set::{ArmOrigin, FireRule, ZoneMetric, ZoneStatus};
use crate::zones::{CompiledZone, CompiledZoneSet};

/// Which set is armed, and under what. Separate from the zones' own statuses
/// because "nothing is armed" is one absence, not one per zone.
pub struct ArmSummary {
    pub zone_set: String,
    pub version: u32,
    pub arm_id: u32,
    pub label: Option<String>,
}

/// One axis as the rig config describes it, for [`Device::new`].
pub struct AxisSetup {
    pub name: String,
    pub counts_per_cm: f64,
    pub invert: bool,
}

/// What the daemon has to talk to.
pub enum Backend {
    /// No board, and no pretence of one. Every route that needs the device
    /// refuses with `no_device` rather than inventing a reading.
    Absent,
    /// A wheel on a thread, for building against. `mousewheeld serve --simulate`.
    Simulated,
}

const SCAN_HZ: f64 = 200.0;
const WIRE_LOG_LINES: usize = 4000;
const STALE_AFTER: Duration = Duration::from_millis(500);

struct AxisRuntime {
    name: String,
    counts_per_cm: f64,
    /// The encoder is wired the other way round. Applied here, to the counting
    /// itself, so that nothing above this module carries a sign.
    invert: bool,
    counts: f64,
    origin_counts: f64,
    distance_counts: f64,
    host_velocity_cm_s: f64,
    device_velocity_cm_s: f64,
}

struct ZoneRuntime {
    zone: CompiledZone,
    inside: bool,
    fired: bool,
    armed: bool,
    fired_at_counts: Option<i64>,
}

struct Inner {
    axes: Vec<AxisRuntime>,
    seq: u64,
    seq_gaps: u64,
    ring_drops: u64,
    last_sample_at: Option<Instant>,
    measured_rate_hz: f64,
    connection_count: u64,
    last_error: Option<String>,
    armed: Option<ArmedSet>,
    flashed: Option<FlashedZoneSet>,
    /// The running animal, so the simulation has something to be.
    resting: bool,
    bout_ends_at: Instant,
    target_speed_cm_s: f64,
    next_loss_at: Instant,
    history: VecDeque<(Instant, f64)>,
}

struct ArmedSet {
    set: CompiledZoneSet,
    arm_id: u32,
    label: Option<String>,
    zones: Vec<ZoneRuntime>,
}

pub struct Device {
    backend: Backend,
    inner: Mutex<Inner>,
    wire_log: Mutex<VecDeque<WireLine>>,
    frames: broadcast::Sender<StreamFrame>,
    wire: broadcast::Sender<WireLine>,
    running: AtomicBool,
    capacities: Capacities,
}

impl Device {
    pub fn new(backend: Backend, axes: Vec<AxisSetup>) -> Arc<Self> {
        let now = Instant::now();
        let device = Arc::new(Self {
            backend,
            inner: Mutex::new(Inner {
                axes: axes
                    .into_iter()
                    .map(|axis| AxisRuntime {
                        name: axis.name,
                        counts_per_cm: axis.counts_per_cm,
                        invert: axis.invert,
                        counts: 0.0,
                        origin_counts: 0.0,
                        distance_counts: 0.0,
                        host_velocity_cm_s: 0.0,
                        device_velocity_cm_s: 0.0,
                    })
                    .collect(),
                seq: 0,
                seq_gaps: 0,
                ring_drops: 0,
                last_sample_at: None,
                measured_rate_hz: 0.0,
                connection_count: 0,
                last_error: None,
                armed: None,
                flashed: None,
                resting: true,
                bout_ends_at: now,
                target_speed_cm_s: 0.0,
                next_loss_at: now + Duration::from_secs(6),
                history: VecDeque::new(),
            }),
            wire_log: Mutex::new(VecDeque::new()),
            frames: broadcast::channel(1024).0,
            wire: broadcast::channel(1024).0,
            running: AtomicBool::new(false),
            capacities: Capacities {
                n_axes: 2,
                max_zones: 16,
                max_lines: 8,
                scan_hz: 5000,
            },
        });
        if matches!(device.backend, Backend::Simulated) {
            device.clone().spawn_simulation();
        }
        device
    }

    pub fn connected(&self) -> bool {
        matches!(self.backend, Backend::Simulated) && self.running.load(Ordering::Relaxed)
    }

    pub fn subscribe_frames(&self) -> broadcast::Receiver<StreamFrame> {
        self.frames.subscribe()
    }

    pub fn subscribe_wire(&self) -> broadcast::Receiver<WireLine> {
        self.wire.subscribe()
    }

    pub fn capacities(&self) -> Capacities {
        self.capacities.clone()
    }

    /// Note a line of the wire, for the monitor panel. Both directions, and
    /// uninterpreted: this is the log for the moment the layers stop agreeing.
    pub fn log_wire(&self, direction: WireDirection, text: impl Into<String>, level: WireLevel) {
        let line = WireLine {
            host_monotonic_ns: monotonic_ns(),
            direction,
            text: text.into(),
            level,
        };
        {
            let mut log = self.wire_log.lock().unwrap();
            log.push_back(line.clone());
            while log.len() > WIRE_LOG_LINES {
                log.pop_front();
            }
        }
        let _ = self.wire.send(line);
    }

    pub fn wire_log(&self, most: usize) -> Vec<WireLine> {
        let log = self.wire_log.lock().unwrap();
        log.iter().rev().take(most).rev().cloned().collect()
    }

    pub fn info(&self, port: String) -> DeviceInfo {
        let inner = self.inner.lock().unwrap();
        let simulated = matches!(self.backend, Backend::Simulated);
        DeviceInfo {
            connected: self.connected(),
            port,
            board: if simulated { "simulated".into() } else { "none".into() },
            firmware_version: if simulated { "0.0.0+simulated".into() } else { String::new() },
            protocol_version: 1,
            uptime_device_us: inner
                .last_sample_at
                .map(|_| (inner.seq as f64 / SCAN_HZ * 1e6) as u64)
                .unwrap_or(0),
            capacities: self.capacities.clone(),
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
            link: LinkState {
                connected: self.connected(),
            },
        }
    }

    /// Move the API origin. Never the accumulator: see `ZeroRequest`.
    pub fn zero(&self, axes: &[String]) {
        let mut inner = self.inner.lock().unwrap();
        for axis in inner.axes.iter_mut() {
            if axes.is_empty() || axes.iter().any(|name| name == &axis.name) {
                axis.origin_counts = axis.counts;
                axis.distance_counts = 0.0;
            }
        }
        drop(inner);
        self.log_wire(WireDirection::Out, r#"{"zero":{}}"#, WireLevel::Info);
    }

    pub fn arm(&self, set: CompiledZoneSet, origin: ArmOrigin, label: Option<String>) -> u32 {
        let arm_id = rand::random::<u16>() as u32 + 1000;
        self.log_wire(
            WireDirection::Out,
            format!(r#"{{"zones_begin":{{"zone_set_version":{},"n":{}}}}}"#, set.version, set.zones.len()),
            WireLevel::Info,
        );
        self.log_wire(
            WireDirection::Out,
            format!(r#"{{"arm":{{"arm_id":{arm_id},"origin":"{}"}}}}"#, match origin {
                ArmOrigin::Current => "current",
                ArmOrigin::Absolute => "absolute",
            }),
            WireLevel::Info,
        );
        {
            let mut inner = self.inner.lock().unwrap();
            if origin == ArmOrigin::Current {
                for axis in inner.axes.iter_mut() {
                    axis.origin_counts = axis.counts;
                    axis.distance_counts = 0.0;
                }
            }
            inner.armed = Some(ArmedSet {
                zones: set
                    .zones
                    .iter()
                    .cloned()
                    .map(|zone| ZoneRuntime {
                        zone,
                        inside: false,
                        fired: false,
                        armed: true,
                        fired_at_counts: None,
                    })
                    .collect(),
                set,
                arm_id,
                label,
            });
        }
        self.log_wire(
            WireDirection::In,
            format!(r#"{{"armed":{{"arm_id":{arm_id}}}}}"#),
            WireLevel::Info,
        );
        arm_id
    }

    pub fn disarm(&self) {
        self.inner.lock().unwrap().armed = None;
        self.log_wire(WireDirection::Out, r#"{"disarm":{}}"#, WireLevel::Info);
    }

    /// Write the armed set to the board's flash. The store on the host stays
    /// the source; a flashed set is a copy the board reports by name.
    pub fn save_to_flash(&self) -> Option<FlashedZoneSet> {
        let mut inner = self.inner.lock().unwrap();
        let armed = inner.armed.as_ref()?;
        let flashed = FlashedZoneSet {
            name: armed.set.name.clone(),
            version: armed.set.version,
        };
        inner.flashed = Some(flashed.clone());
        drop(inner);
        self.log_wire(WireDirection::Out, r#"{"save":{}}"#, WireLevel::Info);
        Some(flashed)
    }

    pub fn armed_zones(&self) -> (Option<ArmSummary>, Vec<ZoneStatus>) {
        let inner = self.inner.lock().unwrap();
        match &inner.armed {
            None => (None, Vec::new()),
            Some(armed) => {
                let counts_per_cm = inner.axes.first().map(|a| a.counts_per_cm).unwrap_or(1.0);
                (
                    Some(ArmSummary {
                        zone_set: armed.set.name.clone(),
                        version: armed.set.version,
                        arm_id: armed.arm_id,
                        label: armed.label.clone(),
                    }),
                    armed
                        .zones
                        .iter()
                        .map(|zone| ZoneStatus {
                            name: zone.zone.name.clone(),
                            armed: zone.armed,
                            fired: zone.fired,
                            inside: zone.inside,
                            fired_at_cm: zone
                                .fired_at_counts
                                .map(|counts| counts as f64 / counts_per_cm),
                            // Back to centimetres from the counts the board is
                            // comparing against — the bounds as armed, not as
                            // authored.
                            min_cm: zone
                                .zone
                                .min_counts
                                .iter()
                                .map(|bound| bound.map(|counts| counts as f64 / counts_per_cm))
                                .collect(),
                            max_cm: zone
                                .zone
                                .max_counts
                                .iter()
                                .map(|bound| bound.map(|counts| counts as f64 / counts_per_cm))
                                .collect(),
                            wrap_cm: zone
                                .zone
                                .wrap_counts
                                .map(|counts| counts as f64 / counts_per_cm),
                            metric: zone.zone.metric,
                        })
                        .collect(),
                )
            }
        }
    }

    /// The counter right now, for the calibration measurement.
    pub fn counts_of(&self, axis_name: &str) -> Option<i64> {
        let inner = self.inner.lock().unwrap();
        inner
            .axes
            .iter()
            .find(|axis| axis.name == axis_name)
            .map(|axis| axis.counts as i64)
    }

    /// Apply a new calibration to the running axes, so what the API reports in
    /// centimetres changes the moment the number behind it does.
    pub fn recalibrate(&self, axis_name: &str, counts_per_cm: f64) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(axis) = inner.axes.iter_mut().find(|axis| axis.name == axis_name) {
            axis.counts_per_cm = counts_per_cm;
        }
    }

    // ------------------------------------------------------- simulation ---

    fn spawn_simulation(self: Arc<Self>) {
        self.running.store(true, Ordering::Relaxed);
        {
            let mut inner = self.inner.lock().unwrap();
            inner.connection_count += 1;
        }
        self.log_wire(
            WireDirection::Out,
            r#"{"hello":{"protocol_version":1}}"#,
            WireLevel::Info,
        );
        self.log_wire(
            WireDirection::In,
            r#"{"hello_ack":{"board":"simulated","firmware":"0.0.0+simulated","n_axes":2,"max_zones":16}}"#,
            WireLevel::Info,
        );

        std::thread::Builder::new()
            .name("simulated-wheel".into())
            .spawn(move || {
                let period = Duration::from_secs_f64(1.0 / SCAN_HZ);
                let mut next = Instant::now();
                while self.running.load(Ordering::Relaxed) {
                    next += period;
                    let now = Instant::now();
                    if next > now {
                        std::thread::sleep(next - now);
                    } else {
                        next = now;
                    }
                    self.tick();
                }
            })
            .expect("the simulated wheel needs a thread");
    }

    fn tick(&self) {
        let now = Instant::now();
        let (sample, hits) = {
            let mut inner = self.inner.lock().unwrap();

            if now >= inner.bout_ends_at {
                inner.resting = !inner.resting;
                let seconds = if inner.resting { 2.0 } else { 6.0 };
                inner.bout_ends_at = now + Duration::from_secs_f64(seconds);
                inner.target_speed_cm_s = if inner.resting {
                    0.0
                } else {
                    12.0 + f64::from(rand::random::<u8>()) / 255.0 * 26.0
                };
            }

            let phase = inner.seq as f64 / SCAN_HZ;
            let wobble = 1.0 + 0.12 * (phase * 5.3).sin();
            let speed_cm_s = (inner.target_speed_cm_s * wobble).max(0.0);

            for axis in inner.axes.iter_mut() {
                let direction = if axis.invert { -1.0 } else { 1.0 };
                let step = direction * speed_cm_s * axis.counts_per_cm / SCAN_HZ;
                axis.counts += step;
                axis.distance_counts += step.abs();
                // The device's velocity is whole counts over its own window:
                // the same motion through a coarser sieve than the host's.
                let window_counts = (step * SCAN_HZ / 20.0).round();
                axis.device_velocity_cm_s = window_counts * 20.0 / axis.counts_per_cm;
            }

            let leading = inner.axes[0].counts;
            inner.history.push_back((now, leading));
            while inner
                .history
                .front()
                .is_some_and(|(at, _)| now.duration_since(*at) > Duration::from_millis(500))
            {
                inner.history.pop_front();
            }
            if let (Some((first_at, first_counts)), Some(axis)) =
                (inner.history.front().copied(), inner.axes.first())
            {
                let span = now.duration_since(first_at).as_secs_f64();
                if span > 0.0 {
                    let velocity = (leading - first_counts) / axis.counts_per_cm / span;
                    inner.axes[0].host_velocity_cm_s = velocity;
                }
            }

            inner.seq += 1;
            let mut lost_before = 0;
            if now >= inner.next_loss_at {
                // Lines the daemon never received. A consumer cannot read this
                // off `seq` — a decimated stream skips it by design — so the
                // sample carries the count.
                lost_before = 3;
                inner.seq += lost_before;
                inner.seq_gaps += 1;
                inner.next_loss_at = now + Duration::from_secs(7);
            }
            if let Some(previous) = inner.last_sample_at {
                let interval = now.duration_since(previous).as_secs_f64();
                if interval > 0.0 {
                    // A slow average, so the number is the link's rate and not
                    // the jitter of the last two samples.
                    inner.measured_rate_hz = 0.99 * inner.measured_rate_hz + 0.01 / interval;
                }
            }
            inner.last_sample_at = Some(now);

            let hits = evaluate_zones(&mut inner);
            let sample = Sample {
                seq: inner.seq,
                device_us: (inner.seq as f64 / SCAN_HZ * 1e6) as u64,
                host_monotonic_ns: monotonic_ns(),
                lost_before,
                axes: inner.axes.iter().map(axis_state).collect(),
            };
            (sample, hits)
        };

        for hit in hits {
            self.log_wire(
                WireDirection::In,
                format!(r#"{{"zone_hit":{{"zone":"{}","seq":{}}}}}"#, hit.zone, hit.seq),
                WireLevel::Info,
            );
            let _ = self.frames.send(StreamFrame::ZoneHit(hit));
        }
        let _ = self.frames.send(StreamFrame::Sample(sample));
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

/// Zone evaluation, in counts, on the scan that saw the movement — the shape
/// the firmware's own loop will have.
///
/// A zone fires on the **entry edge**: arming a zone the animal is already
/// inside does not fire it unless the zone says `level`. `once` disarms after
/// firing; `rearm` comes back once the position has left by the hysteresis.
fn evaluate_zones(inner: &mut Inner) -> Vec<ZoneHitEvent> {
    let Some(armed) = inner.armed.as_mut() else {
        return Vec::new();
    };
    let arm_id = armed.arm_id;
    let mut hits = Vec::new();
    for runtime in armed.zones.iter_mut() {
        let axis_index = runtime.zone.axis_indices[0];
        let axis = &inner.axes[axis_index];
        let raw = match runtime.zone.metric {
            ZoneMetric::Displacement => axis.counts - axis.origin_counts,
            ZoneMetric::Distance => axis.distance_counts,
        };
        let value = match runtime.zone.wrap_counts {
            Some(period) if period > 0 => raw.rem_euclid(period as f64),
            _ => raw,
        };
        let counts = value as i64;

        let low = runtime.zone.min_counts[0];
        let high = runtime.zone.max_counts[0];
        let inside = low.is_none_or(|low| counts >= low) && high.is_none_or(|high| counts <= high);

        if inside && !runtime.inside && runtime.armed {
            runtime.fired = true;
            runtime.fired_at_counts = Some(counts);
            if runtime.zone.fire == FireRule::Once {
                runtime.armed = false;
            }
            hits.push(ZoneHitEvent {
                zone: runtime.zone.name.clone(),
                arm_id,
                seq: inner.seq,
                host_monotonic_ns: monotonic_ns(),
                position_cm: counts as f64 / axis.counts_per_cm,
            });
        }

        if !inside && runtime.zone.fire == FireRule::Rearm {
            // Only past the hysteresis: a boundary the animal is standing on
            // must not be a pulse train.
            let left_by = low
                .map(|low| (low - counts).max(0))
                .unwrap_or(0)
                .max(high.map(|high| (counts - high).max(0)).unwrap_or(0));
            if left_by >= runtime.zone.hysteresis_counts {
                runtime.armed = true;
            }
        }
        runtime.inside = inside;
    }
    hits
}

fn axis_state(axis: &AxisRuntime) -> AxisState {
    AxisState {
        name: axis.name.clone(),
        counts: axis.counts as i64,
        position_cm: (axis.counts - axis.origin_counts) / axis.counts_per_cm,
        distance_cm: axis.distance_counts / axis.counts_per_cm,
        velocity_cm_s: axis.host_velocity_cm_s,
        device_velocity_cm_s: axis.device_velocity_cm_s,
    }
}

pub fn monotonic_ns() -> u64 {
    let mut spec = libc_timespec();
    // SAFETY: `spec` is a valid out-pointer for the duration of the call.
    unsafe { clock_gettime(CLOCK_MONOTONIC, &mut spec) };
    spec.tv_sec as u64 * 1_000_000_000 + spec.tv_nsec as u64
}

// The host's monotonic clock, which is the join key with statemachined's trace
// and vstimd's vblank timestamps. Declared here rather than pulling in a crate
// for two lines.
#[repr(C)]
struct Timespec {
    tv_sec: i64,
    tv_nsec: i64,
}
const CLOCK_MONOTONIC: i32 = 1;
extern "C" {
    fn clock_gettime(clock: i32, spec: *mut Timespec) -> i32;
}
fn libc_timespec() -> Timespec {
    Timespec { tv_sec: 0, tv_nsec: 0 }
}
