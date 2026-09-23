//! One frame of the wire: COBS, the CRC, and the rules for checking them.
//!
//! `docs/reference/protocol.md` §1. Both directions, identically:
//!
//! ```text
//! COBS( protobuf message ‖ CRC-16 big-endian ) ‖ 0x00
//! ```
//!
//! The two rules that shape this module:
//!
//! - **A zero byte ends a frame and appears nowhere else.** COBS guarantees it,
//!   so a receiver that lost bytes — a USB re-enumeration, a board that reset
//!   mid-frame — finds the next frame by waiting for the next zero, without
//!   parsing anything.
//! - **A frame that fails its CRC is never acted on, not even partially.** It
//!   is logged and dropped. A half-applied zone upload is exactly the failure a
//!   CRC exists to prevent.

/// CRC-16/CCITT-FALSE: polynomial 0x1021, initial 0xFFFF, no reflection, no
/// final XOR. The same parameters as the firmware's
/// `firmware/core/protocol/crc16.cpp`.
pub fn crc16(bytes: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for byte in bytes {
        crc ^= u16::from(*byte) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 {
                (crc << 1) ^ 0x1021
            } else {
                crc << 1
            };
        }
    }
    crc
}

/// What a frame can be wrong in. Each one is reported by name, never silently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FramingError {
    /// Not valid COBS: a code byte that runs past the end of the frame.
    BadCobs,
    /// Shorter than the two bytes of CRC every frame carries.
    TooShort,
    /// The CRC did not match the bytes before it.
    BadCrc { expected: u16, found: u16 },
    /// Longer than the board said it could hold.
    TooLong { limit: usize },
}

impl std::fmt::Display for FramingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FramingError::BadCobs => write!(f, "not a COBS frame"),
            FramingError::TooShort => write!(f, "a frame too short to carry a crc"),
            FramingError::BadCrc { expected, found } => {
                write!(f, "bad crc: computed {expected:04X}, frame says {found:04X}")
            }
            FramingError::TooLong { limit } => write!(f, "longer than {limit} bytes"),
        }
    }
}

/// Seal an encoded message into a frame, delimiter included.
pub fn seal(message: &[u8]) -> Vec<u8> {
    let mut sealed = Vec::with_capacity(message.len() + 2);
    sealed.extend_from_slice(message);
    sealed.extend_from_slice(&crc16(message).to_be_bytes());
    let mut frame = cobs_encode(&sealed);
    frame.push(0);
    frame
}

/// Check one frame's bytes — without its delimiter — and hand back the message
/// it carries.
pub fn check(frame: &[u8], max_frame: usize) -> Result<Vec<u8>, FramingError> {
    // The limit counts the delimiter, as the board's does.
    if frame.len() + 1 > max_frame {
        return Err(FramingError::TooLong { limit: max_frame });
    }
    let mut decoded = cobs_decode(frame).ok_or(FramingError::BadCobs)?;
    if decoded.len() < 2 {
        return Err(FramingError::TooShort);
    }
    let at = decoded.len() - 2;
    let found = u16::from_be_bytes([decoded[at], decoded[at + 1]]);
    decoded.truncate(at);
    let expected = crc16(&decoded);
    if expected != found {
        return Err(FramingError::BadCrc { expected, found });
    }
    Ok(decoded)
}

fn cobs_encode(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len() + input.len() / 254 + 2);
    let mut code_at = 0;
    out.push(0);
    let mut code: u8 = 1;
    for byte in input {
        if *byte == 0 {
            out[code_at] = code;
            code_at = out.len();
            out.push(0);
            code = 1;
            continue;
        }
        out.push(*byte);
        code += 1;
        if code == 0xFF {
            out[code_at] = code;
            code_at = out.len();
            out.push(0);
            code = 1;
        }
    }
    out[code_at] = code;
    out
}

fn cobs_decode(input: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(input.len());
    let mut read = 0;
    while read < input.len() {
        let code = input[read];
        read += 1;
        if code == 0 {
            return None;
        }
        for _ in 1..code {
            let byte = *input.get(read)?;
            if byte == 0 {
                return None;
            }
            out.push(byte);
            read += 1;
        }
        // A zero was elided after every group but a full one and the last.
        if code != 0xFF && read < input.len() {
            out.push(0);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(frame: &[u8]) -> &[u8] {
        assert_eq!(frame.last(), Some(&0), "a frame ends with its delimiter");
        &frame[..frame.len() - 1]
    }

    #[test]
    fn the_crc_matches_the_reference_vector() {
        // "123456789" → 0x29B1 is CCITT-FALSE's check value, and the one line
        // in this module worth pinning to something outside this repository.
        assert_eq!(crc16(b"123456789"), 0x29B1);
    }

    #[test]
    fn a_sealed_frame_checks_out_and_has_no_zero_inside() {
        let message = [0x08, 0x00, 0x2A, 0x00, 0xFF];
        let frame = seal(&message);
        assert!(!body(&frame).contains(&0));
        assert_eq!(check(body(&frame), 256).unwrap(), message);
    }

    #[test]
    fn cobs_round_trips_across_the_254_byte_boundary() {
        for len in [0usize, 1, 253, 254, 255, 508, 600] {
            let mut run = vec![0x42u8; len];
            if len > 2 {
                run[len / 2] = 0;
            }
            let encoded = cobs_encode(&run);
            assert!(!encoded.contains(&0), "len {len}");
            assert_eq!(cobs_decode(&encoded).unwrap(), run, "len {len}");
        }
    }

    #[test]
    fn a_flipped_byte_is_refused_rather_than_parsed() {
        let mut frame = seal(&[1, 2, 3, 4, 5, 6]);
        frame[3] ^= 0x01;
        match check(body(&frame), 256) {
            Err(FramingError::BadCrc { .. }) => {}
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_truncated_frame_is_refused() {
        let frame = seal(&[1, 2, 3, 4, 5, 6, 7, 8]);
        assert!(check(&frame[..frame.len() / 2], 256).is_err());
    }

    #[test]
    fn a_frame_over_the_limit_is_refused_by_length_not_by_crc() {
        let frame = seal(&[7u8; 300]);
        assert_eq!(check(body(&frame), 256), Err(FramingError::TooLong { limit: 256 }));
    }

    /// The firmware's own bytes for `DeviceMessage{hello_ack{board: "native",
    /// …}}`, as `mousewheeld_native_device` wrote them at boot. A frame sealed
    /// by one side and checked by the other is the only test that the two
    /// implementations of COBS and the CRC agree.
    #[test]
    fn a_frame_the_firmware_wrote_checks_out() {
        let from_firmware: &[u8] = &[
            0x22, 0x52, 0x1d, 0x0a, 0x06, 0x6e, 0x61, 0x74, 0x69, 0x76, 0x65, 0x12, 0x05, 0x30,
            0x2e, 0x30, 0x2e, 0x30, 0x18, 0x01, 0x20, 0x02, 0x28, 0x10, 0x30, 0x08, 0x38, 0x80,
            0x02, 0x40, 0x88, 0x27, 0xc6, 0xae,
        ];
        let message = check(from_firmware, 256).unwrap();
        assert_eq!(&message[..3], &[0x52, 0x1d, 0x0a], "field 10 (hello_ack), 29 bytes");
    }
}
