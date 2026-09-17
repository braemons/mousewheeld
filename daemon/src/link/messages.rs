//! The messages themselves, both directions.
//!
//! Typed rather than hand-parsed, with one exception noted below. Unknown
//! members are ignored — that is how this protocol gains fields without a
//! version bump — so nothing here denies them, which is the opposite of the
//! rule on the HTTP surface and deliberate: an HTTP body comes from a person
//! who can be told they made a typo, and a wire line comes from a firmware
//! build that may be older than this daemon.

use serde::{Deserialize, Serialize};

use super::framing::seal;

/// Device → host. `msg_type` picks the variant; an unknown one is refused by
/// name rather than dropped.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "msg_type", rename_all = "snake_case")]
pub enum FromDevice {
    HelloAck(HelloAck),
    Sample(WireSample),
    ZoneHit(WireZoneHit),
    Armed(Armed),
    StateReport(StateReport),
    Ok(Acknowledgement),
    Error(DeviceError),
    Log(LogLine),
    Pong(Acknowledgement),
}

#[derive(Debug, Clone, Deserialize)]
pub struct HelloAck {
    pub board: String,
    pub firmware: String,
    #[serde(default)]
    pub protocol_version: u32,
    #[serde(default)]
    pub n_axes: u8,
    #[serde(default)]
    pub max_zones: u16,
    #[serde(default)]
    pub max_lines: u8,
    /// The device's own line-length limit. The host holds it rather than
    /// assuming one, because a smaller board is a real possibility and a line
    /// silently truncated is the failure the CRC exists to catch late.
    #[serde(default = "default_max_line")]
    pub max_line: usize,
    #[serde(default)]
    pub scan_hz: u32,
    /// The set in the board's flash, if it came up with one.
    #[serde(default)]
    pub flashed: Option<FlashedSet>,
}

fn default_max_line() -> usize {
    512
}

#[derive(Debug, Clone, Deserialize)]
pub struct FlashedSet {
    pub name: String,
    pub version: u32,
}

