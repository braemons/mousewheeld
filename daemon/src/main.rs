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
use mousewheeld::model::config::{RigConfig, StoredCalibration};
use mousewheeld::zones::ZoneSetStore;
use mousewheeld::{device, grpc, publish, web};

/// vstimd is 8080, statemachined 8081, triald 8420.
///
/// **8083 and not 8082, which is where this used to be.** A Python daemon
/// cannot serve gRPC and a browser on one socket -- `grpc.aio` owns its port
/// outright and no ASGI server speaks native gRPC -- so it binds *two*: the
/// panels on its port and gRPC on one above it. statemachined was Python and
/// held 8081 and 8082, and a rig running both daemons on their defaults had a
/// collision. statemachined is Rust now and 8082 is free; triald still holds
/// 8420 and 8421.
///
/// This daemon is the one that moved because it is the one that needs a single
/// port: tonic-web serves the panels and the rpcs on the same socket, which is
/// the whole reason there is no `+ 1` here. `contracts/DAEMON_LAYOUT.md` has
/// the family's allocation and the rule that produced it.
const DEFAULT_PORT: u16 = 8083;
const DEFAULT_RIG_CONFIG: &str = "/etc/braemons/mousewheeld-rig-config.toml";
/// Under the family's one parent, so a rig has one directory to back up.
const DEFAULT_STORAGE_DIR: &str = "/var/lib/braemons/mousewheeld";

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
        /// Do not advertise `_mousewheeld._tcp`. The API is served either way;
        /// a console then needs this rig's address by hand.
        #[arg(long)]
        no_mdns: bool,
    },
    /// Print the JSON Schema of a zone-set file.
    ///
    /// A subcommand and not an rpc, because a zone set is a *file*: an editor
    /// with one open and a CI job checking one both need its schema, and
    /// neither should have to start a daemon to get it. The committed copy in
    /// `docs/reference/zone-set.schema.json` is this output.
    Schema,
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
            no_mdns,
        } => serve(port, bind, rig_config, storage_dir, simulate, no_mdns),
        Command::Schema => {
            let schema = mousewheeld::file_schema::zone_set_schema();
            println!(
                "{}",
                serde_json::to_string_pretty(&schema).expect("a schema")
            );
        }
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

fn serve(
    port: u16,
    bind: String,
    rig_config: PathBuf,
    storage_dir: PathBuf,
    simulate: bool,
    no_mdns: bool,
) {
    let mut config = match RigConfig::load(&rig_config) {
        Ok(config) => config,
        Err(problem) => {
            eprintln!("mousewheeld: {problem}");
            std::process::exit(1);
        }
    };
    // The calibration the API last measured, over the file's. The rig config is
    // read only; this file in the storage directory is what the API writes.
    let calibration_path = storage_dir.join("calibration.toml");
    match StoredCalibration::load(&calibration_path) {
        Ok(Some(stored)) => {
            for name in config.apply_stored_calibration(&stored) {
                log::warn!(
                    "{} calibrates an axis named {name}, and the rig config has none: ignored",
                    calibration_path.display()
                );
            }
            log::info!("calibration from {}", calibration_path.display());
        }
        Ok(None) => {}
        Err(problem) => {
            eprintln!("mousewheeld: {problem}");
            std::process::exit(1);
        }
    }
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
        (false, port) => Backend::Port {
            path: port.to_string(),
            baud: config.device.baud,
        },
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
        &config
            .axes
            .iter()
            .map(|axis| axis.name.clone())
            .collect::<Vec<_>>(),
    );
    let device = Device::new(backend, axes, lines, stream_rate_hz, publisher);

    let daemon = Arc::new(Daemon {
        calibration_path,
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
        log::info!("mousewheeld on {address}  (panels at /, gRPC and reflection on the same port)");

        // **axum and tonic on one listener.**
        //
        // A tonic service is a tower service, so each of the five merges into
        // the axum router that serves the panels. One port, one systemd unit,
        // one line in the firewall — the deb does not change because the
        // daemon grew an RPC surface.
        //
        // `tonic_web` translates gRPC-Web in process, so a browser reaches
        // these without a proxy and without a second daemon. Bidirectional
        // streaming is the one thing it cannot carry, which matters for
        // exactly one rpc: reflection. A browser therefore cannot discover the
        // API this way; a client library can, and does.
        // Each service is its own type, so this is five calls rather than a loop.
        // `tonic::service::Routes` collects the services and hands back an axum
        // router, which merges with the one serving the panels. Each service
        // registers its own path — `/mousewheeld.v1.Zones/…` — so no route is
        // written down anywhere and renaming a service in the `.proto` moves it.
        let daemon_services = grpc::DaemonServices::new(daemon.clone());
        let mut routes = tonic::service::Routes::builder();
        routes
            .add_service(grpc::device::server(daemon_services.clone()))
            .add_service(grpc::state::server(daemon_services.clone()))
            .add_service(grpc::calibration::server(daemon_services.clone()))
            .add_service(grpc::zones::server(daemon_services.clone()))
            .add_service(grpc::config::server(daemon_services));

        // Both reflection versions: clients disagree about which to ask for,
        // and grpcurl and Python's reflection database still want v1alpha.
        let reflection = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(mousewheeld::wire::DESCRIPTOR)
            .build_v1()
            .expect("the descriptor set this binary was built from");
        let reflection_alpha = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(mousewheeld::wire::DESCRIPTOR)
            .build_v1alpha()
            .expect("the descriptor set this binary was built from");
        routes.add_service(reflection).add_service(reflection_alpha);

        // **CORS is open, and `/elements/` is why.** A console served from
        // somewhere else imports these panels by URL and calls this daemon
        // from its own origin. `permissive` rather than `very_permissive`: the
        // latter allows credentials, which cannot be combined with a wildcard
        // `expose-headers`, and exposing `grpc-status` and `grpc-message` is
        // how a refusal reaches a panel across an origin. It went missing with
        // the REST layer once, and a console could not load a single panel.
        let cors = tower_http::cors::CorsLayer::permissive();

        // gRPC-Web on the whole stack rather than per service: the layer only
        // acts on requests that arrive with a gRPC-Web content type, so the
        // panels pass through it untouched.
        let app = web::router(daemon)
            .merge(
                routes
                    .routes()
                    .into_axum_router()
                    .layer(tonic_web::GrpcWebLayer::new()),
            )
            .layer(cors);

        // Advertised once the port is bound, so a console that finds the
        // record finds a daemon; withdrawn when it stops.
        let mut advertisement = (!no_mdns).then(|| {
            let mut advertisement =
                mousewheeld::mdns_service_advertisement::MdnsServiceAdvertisement::new(
                    port,
                    env!("CARGO_PKG_VERSION"),
                );
            advertisement.start();
            advertisement
        });

        let served = axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = tokio::signal::ctrl_c().await;
                log::info!("mousewheeld: stopping");
            })
            .await;
        if let Some(advertisement) = advertisement.as_mut() {
            advertisement.stop();
        }
        if let Err(problem) = served {
            eprintln!("mousewheeld: {problem}");
            std::process::exit(1);
        }
    });
}
