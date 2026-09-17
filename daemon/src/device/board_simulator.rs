//! A board, in software, on the far end of a pty.
//!
//! **This is the firmware's stand-in, not the daemon's.** It speaks
//! `docs/reference/protocol.md` over a real file descriptor: it parses framed
//! lines, checks their CRCs, refuses what it does not understand, keeps counts
//! in a 64-bit accumulator, evaluates zones **in counts** on its own scan, and
//! emits `sample` and `zone_hit` at the rate it was asked for. The daemon on
//! the other side runs exactly the code it will run against a Teensy.
//!
//! What it is not is a reference implementation. The real firmware has an
//! interrupt, rings, a fixed-format writer and no allocator; this has a thread
//! and `format!`. Where it is faithful is the **wire and the semantics** —
//! which is what the daemon can be wrong about.
//!
//! It also does the one thing a happy path never does: **it drops lines**. Every
//! few seconds a sample is not written while its `seq` is still consumed, which
//! is what a full ring on the device looks like from the host. A consumer that
//! has never seen a gap has never been tested.

use std::io;
use std::os::fd::OwnedFd;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::link::framing::{check, seal, FramingError};
use crate::link::messages::WireZone;
use crate::link::serial::{duplicate, LineReader, LineWriter};

const SCAN_HZ: f64 = 1000.0;
const MAX_LINE: usize = 512;

