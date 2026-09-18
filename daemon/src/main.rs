// SPDX-License-Identifier: AGPL-3.0-or-later
//
//! mousewheeld — the locomotion input of a braemons rig.
//!
//! One binary, two subcommands: `serve` owns a board and publishes what it
//! reads; `relay` subscribes to another host's stream and writes local shared
//! memory, so a camera on a second machine is served without teaching vstimd a
//! network position backend.
//!
//! **What is built here is the public surface**: the API, the zone-set store
//! and compiler, the calibration and its measurement procedure, the config, and
//! the console elements. Behind it, where the real-time thread and the serial
//! link will be, is a simulated wheel — `--simulate`. The seam is
//! [`device::Device`], and nothing above it changes when a board arrives.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use clap::{Parser, Subcommand};

use mousewheeld::daemon_state::Daemon;
use mousewheeld::device::{Backend, Device};
use mousewheeld::model::config::RigConfig;
use mousewheeld::zones::ZoneSetStore;
use mousewheeld::{api, device, publish};

/// statemachined is 8081, vstimd 8080, triald 8420.
const DEFAULT_PORT: u16 = 8082;
const DEFAULT_RIG_CONFIG: &str = "/etc/braemons/mousewheeld-rig-config.toml";
const DEFAULT_STORAGE_DIR: &str = "/var/lib/mousewheeld";

#[derive(Parser)]
#[command(name = "mousewheeld", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Own the board, publish what it reads, and serve the API.
    Serve {
        #[arg(long, default_value_t = DEFAULT_PORT)]
        port: u16,
        /// Bind address. Loopback by default on a development box; a rig's
        /// unit binds the rig network.
        #[arg(long, default_value = "127.0.0.1")]
        bind: String,
        /// The physical rig: the board's port, the line map, the calibration.
        #[arg(long, default_value = DEFAULT_RIG_CONFIG)]
        rig_config: PathBuf,
        /// Where the zone-set store lives.
        #[arg(long, default_value = DEFAULT_STORAGE_DIR)]
        storage_dir: PathBuf,
        /// Run a wheel on a thread instead of a board.
        ///
        /// For building against, and for a demonstration with no hardware. It
        /// evaluates zones in counts off the real compiler's output, so what it
        /// exercises is the daemon rather than a story about one.
        #[arg(long)]
        simulate: bool,
    },
    /// Subscribe to another host's stream and write the local vinput segment.
    Relay {
        /// The publishing daemon, e.g. `tcp://rig-a:5557`.
        #[arg(long)]
        from: String,
    },
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    match Cli::parse().command {
        Command::Serve {
            port,
            bind,
            rig_config,
            storage_dir,
            simulate,
        } => serve(port, bind, rig_config, storage_dir, simulate),
        Command::Relay { from } => {
            // Named here rather than hidden, because the subcommand is part of
            // the shape: a camera on another host is served by a relay, not by
            // a network position backend inside vstimd.
            eprintln!(
                "mousewheeld relay: not built yet — it arrives with the ZMQ publisher (M4).\n\
                 It will subscribe to {from} and write the local vinput segment."
            );
            std::process::exit(1);
        }
    }
}

fn serve(port: u16, bind: String, rig_config: PathBuf, storage_dir: PathBuf, simulate: bool) {
    let config = match RigConfig::load(&rig_config) {
        Ok(config) => config,
        Err(problem) => {
            eprintln!("mousewheeld: {problem}");
            std::process::exit(1);
        }
    };
    if !rig_config.exists() {
        log::warn!(
            "no rig config at {} — running on defaults. A rig's wiring and calibration live there",
            rig_config.display()
        );
    }
    if config.lines.is_empty() {
        log::warn!(
            "no output lines in the rig config: every zone set will be refused, because a zone \
             names a line and this rig has none"
        );
    }
    if config.stream.starves_the_display() {
        log::warn!(
            "stream rate {} Hz is at or below the display's {} Hz: a camera following this wheel \
             will move in uneven steps",
            config.stream.rate_hz,
            config.stream.display_hz
        );
    }

    let store = ZoneSetStore::new(storage_dir.join("zone-sets"));
    if let Err(problem) = store.seed_if_empty() {
        log::warn!("could not seed the zone-set store: {problem}");
    }

    let axes = config
        .axes
        .iter()
        .map(|axis| device::AxisSetup {
            name: axis.name.clone(),
            counts_per_cm: axis.counts_per_cm,
            invert: axis.invert,
        })
        .collect();
    // A port in the rig config is a board; `--simulate` is a pty with one on
    // the far end; neither is the same as "no device", which is what a
    // development box without either honestly has.
    let backend = match (simulate, config.device.port.as_str()) {
        (true, _) => Backend::Simulated,
        (false, "") => Backend::Absent,
        (false, port) => Backend::Port { path: port.to_string(), baud: config.device.baud },
    };
    let stream_rate_hz = config.stream.rate_hz;
    let lines = config
        .lines
        .iter()
        .map(|line| (line.index, line.pin, line.safe_high))
        .collect();
    // The segment vstimd reads every frame. Created before the link, so the
    // first sample that arrives already has somewhere to go.
    let publisher = publish::SegmentPublisher::create(
        &config.publish.shm_name,
        &config.axes.iter().map(|axis| axis.name.clone()).collect::<Vec<_>>(),
    );
    let device = Device::new(backend, axes, lines, stream_rate_hz, publisher);

    let daemon = Arc::new(Daemon {
        config_path: rig_config,
        config: Mutex::new(config),
        store,
        device,
        measuring: Mutex::new(None),
        measured: Mutex::new(None),
    });

    if simulate {
        log::warn!("--simulate: a wheel on a thread, not a board. Nothing here is a measurement");
    } else if !daemon.device.connected() {
        log::info!("no board attached; the API is up and the device routes will say so");
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("a tokio runtime");
    runtime.block_on(async move {
        let address = format!("{bind}:{port}");
        let listener = match tokio::net::TcpListener::bind(&address).await {
            Ok(listener) => listener,
            Err(problem) => {
                eprintln!("mousewheeld: cannot bind {address}: {problem}");
                std::process::exit(1);
            }
        };
        log::info!("mousewheeld on http://{address}  (panels at /, API at /api/openapi.json)");
        let app = api::router(daemon);
        if let Err(problem) = axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = tokio::signal::ctrl_c().await;
                log::info!("mousewheeld: stopping");
            })
            .await
        {
            eprintln!("mousewheeld: {problem}");
            std::process::exit(1);
        }
    });
}
