//! The messages themselves, both directions.
//!
//! The shapes are generated from `proto/mousewheeld/link/v1/link.proto` into
//! [`crate::wire::link`]; this module is the little that sits on top of them:
//! naming a message, sealing a command into a frame, decoding a frame into a
//! message, and a readable rendering for the wire monitor.
//!
//! Unknown fields are ignored — protobuf does that, and it is how this protocol
//! gains fields without a version bump. An unknown *message* is not: a
//! `DeviceMessage` whose `body` this daemon cannot name is refused by name
//! rather than dropped, because a firmware build newer than this daemon is
//! exactly when somebody needs to be told.

use prost::Message;

use super::framing::seal;
use crate::wire::link::{device_message, host_message, DeviceMessage, HostMessage};

pub use crate::wire::link::{
    Action, AxisConfig, AxisSource, Fire, Interval, Line, Metric, Origin, Sample, Zone,
};
pub use device_message::Body as FromDevice;

/// A frame that checked out but is not a message this daemon can act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeProblem {
    /// Not a `DeviceMessage` at all.
    Malformed(String),
    /// A `DeviceMessage` whose body this daemon cannot name.
    UnknownBody { message_id: u32 },
}

impl std::fmt::Display for DecodeProblem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeProblem::Malformed(problem) => write!(f, "not a device message: {problem}"),
            DecodeProblem::UnknownBody { message_id } => {
                write!(f, "message {message_id} carries a body this daemon does not know")
            }
        }
    }
}

/// A checked frame's message, as the board sent it.
pub fn decode(message: &[u8]) -> Result<(u32, FromDevice), DecodeProblem> {
    let decoded =
        DeviceMessage::decode(message).map_err(|e| DecodeProblem::Malformed(e.to_string()))?;
    match decoded.body {
        Some(body) => Ok((decoded.message_id, body)),
        None => Err(DecodeProblem::UnknownBody { message_id: decoded.message_id }),
    }
}

/// The board's name for a message, as protocol.md writes it.
pub fn device_message_name(body: &FromDevice) -> &'static str {
    match body {
        FromDevice::HelloAck(_) => "hello_ack",
        FromDevice::Sample(_) => "sample",
        FromDevice::ZoneHit(_) => "zone_hit",
        FromDevice::Armed(_) => "armed",
        FromDevice::StateReport(_) => "state_report",
        FromDevice::Ok(_) => "ok",
        FromDevice::Error(_) => "error",
        FromDevice::Log(_) => "log",
        FromDevice::Pong(_) => "pong",
    }
}

/// One frame as the wire monitor shows it.
///
/// The header is JSON-shaped on purpose: the monitor panel finds samples by
/// `"msg_type":"sample"`, and a reader of the monitor learnt the protocol by
/// those names. What follows is the message's own fields.
pub fn describe_device(message_id: u32, body: &FromDevice) -> String {
    let fields = match body {
        FromDevice::HelloAck(m) => format!("{m:?}"),
        FromDevice::Sample(m) => format!("{m:?}"),
        FromDevice::ZoneHit(m) => format!("{m:?}"),
        FromDevice::Armed(m) => format!("{m:?}"),
        FromDevice::StateReport(m) => format!("{m:?}"),
        FromDevice::Ok(m) => format!("{m:?}"),
        FromDevice::Error(m) => format!("{m:?}"),
        FromDevice::Log(m) => format!("{m:?}"),
        FromDevice::Pong(m) => format!("{m:?}"),
    };
    format!(
        r#"{{"msg_type":"{}","message_id":{message_id}}} {fields}"#,
        device_message_name(body)
    )
}

// ------------------------------------------------------- host → device ---

/// A command, ready to be sealed and written.
pub struct Command {
    pub message_id: u16,
    pub body: host_message::Body,
}

