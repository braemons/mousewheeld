//! The file descriptor under the protocol: a real serial port, or a pty pair.
//!
//! **Why a pty and not a mock object.** The board simulator could have been a
//! function the daemon calls; then the framing, the CRC, the line splitting,
//! the partial read and the reader thread — every part of this link that can
//! actually be wrong — would be exercised by nothing. On a pty, the daemon runs
//! the code it will run against a Teensy: bytes in, lines out, and a device on
//! the far end that can be slow, wrong, or gone.
//!
//! Raw mode is not optional. A tty in canonical mode buffers by line, echoes
//! what it receives and translates `\r`; on a link whose messages are
//! newline-delimited that is three different ways to corrupt a sample.

use std::ffi::CStr;
use std::io::{self, Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::path::PathBuf;

/// Open a serial port and put it in raw mode.
pub fn open_port(path: &str, baud: u32) -> io::Result<OwnedFd> {
    let c_path = std::ffi::CString::new(path)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "a NUL in the port name"))?;
    // SAFETY: a valid C string; O_NOCTTY keeps this process from acquiring the
    // port as its controlling terminal, which would route signals through it.
    let fd = unsafe { libc::open(c_path.as_ptr(), libc::O_RDWR | libc::O_NOCTTY) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `open` returned an owned descriptor.
    let fd = unsafe { OwnedFd::from_raw_fd(fd) };
    make_raw(&fd, baud)?;
    Ok(fd)
}

/// A pty pair: the descriptor to hand the fake board, and the path the daemon
/// opens as though it were a serial port.
pub fn pty_pair() -> io::Result<(OwnedFd, PathBuf)> {
    // SAFETY: no arguments to get wrong.
    let master = unsafe { libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY) };
    if master < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `master` is an open pty multiplexer descriptor.
    let master = unsafe { OwnedFd::from_raw_fd(master) };
    // SAFETY: both take the master descriptor and touch nothing else.
    if unsafe { libc::grantpt(master.as_raw_fd()) } < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: as above.
    if unsafe { libc::unlockpt(master.as_raw_fd()) } < 0 {
        return Err(io::Error::last_os_error());
    }
    let mut name = [0i8; 128];
    // SAFETY: `name` is a buffer of exactly the length passed.
    if unsafe { libc::ptsname_r(master.as_raw_fd(), name.as_mut_ptr(), name.len()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `ptsname_r` NUL-terminated the buffer on success.
    let slave = unsafe { CStr::from_ptr(name.as_ptr()) }
        .to_str()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "a pty name that is not UTF-8"))?
        .to_string();
    make_raw(&master, 0)?;
    Ok((master, PathBuf::from(slave)))
}