/// The only high-rate message, and the reason the wire is shaped the way it is.
///
/// Members are short because this line is written by the device thousands of
/// times a second, by a fixed-format writer with no serializer behind it.
/// **Counts are cumulative**: the host differences successive totals, so a lost
/// line costs resolution and never distance.
#[derive(Debug, Clone, Deserialize)]
pub struct WireSample {
    /// Contiguous by construction. A jump is loss, and the host counts it.
    pub seq: u64,
    /// The device clock, free-running from boot.
    pub t_us: u64,
    /// Counts per axis.
    pub c: Vec<i64>,
    /// The device's own velocity, counts per second, when it was asked for it.
    #[serde(default)]
    pub v: Option<Vec<i64>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WireZoneHit {
    pub seq: u64,
    pub arm_id: u32,
    /// The zone's index in the armed set. The board holds no names.
    pub zone: usize,
    pub t_us: u64,
    pub c: Vec<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Armed {
    pub arm_id: u32,
    pub zone_set_version: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StateReport {
    #[serde(default)]
    pub c: Vec<i64>,
    #[serde(default)]
    pub origin: Vec<i64>,
    #[serde(default)]
    pub ring_drops: u64,
    #[serde(default)]
    pub scan_overruns: u64,
}

/// `ok` and `pong`. **`answers`, not `message_id`**: every line has its own
/// `message_id`, so a reply that reused the name would carry the member twice.
#[derive(Debug, Clone, Deserialize)]
pub struct Acknowledgement {
    #[serde(default)]
    pub answers: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceError {
    pub code: String,
    #[serde(default)]
    pub detail: String,
    /// The command this refuses, where the device could read one.
    #[serde(default)]
    pub answers: Option<u16>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LogLine {
    pub text: String,
}

// ------------------------------------------------------- host → device ---

/// One compiled zone, as the device receives it: **integer counts, no names,
/// no units**. The daemon compiled these against a named calibration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireZone {
    pub i: usize,
    pub ax: Vec<usize>,
    /// 0 displacement, 1 distance.
    pub m: u8,
    pub lo: Vec<Option<i64>>,
    pub hi: Vec<Option<i64>>,
    /// Period in counts; 0 for a straight track.
    pub wrap: i64,
    /// 0 once, 1 rearm.
    pub fire: u8,
    pub hy: i64,
    pub lvl: bool,
    pub line: u8,
    /// 0 pulse, 1 level.
    pub act: u8,
    pub ms: u32,
}

/// A command, ready to be sealed and written.
///
/// Built as a string rather than through a serializer because `crc` must be the
/// final member and nothing else may follow it — a guarantee a struct's field
/// order does not give, and one the receiver's backwards scan depends on.
pub struct Command {
    pub msg_type: &'static str,
    pub message_id: u16,
    body: String,
}

impl Command {
    /// `members` is the JSON for this command's own fields, without braces —
    /// `"rate_hz":500,"velocity":true` — or empty.
    pub fn new(msg_type: &'static str, message_id: u16, members: String) -> Self {
        Self {
            msg_type,
            message_id,
            body: members,
        }
    }

    pub fn encode(&self) -> String {
        let mut line = format!(
            r#"{{"msg_type":"{}","message_id":{}"#,
            self.msg_type, self.message_id
        );
        if !self.body.is_empty() {
            line.push(',');
            line.push_str(&self.body);
        }
        seal(&line)
    }
}

/// The commands this daemon sends. Named functions rather than an enum: each
/// one is a line with its own members, and the shape is the protocol's.
pub mod commands {
    use super::{Command, WireZone};

    pub fn hello(id: u16, protocol_version: u32) -> Command {
        Command::new("hello", id, format!(r#""protocol_version":{protocol_version}"#))
    }

    pub fn ping(id: u16) -> Command {
        Command::new("ping", id, String::new())
    }

    pub fn state(id: u16) -> Command {
        Command::new("state", id, String::new())
    }

    pub fn stream(id: u16, rate_hz: u32, velocity: bool) -> Command {
        Command::new(
            "stream",
            id,
            format!(r#""rate_hz":{rate_hz},"velocity":{velocity}"#),
        )
    }

    pub fn lines(id: u16, lines: &[(u8, u8, bool)]) -> Command {
        let members = lines
            .iter()
            .map(|(index, pin, safe_high)| {
                format!(r#"{{"index":{index},"pin":{pin},"safe_high":{safe_high}}}"#)
            })
            .collect::<Vec<_>>()
            .join(",");
        Command::new("lines", id, format!(r#""lines":[{members}]"#))
    }

    /// The name travels with the set because the board reports what it has
    /// flashed *by name* in `hello_ack`, and a board that was only ever told a
    /// version cannot do that.
    pub fn zones_begin(id: u16, version: u32, n: usize, name: &str) -> Command {
        Command::new(
            "zones_begin",
            id,
            format!(r#""zone_set_version":{version},"n":{n},"name":"{name}""#),
        )
    }

    pub fn zone(id: u16, zone: &WireZone) -> Command {
        // The one place a serializer touches an outgoing line, and it is safe:
        // the members it writes go inside the object, before the crc.
        let members = serde_json::to_string(zone).unwrap_or_default();
        let members = members.trim_start_matches('{').trim_end_matches('}').to_string();
        Command::new("zone", id, members)
    }

    pub fn zones_end(id: u16, checksum: u16) -> Command {
        Command::new("zones_end", id, format!(r#""checksum":{checksum}"#))
    }

    pub fn arm(id: u16, arm_id: u32, version: u32, origin_is_current: bool) -> Command {
        Command::new(
            "arm",
            id,
            format!(
                r#""arm_id":{arm_id},"zone_set_version":{version},"origin":"{}""#,
                if origin_is_current { "current" } else { "absolute" }
            ),
        )
    }

    pub fn disarm(id: u16, arm_id: u32) -> Command {
        Command::new("disarm", id, format!(r#""arm_id":{arm_id}"#))
    }

    pub fn zero(id: u16, axes: &[usize]) -> Command {
        let list = axes.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(",");
        Command::new("zero", id, format!(r#""axes":[{list}]"#))
    }

    pub fn save(id: u16) -> Command {
        Command::new("save", id, String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::link::framing::check;

    #[test]
    fn a_command_is_one_sealed_line_with_the_crc_last() {
        let line = commands::stream(41, 500, true).encode();
        assert_eq!(
            line,
            seal(r#"{"msg_type":"stream","message_id":41,"rate_hz":500,"velocity":true"#)
        );
        let checked = check(&line, 512).unwrap();
        assert!(checked.ends_with('}'));
    }

    #[test]
    fn a_device_line_round_trips_into_its_variant() {
        let line = seal(r#"{"msg_type":"sample","message_id":903,"seq":41822,"t_us":8391204,"c":[173884]"#);
        let body = check(&line, 512).unwrap();
        let message: FromDevice = serde_json::from_str(body).unwrap();
        match message {
            FromDevice::Sample(sample) => {
                assert_eq!(sample.seq, 41822);
                assert_eq!(sample.c, vec![173_884]);
                assert!(sample.v.is_none(), "velocity is optional");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn unknown_members_are_ignored_so_a_newer_firmware_still_parses() {
        let line = seal(
            r#"{"msg_type":"hello_ack","board":"teensy41","firmware":"1.2.3","n_axes":2,"tomorrows_field":7"#,
        );
        let body = check(&line, 512).unwrap();
        let message: FromDevice = serde_json::from_str(body).unwrap();
        assert!(matches!(message, FromDevice::HelloAck(_)));
    }

    #[test]
    fn an_unknown_message_type_is_an_error_rather_than_a_silent_drop() {
        let line = seal(r#"{"msg_type":"tomorrow","message_id":1"#);
        let body = check(&line, 512).unwrap();
        assert!(serde_json::from_str::<FromDevice>(body).is_err());
    }

    #[test]
    fn a_zone_goes_out_as_counts() {
        let zone = WireZone {
            i: 0,
            ax: vec![0],
            m: 0,
            lo: vec![Some(17_384)],
            hi: vec![None],
            wrap: 0,
            fire: 0,
            hy: 174,
            lvl: false,
            line: 0,
            act: 0,
            ms: 10,
        };
        let line = commands::zone(18, &zone).encode();
        assert!(line.contains(r#""lo":[17384]"#), "{line}");
        assert!(line.contains(r#""hi":[null]"#), "{line}");
        assert!(check(&line, 512).is_ok());
    }
}