impl Command {
    pub fn name(&self) -> &'static str {
        use host_message::Body::*;
        match &self.body {
            Hello(_) => "hello",
            Axes(_) => "axes",
            Lines(_) => "lines",
            Stream(_) => "stream",
            ZonesBegin(_) => "zones_begin",
            Zone(_) => "zone",
            ZonesEnd(_) => "zones_end",
            Arm(_) => "arm",
            Disarm(_) => "disarm",
            Zero(_) => "zero",
            Analog(_) => "analog",
            Save(_) => "save",
            Ping(_) => "ping",
            State(_) => "state",
            Debug(_) => "debug",
        }
    }

    /// The sealed frame, delimiter included.
    pub fn encode(&self) -> Vec<u8> {
        let message = HostMessage {
            message_id: u32::from(self.message_id),
            body: Some(self.body.clone()),
        };
        seal(&message.encode_to_vec())
    }

    /// As the wire monitor shows it; see [`describe_device`].
    pub fn describe(&self) -> String {
        use host_message::Body::*;
        let fields = match &self.body {
            Hello(m) => format!("{m:?}"),
            Axes(m) => format!("{m:?}"),
            Lines(m) => format!("{m:?}"),
            Stream(m) => format!("{m:?}"),
            ZonesBegin(m) => format!("{m:?}"),
            Zone(m) => format!("{m:?}"),
            ZonesEnd(m) => format!("{m:?}"),
            Arm(m) => format!("{m:?}"),
            Disarm(m) => format!("{m:?}"),
            Zero(m) => format!("{m:?}"),
            Analog(m) => format!("{m:?}"),
            Save(m) => format!("{m:?}"),
            Ping(m) => format!("{m:?}"),
            State(m) => format!("{m:?}"),
            Debug(m) => format!("{m:?}"),
        };
        format!(
            r#"{{"msg_type":"{}","message_id":{}}} {fields}"#,
            self.name(),
            self.message_id
        )
    }
}

/// The commands this daemon sends. Named functions rather than struct literals
/// at every call site: each is a message with its own fields, and the shape is
/// the protocol's.
pub mod commands {
    use super::{AxisConfig, AxisSource, Command, Line, Origin, Zone};
    use crate::wire::link::{self, host_message::Body};

    fn command(message_id: u16, body: Body) -> Command {
        Command { message_id, body }
    }

    pub fn hello(id: u16, protocol_version: u32) -> Command {
        command(id, Body::Hello(link::Hello { protocol_version }))
    }

    pub fn ping(id: u16) -> Command {
        command(id, Body::Ping(link::Ping {}))
    }

    pub fn state(id: u16) -> Command {
        command(id, Body::State(link::StateRequest {}))
    }

    pub fn stream(id: u16, rate_hz: u32, velocity: bool) -> Command {
        command(id, Body::Stream(link::Stream { rate_hz, velocity }))
    }

    /// Every axis as quadrature and none inverted: the daemon applies
    /// inversion itself, once, so nothing above the link carries a sign — and a
    /// board that inverted too would cancel it.
    pub fn axes(id: u16, n: usize) -> Command {
        let axes = (0..n)
            .map(|_| AxisConfig { source: AxisSource::Quadrature as i32, invert: false })
            .collect();
        command(id, Body::Axes(link::Axes { axes }))
    }

    pub fn lines(id: u16, lines: &[(u8, u8, bool)]) -> Command {
        let lines = lines
            .iter()
            .map(|(index, pin, safe_high)| Line {
                index: u32::from(*index),
                pin: u32::from(*pin),
                safe_high: *safe_high,
            })
            .collect();
        command(id, Body::Lines(link::Lines { lines }))
    }

    /// The name travels with the set because the board reports what it has
    /// flashed *by name* in `hello_ack`, and a board that was only ever told a
    /// version cannot do that.
    pub fn zones_begin(id: u16, version: u32, n: usize, name: &str) -> Command {
        command(
            id,
            Body::ZonesBegin(link::ZonesBegin {
                zone_set_version: version,
                n: n as u32,
                name: name.to_string(),
            }),
        )
    }

    pub fn zone(id: u16, zone: Zone) -> Command {
        command(id, Body::Zone(zone))
    }

