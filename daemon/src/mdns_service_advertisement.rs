// SPDX-License-Identifier: AGPL-3.0-or-later
//! Telling the network this rig has a wheel: `_mousewheeld._tcp`.
//!
//! Beside `_vstimd._tcp`, `_statemachined._tcp` and `_triald._tcp`, so one
//! browse of the local domain finds every braemons daemon on a rig and the
//! console needs no `rigs.json` entry for it. statemachined's
//! `mdns_service_advertisement.rs` is the reference, and this follows it.
//!
//! Two identifiers, for two questions:
//!
//! - **`id`** — this daemon on this box, salted with the daemon's name so it
//!   is stable across renames and reboots and unique per daemon.
//! - **`rig`** — the box, salted with `braemons:` and so the *same* string in
//!   every braemons daemon's record on it. That is what lets a console show
//!   the wheel, the display and the state machine as one rig rather than
//!   guessing from a hostname (`console/docs/PLAN.md` §4).
//!
//! Both are hashes of `/etc/machine-id`, which is meant to be confidential.
//!
//! **Advertising is never fatal.** A daemon whose network is down still owns a
//! board and still serves an API somebody may reach by IP.

use std::collections::HashMap;
use std::path::Path;

use sha2::{Digest, Sha256};

pub const SERVICE_TYPE: &str = "_mousewheeld._tcp.local.";
pub const MACHINE_ID_PATH: &str = "/etc/machine-id";

/// Sixteen hex digits of `sha256(salt + machine-id)`.
///
/// Falls back to the hostname where there is no machine-id — a container, a
/// developer's checkout: weaker, and better than a value that changes every
/// restart.
pub fn salted_machine_identifier(salt: &str, machine_id_path: &Path) -> String {
    let seed = std::fs::read_to_string(machine_id_path)
        .map(|text| text.trim().to_string())
        .unwrap_or_default();
    let seed = if seed.is_empty() { hostname() } else { seed };
    let digest = Sha256::digest(format!("{salt}{seed}").as_bytes());
    digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()[..16]
        .to_string()
}

/// What a console can act on without opening a connection first.
pub fn text_records_for(
    daemon_identifier: &str,
    rig_identifier: &str,
    port: u16,
    version: &str,
) -> HashMap<String, String> {
    [
        ("id", daemon_identifier.to_string()),
        ("rig", rig_identifier.to_string()),
        ("version", version.to_string()),
        ("elements", "/elements/mousewheeld.js".to_string()),
        ("port", port.to_string()),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_string(), value))
    .collect()
}

/// This box's name, as somebody standing next to the rig would say it.
pub fn hostname() -> String {
    let mut buffer = [0u8; 256];
    // SAFETY: a buffer and its length, and gethostname writes within it.
    let written = unsafe { libc::gethostname(buffer.as_mut_ptr().cast(), buffer.len()) };
    if written != 0 {
        return "mousewheeld".into();
    }
    let end = buffer
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(buffer.len());
    String::from_utf8_lossy(&buffer[..end]).to_string()
}

fn short_hostname() -> String {
    hostname().split('.').next().unwrap_or_default().to_string()
}

/// One `_mousewheeld._tcp` registration, withdrawn when the daemon stops.
pub struct MdnsServiceAdvertisement {
    port: u16,
    version: String,
    daemon: Option<mdns_sd::ServiceDaemon>,
    fullname: Option<String>,
}

impl MdnsServiceAdvertisement {
    pub fn new(port: u16, version: &str) -> Self {
        Self {
            port,
            version: version.to_string(),
            daemon: None,
            fullname: None,
        }
    }

    /// The record, as the responder wants it. Separate so a test can read it.
    pub fn build_service_info(&self) -> Result<mdns_sd::ServiceInfo, String> {
        let machine_id = Path::new(MACHINE_ID_PATH);
        let records = text_records_for(
            &salted_machine_identifier("mousewheeld:", machine_id),
            &salted_machine_identifier("braemons:", machine_id),
            self.port,
            &self.version,
        );
        let host = short_hostname();
        mdns_sd::ServiceInfo::new(
            SERVICE_TYPE,
            &host,
            &format!("{host}.local."),
            "",
            self.port,
            records,
        )
        .map(|info| info.enable_addr_auto())
        .map_err(|problem| problem.to_string())
    }

    /// Register. Returns whether it worked; never fails the daemon.
    pub fn start(&mut self) -> bool {
        let registered = (|| {
            let info = self.build_service_info()?;
            let daemon = mdns_sd::ServiceDaemon::new().map_err(|problem| problem.to_string())?;
            let fullname = info.get_fullname().to_string();
            daemon
                .register(info)
                .map_err(|problem| problem.to_string())?;
            Ok::<_, String>((daemon, fullname))
        })();
        match registered {
            Ok((daemon, fullname)) => {
                log::info!("mDNS: advertising {fullname}");
                self.daemon = Some(daemon);
                self.fullname = Some(fullname);
                true
            }
            Err(problem) => {
                log::warn!(
                    "mDNS: not advertising ({problem}). The API is still served; a console \
                     will need this rig's address by hand."
                );
                false
            }
        }
    }

    /// Withdraw the record, then close. Never fails.
    pub fn stop(&mut self) {
        if let (Some(daemon), Some(fullname)) = (&self.daemon, &self.fullname) {
            let _ = daemon.unregister(fullname);
        }
        if let Some(daemon) = self.daemon.take() {
            let _ = daemon.shutdown();
        }
        self.fullname = None;
    }
}

impl Drop for MdnsServiceAdvertisement {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rig_identifier_is_the_same_in_every_daemon_and_the_daemon_one_is_not() {
        let directory =
            std::env::temp_dir().join(format!("mousewheeld-mdns-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let machine_id = directory.join("machine-id");
        std::fs::write(&machine_id, "dc4f4b06f2d84f6b9e2a7b0c1d2e3f40\n").unwrap();

        let rig = salted_machine_identifier("braemons:", &machine_id);
        let daemon = salted_machine_identifier("mousewheeld:", &machine_id);
        assert_eq!(rig.len(), 16);
        assert_ne!(rig, daemon);
        assert!(!rig.contains("dc4f4b06"));
        // statemachined's `id` for this machine-id, from its own test: proof
        // that the salting is the one the family uses.
        assert_eq!(
            salted_machine_identifier("statemachined:", &machine_id),
            "0b60208b6da842d7"
        );
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn the_text_records_say_where_the_elements_are_and_which_rig() {
        let records = text_records_for("abc", "def", 8083, "1.2.3");
        assert_eq!(records["elements"], "/elements/mousewheeld.js");
        assert_eq!(records["rig"], "def");
        assert_eq!(records["port"], "8083");
    }
}
