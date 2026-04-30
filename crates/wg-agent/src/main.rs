#![allow(dead_code)]

use std::sync::Arc;
use std::sync::atomic::AtomicU32;

use clap::Parser;
use tokio::sync::{broadcast, mpsc, watch};
use tracing::{error, info};

mod backoff;
mod capabilities;
mod cli_server;
mod config;
mod control_channel;
mod http_scanner;
mod disk_buffer;
mod failure_buffer;
mod lifecycle;
mod panic_hook;
mod pipeline;
mod platform;
mod proto;
mod proto_conv;
mod state;
mod state_machine;
mod tls;
mod tunnel;

use config::Config;
use disk_buffer::DiskBuffer;
use failure_buffer::FailureBuffer;
use state::DaemonState;

// ---------------------------------------------------------------------------
// CLI args
// ---------------------------------------------------------------------------

#[derive(Parser, Debug)]
#[command(name = "wg-agent", about = "WallGuard device agent")]
struct Args {
    /// Path to the agent config file.
    #[arg(long, default_value = "/etc/wallguard/config.toml")]
    config: std::path::PathBuf,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("failed to install rustls crypto provider");

    let args = Args::parse();

    let config = match Config::load(&args.config) {
        Ok(c) => Arc::new(c),
        Err(e) => {
            eprintln!("error: {e}\nRun `wg-cli enroll` to create the configuration file.");
            std::process::exit(1);
        }
    };

    // Init failure buffer before the async runtime so the panic hook can use it.
    let buf_dir = config
        .transmission
        .disk_buffer_path
        .parent()
        .unwrap_or(std::path::Path::new("/var/lib/wallguard"));
    failure_buffer::BUFFER.get_or_init(|| {
        FailureBuffer::load_or_create(buf_dir.join("failures.jsonl"))
    });

    panic_hook::install();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    if let Err(e) = rt.block_on(run(config)) {
        error!("agent exited with error: {e:#}");
        std::process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// Async entry
// ---------------------------------------------------------------------------

async fn run(config: Arc<Config>) -> anyhow::Result<()> {
    init_tracing(&config);
    info!(
        version = env!("CARGO_PKG_VERSION"),
        os      = ?platform::TARGET_OS,
        "wg-agent starting"
    );

    if config.observability.metrics_port != 0 {
        metrics_exporter_prometheus::PrometheusBuilder::new()
            .with_http_listener(([0, 0, 0, 0], config.observability.metrics_port))
            .install()
            .unwrap_or_else(|e| tracing::warn!("metrics endpoint failed: {e}"));
        tracing::info!(port = config.observability.metrics_port, "agent metrics endpoint listening");
    }

    let rd_available = capabilities::probe_remote_desktop().await;
    let features     = wg_shared::capabilities::derive_capabilities(
        config.device.firewall_kind,
        rd_available,
    );

    let (shutdown_tx, _) = broadcast::channel::<()>(1);
    let (state_tx, state_rx) = watch::channel(DaemonState::Provisioning);

    // Packet pipeline shared state.
    let disk_buf      = Arc::new(DiskBuffer::new(
        config.transmission.disk_buffer_path.clone(),
        config.transmission.disk_buffer_max_bytes,
        config.transmission.disk_min_free_bytes,
    ));
    let sampling_rate = Arc::new(AtomicU32::new(1.0f32.to_bits())); // 100%

    // Packet pipeline channels.
    let (cap_tx, cap_rx)     = mpsc::channel::<proto::data::Packet>(config.transmission.packet_queue_depth);
    let (batch_tx, batch_rx) = mpsc::channel::<proto::data::PacketBatch>(32);

    // Spawn pipeline tasks.
    pipeline::capture::spawn(cap_tx);
    tokio::spawn(pipeline::batch::run_batcher(
        cap_rx, batch_tx, sampling_rate.clone(), shutdown_tx.subscribe(),
    ));
    tokio::spawn(pipeline::transmit::run_transmitter(
        batch_rx, config.clone(), disk_buf.clone(), shutdown_tx.subscribe(),
    ));

    // Signal handler task — converts SIGTERM / SIGINT into the shutdown channel.
    let sig_tx = shutdown_tx.clone();
    tokio::spawn(async move {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigterm = signal(SignalKind::terminate())
                .expect("failed to register SIGTERM handler");
            tokio::select! {
                _ = sigterm.recv()          => info!("SIGTERM received"),
                _ = tokio::signal::ctrl_c() => info!("SIGINT received"),
            }
        }
        #[cfg(not(unix))]
        { let _ = tokio::signal::ctrl_c().await; }

        let _ = sig_tx.send(());
    });

    // CLI gRPC server on Unix socket — background task.
    let cli_handle = {
        let cfg = config.clone();
        let rx  = state_rx.clone();
        let tx  = shutdown_tx.clone();
        tokio::spawn(async move { cli_server::run_cli_server(cfg, rx, tx).await })
    };

    state_machine::run_state_machine(
        config, features, state_tx, shutdown_tx.subscribe(),
        disk_buf, sampling_rate,
    ).await?;

    // Stop CLI server.
    let _ = shutdown_tx.send(());
    cli_handle.abort();
    let _ = cli_handle.await;

    info!("wg-agent stopped");
    Ok(())
}

fn init_tracing(config: &Config) {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    if config.observability.log_format == "json" {
        fmt().json().with_env_filter(filter).init();
    } else {
        fmt().with_env_filter(filter).init();
    }
}
