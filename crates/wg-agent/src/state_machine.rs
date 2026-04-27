use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::time::Duration;

use tokio::sync::{broadcast, watch};
use tracing::{info, warn};
use wg_shared::types::Feature;

use crate::backoff::Backoff;
use crate::config::Config;
use crate::control_channel::{run_connected_loop, try_connect, ConnectResult};
use crate::disk_buffer::DiskBuffer;
use crate::failure_buffer;
use crate::failure_buffer::FailureBuffer;
use crate::state::{DaemonState, IdleReason};

pub async fn run_state_machine(
    config:       Arc<Config>,
    features:     Vec<Feature>,
    state_tx:     watch::Sender<DaemonState>,
    mut shutdown: broadcast::Receiver<()>,
    disk_buf:     Arc<DiskBuffer>,
    sampling:     Arc<AtomicU32>,
) -> anyhow::Result<()> {
    let buf    = failure_buffer::BUFFER.get().expect("buffer must be initialised");
    let mut bo = Backoff::new(
        config.agent.reconnect_base_s as f64,
        config.agent.reconnect_max_s  as f64,
        2.0,
    );
    let mut state = initial_state(&config);

    loop {
        state_tx.send(state.clone()).ok();

        let next = tokio::select! {
            biased;
            _ = shutdown.recv() => {
                info!("shutdown signal — stopping state machine");
                break;
            }
            next = step(&state, &config, &features, buf, &mut bo, &disk_buf, &sampling) => next,
        };

        state = next;
    }

    Ok(())
}

fn initial_state(config: &Config) -> DaemonState {
    if config.tls.device_cert.exists() && config.tls.device_key.exists() {
        DaemonState::Connecting
    } else {
        DaemonState::Provisioning
    }
}

async fn step(
    state:    &DaemonState,
    config:   &Arc<Config>,
    features: &[Feature],
    buf:      &'static FailureBuffer,
    bo:       &mut Backoff,
    disk_buf: &Arc<DiskBuffer>,
    sampling: &Arc<AtomicU32>,
) -> DaemonState {
    match state {
        DaemonState::Provisioning     => step_provisioning(config).await,
        DaemonState::Idle(reason)     => step_idle(reason).await,
        DaemonState::Connecting       => step_connecting(config, features, buf, bo, disk_buf, sampling).await,
        DaemonState::Connected { .. } => DaemonState::Connecting,
    }
}

async fn step_provisioning(config: &Config) -> DaemonState {
    tokio::time::sleep(Duration::from_secs(5)).await;
    if config.tls.device_cert.exists() && config.tls.device_key.exists() {
        info!("device certificate found — connecting");
        DaemonState::Connecting
    } else {
        DaemonState::Provisioning
    }
}

async fn step_idle(reason: &IdleReason) -> DaemonState {
    match reason {
        IdleReason::VersionRejected { min_required } => {
            warn!(
                min_required,
                "server requires protocol version {min_required}; \
                 manual agent upgrade required — agent will not retry"
            );
        }
    }
    // Park indefinitely; only the outer shutdown select exits this.
    std::future::pending::<DaemonState>().await
}

async fn step_connecting(
    config:   &Arc<Config>,
    features: &[Feature],
    buf:      &'static FailureBuffer,
    bo:       &mut Backoff,
    disk_buf: &Arc<DiskBuffer>,
    sampling: &Arc<AtomicU32>,
) -> DaemonState {
    info!(server = %config.server.name, "connecting …");

    match try_connect(config, features).await {
        Err(e) => {
            let delay = bo.next();
            warn!("connection failed: {e:#} — retry in {delay:?}");
            metrics::counter!("wg_agent_reconnect_attempts_total").increment(1);
            tokio::time::sleep(delay).await;
            DaemonState::Connecting
        }
        Ok(ConnectResult::VersionRejected(min)) => {
            DaemonState::Idle(IdleReason::VersionRejected { min_required: min })
        }
        Ok(ConnectResult::Connected(cs)) => {
            bo.reset();
            info!("connected; running connected loop");
            run_connected_loop(cs, config, buf, disk_buf, sampling).await
        }
    }
}
