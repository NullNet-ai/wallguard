use tokio::sync::mpsc;

use crate::proto::data::Packet;

/// Spawns the packet capture background task.
///
/// Currently a stub — real capture requires `nullnet-traffic-monitor`
/// (see workspace Cargo.toml TODO).  The task parks indefinitely so the
/// pipeline infrastructure is fully wired; real packets will flow through
/// once the capture backend is plugged in.
pub fn spawn(tx: mpsc::Sender<Packet>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let _tx = tx; // keep sender alive so pipeline channel stays open
        metrics::gauge!("wg_agent_capture_queue_depth").set(0.0);
        std::future::pending::<()>().await
    })
}
