use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use tokio::sync::{broadcast, mpsc};
use tracing::warn;

use crate::proto::data::{Packet, PacketBatch};

const BATCH_MAX:    usize    = 1_000;
const BATCH_WINDOW: Duration = Duration::from_millis(500);

/// Accumulates captured packets and forwards `PacketBatch` objects to the
/// transmitter.
///
/// Flushes when the batch reaches `BATCH_MAX` packets or `BATCH_WINDOW`
/// has elapsed since the first packet.  Applies the sampling rate
/// maintained by the server via `ThrottleMonitoring` messages.
pub async fn run_batcher(
    mut rx:       mpsc::Receiver<Packet>,
    tx:           mpsc::Sender<PacketBatch>,
    sampling:     Arc<AtomicU32>,
    mut shutdown: broadcast::Receiver<()>,
) {
    let mut batch_id: u64        = 0;
    let mut buf:      Vec<Packet> = Vec::with_capacity(BATCH_MAX);

    let mut window = tokio::time::interval(BATCH_WINDOW);
    window.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            biased;

            _ = shutdown.recv() => break,

            _ = window.tick() => {
                if !buf.is_empty() {
                    flush(&mut buf, &tx, &mut batch_id).await;
                }
            }

            maybe = rx.recv() => {
                let Some(pkt) = maybe else { break };

                let rate = f32::from_bits(sampling.load(Ordering::Relaxed));
                if rate < 1.0 && rand::random::<f32>() >= rate {
                    metrics::counter!("wg_agent.packets.sampled_out").increment(1);
                    continue;
                }

                buf.push(pkt);
                if buf.len() >= BATCH_MAX {
                    flush(&mut buf, &tx, &mut batch_id).await;
                }
            }
        }
    }

    // Flush any remainder on shutdown.
    if !buf.is_empty() {
        flush(&mut buf, &tx, &mut batch_id).await;
    }
}

async fn flush(buf: &mut Vec<Packet>, tx: &mpsc::Sender<PacketBatch>, batch_id: &mut u64) {
    *batch_id += 1;
    let batch = PacketBatch {
        batch_id: *batch_id,
        packets:  std::mem::take(buf),
    };
    if tx.send(batch).await.is_err() {
        warn!("batch transmit channel closed");
    }
}
