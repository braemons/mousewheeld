//! The rig config, and the part of it the API may change.
//!
//! **Two kinds of setting, and the split is deliberate.** The wiring — which
//! pin is `zone_goal`, which port the board is on — is changed where wiring is
//! described, in the file, and is not writable over the API. The rates and the
//! calibration are changed per session, by a person at a console, and are.
//!
//! The file is TOML at `/etc/braemons/mousewheeld-rig-config.toml`, beside
//! vstimd's. It is the runtime shape: the daemon rewrites it from these types
//! when a calibration is applied, so there is no second copy to drift.

use std::path::Path;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::calibration::{AxisCalibration, Calibration};
use super::line_map::OutputLine;

/// The whole file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RigConfig {
    #[serde(default)]
    pub device: DeviceConfig,
    #[serde(default)]
    pub stream: StreamConfig,
    #[serde(default)]
    pub publish: PublishConfig,
    /// The calibration, per axis. `[[axis]]` in the file.
    #[serde(default, rename = "axis")]
    pub axes: Vec<AxisCalibration>,
    /// The output line map. `[[line]]` in the file.
    #[serde(default, rename = "line")]
    pub lines: Vec<OutputLine>,
}

impl Default for RigConfig {
    fn default() -> Self {
        Self {
            device: DeviceConfig::default(),
            stream: StreamConfig::default(),
            publish: PublishConfig::default(),
            axes: vec![AxisCalibration {
                name: "wheel".into(),
                // A 4096-count encoder on a 15 cm wheel. Nominal, and the file
                // says as much by leaving `measured_at` unset.
                counts_per_cm: 4096.0 / (std::f64::consts::PI * 15.0),
                counts_per_rev: Some(4096),
                diameter_cm: Some(15.0),
                invert: false,
                measured_at: None,
            }],
            lines: Vec::new(),
        }
    }
}

impl RigConfig {
    pub fn load(path: &Path) -> Result<Self, String> {
        match std::fs::read_to_string(path) {
            Ok(text) => toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display())),
            // A rig that has never been configured runs on the defaults and
            // says so once, rather than refusing to start: the first thing a
            // person does with a new box is open the console.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(format!("{}: {e}", path.display())),
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let text = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        std::fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn calibration(&self) -> Calibration {
        Calibration {
            axes: self.axes.clone(),
            ball: None,
        }
    }
}

/// Which board, and how to reach it. Wiring: the file, not the API.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct DeviceConfig {
    /// The serial port. Empty means "no board on this host", which is a normal
    /// state for a development box and not an error.
    #[serde(default)]
    pub port: String,
    #[serde(default = "DeviceConfig::default_baud")]
    pub baud: u32,
}

impl DeviceConfig {
    fn default_baud() -> u32 {
        1_000_000
    }
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            port: String::new(),
            baud: Self::default_baud(),
        }
    }
}

/// Sample rates. Changeable over the API, because they are a session's concern.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct StreamConfig {
    /// What the device is asked to send. Cumulative counts, so a lower rate
    /// costs resolution and never distance.
    #[serde(default = "StreamConfig::default_rate_hz")]
    pub rate_hz: u32,
    /// The refresh rate of the display this wheel drives, when it drives one.
    ///
    /// Here so the daemon can **warn**: below the display rate, some frames see
    /// no new sample and the next sees two, and the corridor moves in uneven
    /// steps at exactly the speeds a running animal produces. Zero means no
    /// display follows this wheel and there is nothing to compare against.
    #[serde(default = "StreamConfig::default_display_hz")]
    pub display_hz: u32,
    /// How long the in-memory path stays queryable.
    #[serde(default = "StreamConfig::default_ring_minutes")]
    pub ring_minutes: u32,
}

impl StreamConfig {
    fn default_rate_hz() -> u32 {
        500
    }
    fn default_display_hz() -> u32 {
        120
    }
    fn default_ring_minutes() -> u32 {
        30
    }

    /// True when the stream cannot keep a display fed. See `display_hz`.
    pub fn starves_the_display(&self) -> bool {
        self.display_hz > 0 && self.rate_hz <= self.display_hz
    }
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            rate_hz: Self::default_rate_hz(),
            display_hz: Self::default_display_hz(),
            ring_minutes: Self::default_ring_minutes(),
        }
    }
}

/// Where samples go: the segment vstimd reads, and the socket anyone reads.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct PublishConfig {
    /// The vinput segment name. vstimd's rig config names the same string and
    /// never learns who writes it.
    #[serde(default = "PublishConfig::default_shm_name")]
    pub shm_name: String,
    /// The ZMQ PUB port, after vstimd's 5555 REP and 5556 events.
    #[serde(default = "PublishConfig::default_event_port")]
    pub event_port: u16,
}

impl PublishConfig {
    fn default_shm_name() -> String {
        "/vstimd_wheel".into()
    }
    fn default_event_port() -> u16 {
        5557
    }
}

impl Default for PublishConfig {
    fn default() -> Self {
        Self {
            shm_name: Self::default_shm_name(),
            event_port: Self::default_event_port(),
        }
    }
}

/// `GET /api/config` — the settings a session may change, and the ones it may
/// only read, in one answer.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ConfigView {
    pub rate_hz: u32,
    pub display_hz: u32,
    pub ring_minutes: u32,
    pub shm_name: String,
    pub event_port: u16,
    /// Read-only here: wiring is changed where wiring is described.
    pub port: String,
    /// True when `rate_hz` is at or below `display_hz`.
    pub starves_the_display: bool,
}

/// `PATCH /api/config`. Absent fields are left alone.
#[derive(Debug, Clone, Default, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ConfigPatch {
    pub rate_hz: Option<u32>,
    pub display_hz: Option<u32>,
    pub ring_minutes: Option<u32>,
}
