//! The daemon against the firmware itself.
//!
//! `mousewheeld_native_device` is `firmware/` built for this machine — the same
//! core, session and framing a board runs, with the link on stdin/stdout and a
//! wheel that turns at a set speed. Put on the far end of a pty, it is a board
//! as far as the daemon can tell, and this is the test that the two halves of
//! the link agree: nanopb against prost, the firmware's COBS and CRC against
//! the daemon's, the firmware's refusals against the daemon's expectations.
//!
//! It needs the binary, which CMake builds (`make test-firmware`). Without it
//! the test says so and passes, because `cargo test` must work on a checkout
//! with no C++ toolchain; with `MOUSEWHEELD_REQUIRE_NATIVE_DEVICE=1` a missing
//! binary fails instead, which is what CI sets.

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use mousewheeld::device::{AxisSetup, Backend, Device};
use mousewheeld::link::serial::{duplicate, pty_pair};
use mousewheeld::model::state::StreamFrame;
use mousewheeld::model::zone_set::{ArmOrigin, FireRule, ZoneMetric};
use mousewheeld::zones::{CompiledZone, CompiledZoneSet};

const COUNTS_PER_CM: f64 = 40.0;
/// 100 cm/s: a fast mouse, and a zone 20 cm out fires a fifth of a second
/// after the arm.
const SPEED_COUNTS_S: f64 = 4000.0;

struct Board(Child);

impl Drop for Board {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn native_device() -> Option<PathBuf> {
    let path = std::env::var_os("MOUSEWHEELD_NATIVE_DEVICE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../firmware/build/mousewheeld_native_device")
        });
    if path.exists() {
        return Some(path);
    }
    assert!(
        std::env::var_os("MOUSEWHEELD_REQUIRE_NATIVE_DEVICE").is_none(),
        "{} is missing: run `make test-firmware`",
        path.display()
    );
    eprintln!("skipped: {} is not built (`make test-firmware`)", path.display());
    None
}

/// The firmware on the board end of a pty, and the path the daemon opens.
fn spawn_board(program: &PathBuf) -> (Board, String) {
    let (board_side, host_path) = pty_pair().expect("a pty");
    let child = Command::new(program)
        .env("MOUSEWHEELD_NATIVE_SPEED", SPEED_COUNTS_S.to_string())
        .stdin(Stdio::from(duplicate(&board_side).unwrap()))
        .stdout(Stdio::from(board_side))
        .stderr(Stdio::inherit())
        .spawn()
        .expect("the native device starts");
    (Board(child), host_path.to_string_lossy().into_owned())
}

fn wait_for(what: &str, timeout: Duration, mut done: impl FnMut() -> bool) {
    let deadline = Instant::now() + timeout;
    while !done() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn the_daemon_and_the_firmware_agree_on_the_link() {
    let Some(program) = native_device() else { return };
    let (_board, path) = spawn_board(&program);

    let device = Device::new(
        Backend::Port { path: path.clone(), baud: 0 },
        vec![AxisSetup { name: "wheel".into(), counts_per_cm: COUNTS_PER_CM, invert: false }],
        vec![(0, 12, false)],
        500,
        None,
    );
    let mut frames = device.subscribe_frames();

    // The greeting, answered: the board identified itself and its capacities.
    wait_for("the board's hello_ack", Duration::from_secs(5), || device.connected());
    let info = device.info(path);
    assert_eq!(info.board, "native");
    assert_eq!(info.protocol_version, 1);
    assert_eq!(info.capacities.max_zones, 16);
    assert_eq!(info.capacities.scan_hz, 5000);

    // The stream the daemon asked for, moving the way the wheel is.
    let first = device.counts_of("wheel");
    wait_for("the wheel to move on the stream", Duration::from_secs(5), || {
        device.counts_of("wheel") > first.map(|c| c + 400)
    });
    assert!(device.info(String::new()).link.last_error.is_none(), "{:?}", device.info(String::new()).link);

    // A zone 20 cm out, uploaded and armed from here; the firmware evaluates it
    // and says where it fired.
    let set = CompiledZoneSet {
        name: "goal".into(),
        version: 7,
        zones: vec![CompiledZone {
            name: "goal".into(),
            axis_indices: vec![0],
            metric: ZoneMetric::Displacement,
            min_counts: vec![Some((20.0 * COUNTS_PER_CM) as i64)],
            max_counts: vec![None],
            wrap_counts: None,
            fire: FireRule::Once,
            hysteresis_counts: 0,
            level: false,
            line_index: 0,
        }],
        counts_per_cm: COUNTS_PER_CM,
    };
    let arm_id = device.arm(set, ArmOrigin::Current, None).expect("the board said armed");

    let deadline = Instant::now() + Duration::from_secs(5);
    let hit = loop {
        assert!(Instant::now() < deadline, "no zone_hit");
        match frames.try_recv() {
            Ok(StreamFrame::ZoneHit(hit)) => break hit,
            Ok(_) | Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => {}
            Err(_) => std::thread::sleep(Duration::from_millis(5)),
        }
    };
    assert_eq!(hit.zone, "goal");
    assert_eq!(hit.arm_id, arm_id);
    // Where it fired, in the host's frame: at the bound, give or take the
    // counts one 200 µs scan covers at this speed.
    assert!(
        (20.0..20.1).contains(&hit.position_cm),
        "fired at {} cm",
        hit.position_cm
    );
    assert!(device.info(String::new()).link.last_error.is_none(), "{:?}", device.info(String::new()).link);
}
