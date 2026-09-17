//! One line of the wire: the CRC, and the rules for finding it.
//!
//! `docs/reference/protocol.md` §1. The two rules that shape this module:
//!
//! - **`crc` is the last member**, so a receiver finds it by scanning backwards
//!   for `,"crc":"` rather than by parsing. One pass, no buffer — which matters
//!   on the device and costs the host nothing.
//! - **A line that fails its CRC is never acted on, not even partially.** It is
//!   answered with `error`/`bad_crc` and dropped. A half-applied zone upload is
//!   exactly the failure a CRC exists to prevent.

/// CRC-16/CCITT-FALSE: polynomial 0x1021, initial 0xFFFF, no reflection, no
/// final XOR. The same parameters statemachined's link uses, for the same
/// reason — it is what the device can compute as it writes.
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

/// What a line can be wrong in. Each one is answered by name, never silently.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FramingError {
    /// No `,"crc":"...."` before the closing brace.
    NoCrc,
    /// The four hex digits did not match the bytes before them.
    BadCrc { expected: u16, found: u16 },
    /// A byte ≥ 0x80. Non-ASCII text belongs in `log`, escaped.
    NonAscii,
    /// Longer than the device said it could hold.
    TooLong { limit: usize },
}

impl std::fmt::Display for FramingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FramingError::NoCrc => write!(f, "no crc member"),
            FramingError::BadCrc { expected, found } => {
                write!(f, "bad crc: computed {expected:04X}, line says {found:04X}")
            }
            FramingError::NonAscii => write!(f, "a byte above 0x7F"),
            FramingError::TooLong { limit } => write!(f, "longer than {limit} bytes"),
        }
    }
}

const CRC_MEMBER: &str = ",\"crc\":\"";

/// Append the CRC to a line body and terminate it.
///
/// `body` is the object without its closing brace — everything the CRC covers.
pub fn seal(body: &str) -> String {
    let crc = crc16(body.as_bytes());
    format!("{body}{CRC_MEMBER}{crc:04X}\"}}\n")
}

/// Check a received line and hand back the JSON object it carries.
///
/// The returned slice includes the `crc` member: it is a well-formed object and
/// a reader that ignores unknown members — which both sides must — reads it
/// unchanged. Stripping it would mean rebuilding the line.
pub fn check(line: &str, max_line: usize) -> Result<&str, FramingError> {
    let line = line.strip_suffix('\n').unwrap_or(line);
    let line = line.strip_suffix('\r').unwrap_or(line);
    if line.len() + 1 > max_line {
        return Err(FramingError::TooLong { limit: max_line });
    }
    if !line.is_ascii() {
        return Err(FramingError::NonAscii);
    }
    // Backwards, as the protocol promises: the last occurrence is the framing
    // one even if a string member happens to contain the same bytes.
    let at = line.rfind(CRC_MEMBER).ok_or(FramingError::NoCrc)?;
    let covered = &line[..at];
    let rest = &line[at + CRC_MEMBER.len()..];
    let digits = rest.get(..4).ok_or(FramingError::NoCrc)?;
    let found = u16::from_str_radix(digits, 16).map_err(|_| FramingError::NoCrc)?;
    let expected = crc16(covered.as_bytes());
    if expected != found {
        return Err(FramingError::BadCrc { expected, found });
    }
    Ok(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_crc_matches_the_reference_vector() {
        // "123456789" → 0x29B1 is CCITT-FALSE's check value, and the one line
        // in this module worth pinning to something outside this repository.
        assert_eq!(crc16(b"123456789"), 0x29B1);
    }

    #[test]
    fn a_sealed_line_checks_out() {
        let line = seal(r#"{"msg_type":"ping","message_id":41"#);
        assert!(line.ends_with("\"}\n"));
        assert!(check(&line, 512).is_ok());
    }

    #[test]
    fn a_flipped_byte_is_refused_rather_than_parsed() {
        let line = seal(r#"{"msg_type":"sample","seq":41822,"c":[173884]"#);
        let corrupted = line.replace("173884", "173885");
        match check(&corrupted, 512) {
            Err(FramingError::BadCrc { .. }) => {}
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_truncated_line_is_refused() {
        let line = seal(r#"{"msg_type":"ping","message_id":41"#);
        let truncated = &line[..line.len() / 2];
        assert!(check(truncated, 512).is_err());
    }

    #[test]
    fn a_line_over_the_limit_is_refused_by_length_not_by_crc() {
        let line = seal(&format!(r#"{{"msg_type":"log","text":"{}""#, "x".repeat(600)));
        assert_eq!(check(&line, 512), Err(FramingError::TooLong { limit: 512 }));
    }

    #[test]
    fn a_crc_inside_a_string_does_not_fool_the_scan() {
        // The backwards scan finds the framing member, not the decoy in `text`.
        let line = seal(r#"{"msg_type":"log","text":"got ,\"crc\":\"0000\""#);
        assert!(check(&line, 512).is_ok());
    }

    #[test]
    fn a_carriage_return_before_the_newline_is_ignored() {
        let line = seal(r#"{"msg_type":"pong","message_id":7"#);
        let with_cr = line.replace('\n', "\r\n");
        assert!(check(&with_cr, 512).is_ok());
    }
}
