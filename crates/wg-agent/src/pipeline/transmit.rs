use std::sync::Arc;
use std::time::Duration;

use prost::Message;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tracing::warn;

use crate::config::Config;
use crate::disk_buffer::DiskBuffer;
use crate::pipeline::grpc::connect_with_retry;
use crate::proto::data::{data_service_client::DataServiceClient, PacketBatch};

const SESSION_BACKOFF: Duration = Duration::from_secs(5);

/// Sends `PacketBatch` objects to the server's Data gRPC service.
///
/// On connection failure batches are written to `disk_buf`.
/// On reconnect buffered files are drained before live data resumes.
pub async fn run_transmitter(
    mut rx:       mpsc::Receiver<PacketBatch>,
    config:       Arc<Config>,
    disk_buf:     Arc<DiskBuffer>,
    mut shutdown: broadcast::Receiver<()>,
) {
    loop {
        let Some(mut client) = connect_with_retry(&config, &mut shutdown).await else { return };
        run_session(&mut client, &mut rx, &disk_buf, &mut shutdown).await;
        // Back off before reconnecting so a persistent RPC failure (e.g. mTLS
        // rejection before enrollment completes) doesn't spin at full speed.
        if shutdown.try_recv().is_ok() { return; }
        tokio::time::sleep(SESSION_BACKOFF).await;
    }
}

async fn run_session(
    client:   &mut DataServiceClient<tonic::transport::Channel>,
    rx:       &mut mpsc::Receiver<PacketBatch>,
    disk_buf: &DiskBuffer,
    shutdown: &mut broadcast::Receiver<()>,
) {
    let (stream_tx, stream_rx) = mpsc::channel::<PacketBatch>(64);
    let upload_stream          = ReceiverStream::new(stream_rx);

    let rsp = match client.upload_packets(upload_stream).await {
        Ok(r)  => r,
        Err(e) => { warn!("upload_packets RPC failed: {e}"); return; }
    };
    let mut acks = rsp.into_inner();

    // Drain buffered files before sending live data.
    for path in disk_buf.drain_ordered() {
        if shutdown.try_recv().is_ok() { return; }

        let data = match std::fs::read(&path) {
            Ok(d)  => d,
            Err(e) => { warn!("disk_buf read {}: {e}", path.display()); disk_buf.remove(&path); continue; }
        };
        match PacketBatch::decode(data.as_slice()) {
            Ok(batch) => { if stream_tx.send(batch).await.is_err() { return; } disk_buf.remove(&path); }
            Err(e)    => { warn!("disk_buf decode {}: {e} — dropping", path.display()); disk_buf.remove(&path); }
        }
    }

    // Forward live batches until the session ends.
    loop {
        tokio::select! {
            biased;

            _ = shutdown.recv() => return,

            result = acks.message() => {
                match result {
                    Ok(Some(_)) => {}
                    Ok(None)    => { warn!("data gRPC: server closed ack stream"); return; }
                    Err(e)      => { warn!("data gRPC ack error: {e}"); return; }
                }
            }

            batch = rx.recv() => {
                let Some(batch) = batch else { return };
                let packet_count = batch.packets.len() as u64;
                match stream_tx.try_send(batch) {
                    Ok(()) => {
                        metrics::counter!("wg_agent_packets_sent_total").increment(packet_count);
                    }
                    Err(mpsc::error::TrySendError::Full(b)) => {
                        if !disk_buf.try_write(&b.encode_to_vec()) {
                            metrics::counter!("wg_agent.packets.dropped")
                                .increment(b.packets.len() as u64);
                        }
                    }
                    Err(mpsc::error::TrySendError::Closed(b)) => {
                        disk_buf.try_write(&b.encode_to_vec());
                        return;
                    }
                }
            }
        }
    }
}