struct ZoneRuntime {
    zone: WireZone,
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
    staging: Vec<WireZone>,
    staged_version: u32,
    staged_name: String,
    live: Vec<WireZone>,
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
    writer: Mutex<LineWriter>,
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
            writer: Mutex::new(LineWriter::new(fd)),
            running: Arc::new(AtomicBool::new(true)),
            counts_per_cm,
            started: now,
        });

        let running = simulator.running.clone();

        let commands = simulator.clone();
        std::thread::Builder::new()
            .name("board-commands".into())
            .spawn(move || commands.serve_commands(LineReader::new(reader_fd, MAX_LINE)))?;

        let scan = simulator.clone();
        std::thread::Builder::new()
            .name("board-scan".into())
            .spawn(move || scan.run_scan())?;

        Ok(running)
    }

    fn t_us(&self) -> u64 {
        self.started.elapsed().as_micros() as u64
    }

    fn send(&self, body: &str) {
        let line = seal(body);
        let _ = self.writer.lock().unwrap().write_line(&line);
    }

    /// `msg_type` plus this board's own line counter, ready for more members.
    fn begin(&self, msg_type: &str) -> String {
        let mut board = self.board.lock().unwrap();
        board.message_id = board.message_id.wrapping_add(1);
        format!(r#"{{"msg_type":"{msg_type}","message_id":{}"#, board.message_id)
    }

    // ------------------------------------------------------- commands ---

    fn serve_commands(&self, mut reader: LineReader) {
        while self.running.load(Ordering::Relaxed) {
            let lines = match reader.read_lines() {
                Ok(lines) if lines.is_empty() => return, // the host closed the port
                Ok(lines) => lines,
                Err(_) => return,
            };
            for line in lines {
                match check(&line, MAX_LINE) {
                    Ok(body) => self.handle(body),
                    // Refused by name, never acted on — not even partially.
                    Err(FramingError::BadCrc { .. }) => {
                        self.send(&format!(
                            r#"{},"code":"bad_crc","detail":"the line did not survive the wire""#,
                            self.begin("error")
                        ));
                    }
                    Err(problem) => {
                        self.send(&format!(
                            r#"{},"code":"bad_frame","detail":"{problem}""#,
                            self.begin("error")
                        ));
                    }
                }
            }
        }
    }

    fn handle(&self, body: &str) {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(body) else {
            self.send(&format!(
                r#"{},"code":"bad_json","detail":"a line that is not an object""#,
                self.begin("error")
            ));
            return;
        };
        let msg_type = value.get("msg_type").and_then(|v| v.as_str()).unwrap_or("");
        let message_id = value.get("message_id").and_then(|v| v.as_u64()).unwrap_or(0);

        match msg_type {
            "hello" => self.send_hello_ack(),
            "ping" => self.send(&format!(r#"{},"answers":{message_id}"#, self.begin("pong"))),
            "stream" => {
                let mut board = self.board.lock().unwrap();
                board.rate_hz = value.get("rate_hz").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                board.velocity = value.get("velocity").and_then(|v| v.as_bool()).unwrap_or(true);
                drop(board);
                self.acknowledge(message_id);
            }
            "lines" | "axes" | "analog" | "debug" => self.acknowledge(message_id),
            "zones_begin" => {
                let mut board = self.board.lock().unwrap();
                board.staging.clear();
                board.staged_version =
                    value.get("zone_set_version").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                board.staged_name = value
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
            }
            "zone" => match serde_json::from_str::<WireZone>(body) {
                Ok(zone) => {
                    let mut board = self.board.lock().unwrap();
                    if board.staging.len() >= 16 {
                        board.ring_drops += 1;
                        drop(board);
                        self.send(&format!(
                            r#"{},"code":"zone_table_full","detail":"this board holds 16 zones","answers":{message_id}"#,
                            self.begin("error")
                        ));
                        return;
                    }
                    board.staging.push(zone);
                }
                Err(problem) => self.send(&format!(
                    r#"{},"code":"bad_zone","detail":"{problem}","answers":{message_id}"#,
                    self.begin("error")
                )),
            },
            "zones_end" => {
                let mut board = self.board.lock().unwrap();
                board.live = std::mem::take(&mut board.staging);
                board.live_version = board.staged_version;
                board.live_name = board.staged_name.clone();
                drop(board);
                self.acknowledge(message_id);
            }
            "arm" => {
                let arm_id = value.get("arm_id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let origin_is_current =
                    value.get("origin").and_then(|v| v.as_str()) != Some("absolute");
                let version = {
                    let mut board = self.board.lock().unwrap();
                    if origin_is_current {
                        board.origin = board.counts as i64;
                        board.distance = 0.0;
                    }
                    let zones = board
                        .live
                        .iter()
                        .cloned()
                        .map(|zone| ZoneRuntime {
                            zone,
                            inside: false,
                            armed: true,
                        })
                        .collect();
                    board.armed = Some((arm_id, zones));
                    board.live_version
                };
                self.send(&format!(
                    r#"{},"arm_id":{arm_id},"zone_set_version":{version}"#,
                    self.begin("armed")
                ));
            }
            "disarm" => {
                self.board.lock().unwrap().armed = None;
                self.acknowledge(message_id);
            }
            "zero" => {
                let mut board = self.board.lock().unwrap();
                board.origin = board.counts as i64;
                board.distance = 0.0;
                drop(board);
                self.acknowledge(message_id);
            }
            "save" => {
                let mut board = self.board.lock().unwrap();
                if board.live.is_empty() {
                    drop(board);
                    self.send(&format!(
                        r#"{},"code":"nothing_to_save","detail":"no zone set is committed","answers":{message_id}"#,
                        self.begin("error")
                    ));
                    return;
                }
                board.flashed = Some((board.live_name.clone(), board.live_version));
                drop(board);
                self.acknowledge(message_id);
            }
            "state" => self.send_state_report(),
            _ => self.send(&format!(
                r#"{},"code":"unknown_type","detail":"{msg_type}","answers":{message_id}"#,
                self.begin("error")
            )),
        }
    }

    fn acknowledge(&self, message_id: u64) {
        self.send(&format!(r#"{},"answers":{message_id}"#, self.begin("ok")));
    }

    fn send_hello_ack(&self) {
        let flashed = self.board.lock().unwrap().flashed.clone();
        let flashed = match flashed {
            Some((name, version)) => {
                format!(r#","flashed":{{"name":"{name}","version":{version}}}"#)
            }
            None => String::new(),
        };
        self.send(&format!(
            r#"{},"board":"simulated","firmware":"0.0.0+simulated","protocol_version":1,"n_axes":1,"max_zones":16,"max_lines":8,"max_line":{MAX_LINE},"scan_hz":{}{flashed}"#,
            self.begin("hello_ack"),
            SCAN_HZ as u32,
        ));
    }

    fn send_state_report(&self) {
        let board = self.board.lock().unwrap();
        let (counts, origin, drops) = (board.counts as i64, board.origin, board.ring_drops);
        drop(board);
        self.send(&format!(
            r#"{},"c":[{counts}],"origin":[{origin}],"ring_drops":{drops},"scan_overruns":0"#,
            self.begin("state_report")
        ));
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
                self.send(&format!(
                    r#"{},"seq":{seq},"arm_id":{arm_id},"zone":{zone},"t_us":{},"c":[{counts}]"#,
                    self.begin("zone_hit"),
                    self.t_us()
                ));
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

    /// One scan's movement and zone evaluation. Returns `(zone index, seq, counts)`.
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

        let displacement = board.counts as i64 - board.origin;
        let distance = board.distance as i64;
        let seq = board.seq;

        let Some((_, zones)) = board.armed.as_mut() else {
            return Vec::new();
        };
        let mut hits = Vec::new();
        for (index, runtime) in zones.iter_mut().enumerate() {
            let raw = if runtime.zone.m == 1 { distance } else { displacement };
            let value = if runtime.zone.wrap > 0 {
                raw.rem_euclid(runtime.zone.wrap)
            } else {
                raw
            };
            let inside = runtime.zone.lo[0].is_none_or(|low| value >= low)
                && runtime.zone.hi[0].is_none_or(|high| value <= high);

            if inside && !runtime.inside && runtime.armed {
                // The TTL is scheduled here, in the scan that saw the count.
                // The message that follows is only the record.
                if runtime.zone.fire == 0 {
                    runtime.armed = false;
                }
                hits.push((index, seq, value));
            }
            if !inside && runtime.zone.fire == 1 {
                let left_by = runtime.zone.lo[0]
                    .map(|low| (low - value).max(0))
                    .unwrap_or(0)
                    .max(runtime.zone.hi[0].map(|high| (value - high).max(0)).unwrap_or(0));
                if left_by >= runtime.zone.hy {
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
        let velocity = match velocity {
            Some(v) => format!(r#","v":[{v}]"#),
            None => String::new(),
        };
        self.send(&format!(
            r#"{},"seq":{seq},"t_us":{},"c":[{counts}]{velocity}"#,
            self.begin("sample"),
            self.t_us()
        ));
    }
}