    /// The board refuses the commit unless every index in `0..n` arrived.
    pub fn zones_end(id: u16) -> Command {
        command(id, Body::ZonesEnd(link::ZonesEnd {}))
    }

    pub fn arm(id: u16, arm_id: u32, version: u32, origin_is_current: bool) -> Command {
        let origin = if origin_is_current { Origin::Current } else { Origin::Absolute };
        command(
            id,
            Body::Arm(link::Arm { arm_id, zone_set_version: version, origin: origin as i32 }),
        )
    }

    pub fn disarm(id: u16, arm_id: u32) -> Command {
        command(id, Body::Disarm(link::Disarm { arm_id }))
    }

    pub fn zero(id: u16, axes: &[usize]) -> Command {
        let axes = axes.iter().map(|axis| *axis as u32).collect();
        command(id, Body::Zero(link::Zero { axes }))
    }

    pub fn save(id: u16) -> Command {
        command(id, Body::Save(link::Save {}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::link::framing::check;
    use crate::wire::link::HelloAck;

    fn unframe(frame: &[u8]) -> Vec<u8> {
        check(&frame[..frame.len() - 1], 256).unwrap()
    }

    #[test]
    fn a_command_is_one_sealed_frame_that_decodes_to_itself() {
        let frame = commands::stream(41, 500, true).encode();
        let message = HostMessage::decode(unframe(&frame).as_slice()).unwrap();
        assert_eq!(message.message_id, 41);
        assert_eq!(
            message.body,
            Some(host_message::Body::Stream(crate::wire::link::Stream {
                rate_hz: 500,
                velocity: true
            }))
        );
    }

    #[test]
    fn a_device_message_decodes_into_its_variant() {
        let sent = DeviceMessage {
            message_id: 903,
            body: Some(FromDevice::Sample(Sample {
                seq: 41822,
                t_us: 8_391_204,
                c: vec![173_884],
                v: vec![],
            })),
        };
        let (id, body) = decode(&sent.encode_to_vec()).unwrap();
        assert_eq!(id, 903);
        match body {
            FromDevice::Sample(sample) => {
                assert_eq!(sample.seq, 41822);
                assert_eq!(sample.c, vec![173_884]);
                assert!(sample.v.is_empty(), "velocity is only there when asked for");
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn unknown_fields_are_ignored_so_a_newer_firmware_still_parses() {
        let mut bytes = DeviceMessage {
            message_id: 1,
            body: Some(FromDevice::HelloAck(HelloAck {
                board: "teensy41".into(),
                firmware: "1.2.3".into(),
                n_axes: 2,
                ..Default::default()
            })),
        }
        .encode_to_vec();
        // Field 99, varint 7: tomorrow's field on today's message.
        bytes.extend_from_slice(&[0x98, 0x06, 0x07]);
        assert!(matches!(decode(&bytes), Ok((1, FromDevice::HelloAck(_)))));
    }

    #[test]
    fn an_unknown_message_is_refused_by_name_rather_than_dropped() {
        // message_id 5, and a body at field 99 this daemon has never heard of.
        let bytes = [0x08, 0x05, 0x9A, 0x06, 0x00];
        assert_eq!(decode(&bytes), Err(DecodeProblem::UnknownBody { message_id: 5 }));
    }

    #[test]
    fn a_zone_goes_out_as_counts_with_an_open_bound_absent() {
        let zone = Zone {
            index: 0,
            intervals: vec![Interval { axis: 0, lo: Some(17_384), hi: None }],
            hysteresis: 174,
            pulse_ms: 10,
            ..Default::default()
        };
        let frame = commands::zone(18, zone.clone()).encode();
        let message = HostMessage::decode(unframe(&frame).as_slice()).unwrap();
        assert_eq!(message.body, Some(host_message::Body::Zone(zone)));
    }

    #[test]
    fn the_monitor_still_finds_samples_by_name() {
        let text = describe_device(
            3,
            &FromDevice::Sample(Sample { seq: 1, t_us: 2, c: vec![3], v: vec![] }),
        );
        assert!(text.contains(r#""msg_type":"sample""#), "{text}");
    }
}