/// Raw mode, and a read that blocks until at least one byte arrives.
///
/// `VMIN = 1, VTIME = 0` rather than a timeout: this descriptor is read by a
/// thread whose only job is to read it, and a timeout would turn a blocking
/// wait into a spin.
fn make_raw(fd: &OwnedFd, baud: u32) -> io::Result<()> {
    // SAFETY: zeroed termios is a valid starting point for tcgetattr to fill.
    let mut settings: libc::termios = unsafe { std::mem::zeroed() };
    // SAFETY: valid descriptor and out-pointer.
    if unsafe { libc::tcgetattr(fd.as_raw_fd(), &mut settings) } < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `settings` was filled by tcgetattr.
    unsafe { libc::cfmakeraw(&mut settings) };
    settings.c_cc[libc::VMIN] = 1;
    settings.c_cc[libc::VTIME] = 0;
    if let Some(speed) = termios_speed(baud) {
        // SAFETY: both take the settings we own and a speed constant.
        unsafe {
            libc::cfsetispeed(&mut settings, speed);
            libc::cfsetospeed(&mut settings, speed);
        }
    }
    // SAFETY: valid descriptor and settings.
    if unsafe { libc::tcsetattr(fd.as_raw_fd(), libc::TCSANOW, &settings) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Baud matters on a UART bridge (the ESP32) and is meaningless on native USB
/// (the Teensy), which is why an unknown rate is not an error: the board that
/// ignores it is the board this daemon prefers.
fn termios_speed(baud: u32) -> Option<libc::speed_t> {
    Some(match baud {
        0 => return None,
        9600 => libc::B9600,
        115_200 => libc::B115200,
        230_400 => libc::B230400,
        460_800 => libc::B460800,
        500_000 => libc::B500000,
        921_600 => libc::B921600,
        1_000_000 => libc::B1000000,
        2_000_000 => libc::B2000000,
        _ => return None,
    })
}

/// Reads whole lines off a descriptor, keeping what is left of a partial one.
///
/// A serial read returns whatever bytes have arrived, which is regularly half a
/// line — the single most common way a hand-written link corrupts its first
/// message under load.
pub struct LineReader {
    file: std::fs::File,
    buffer: Vec<u8>,
    /// Bytes read so far for the line being assembled.
    pending: Vec<u8>,
    max_line: usize,
    /// Set while a too-long line is being skipped to its terminator, so its tail
    /// is not parsed as a line of its own.
    discarding: bool,
}

impl LineReader {
    pub fn new(fd: OwnedFd, max_line: usize) -> Self {
        Self {
            file: std::fs::File::from(fd),
            buffer: vec![0; 4096],
            pending: Vec::with_capacity(max_line),
            max_line,
            discarding: false,
        }
    }

    /// Block until at least one line is complete, and return all of them.
    ///
    /// Returns an empty vector only at end of file — the board went away.
    pub fn read_lines(&mut self) -> io::Result<Vec<String>> {
        loop {
            let read = self.file.read(&mut self.buffer)?;
            if read == 0 {
                return Ok(Vec::new());
            }
            let mut lines = Vec::new();
            for byte in &self.buffer[..read] {
                if *byte == b'\n' {
                    if !self.discarding {
                        lines.push(String::from_utf8_lossy(&self.pending).into_owned());
                    }
                    self.pending.clear();
                    self.discarding = false;
                    continue;
                }
                if self.pending.len() + 1 >= self.max_line {
                    // Over the limit: drop it and everything up to the next
                    // newline. Half an object is never handed upwards.
                    self.pending.clear();
                    self.discarding = true;
                    continue;
                }
                self.pending.push(*byte);
            }
            if !lines.is_empty() {
                return Ok(lines);
            }
        }
    }
}

/// The writing half. Separate from the reader because they are used from
/// different threads: one blocks on `read`, the other sends commands.
pub struct LineWriter {
    file: std::fs::File,
}

impl LineWriter {
    pub fn new(fd: OwnedFd) -> Self {
        Self {
            file: std::fs::File::from(fd),
        }
    }

    pub fn write_line(&mut self, line: &str) -> io::Result<()> {
        self.file.write_all(line.as_bytes())?;
        self.file.flush()
    }
}

/// Duplicate a descriptor, so a reader and a writer can own one each.
pub fn duplicate(fd: &OwnedFd) -> io::Result<OwnedFd> {
    fd.try_clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pty_carries_whole_lines_through_partial_reads() {
        let (master, slave_path) = pty_pair().expect("a pty");
        let slave = open_port(slave_path.to_str().unwrap(), 0).expect("the slave side");
        let mut reader = LineReader::new(slave, 512);

        let mut board = std::fs::File::from(duplicate(&master).unwrap());
        // Deliberately split across writes, the way a serial port delivers it.
        board.write_all(b"{\"msg_type\":\"pon").unwrap();
        board.write_all(b"g\"}\n{\"msg_type\":\"log\"}\n").unwrap();
        board.flush().unwrap();

        let lines = reader.read_lines().unwrap();
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert_eq!(lines[0], r#"{"msg_type":"pong"}"#);
        assert_eq!(lines[1], r#"{"msg_type":"log"}"#);
    }

    #[test]
    fn an_over_long_line_is_discarded_whole_and_the_next_one_survives() {
        let (master, slave_path) = pty_pair().expect("a pty");
        let slave = open_port(slave_path.to_str().unwrap(), 0).expect("the slave side");
        let mut reader = LineReader::new(slave, 64);

        let mut board = std::fs::File::from(duplicate(&master).unwrap());
        board.write_all("x".repeat(200).as_bytes()).unwrap();
        board.write_all(b"\n{\"msg_type\":\"pong\"}\n").unwrap();
        board.flush().unwrap();

        let lines = reader.read_lines().unwrap();
        assert_eq!(lines, vec![r#"{"msg_type":"pong"}"#.to_string()]);
    }
}
