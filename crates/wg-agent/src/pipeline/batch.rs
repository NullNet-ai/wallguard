use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{broadcast, mpsc};
use tracing::warn;

use crate::pipeline::control::PipelineControl;
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
    ctrl:         Arc<PipelineControl>,
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

                let rate = ctrl.sampling_rate();
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::broadcast;

    fn ctrl(rate: f32) -> Arc<PipelineControl> {
        let c = Arc::new(PipelineControl::new());
        c.set_sampling_rate(rate);
        c
    }

    /// Drive the batcher for one window tick with `n` packets, return all
    /// batches that arrive on `batch_rx` within a short deadline.
    async fn collect_batches(
        packets:  Vec<Packet>,
        ctrl:     Arc<PipelineControl>,
    ) -> Vec<PacketBatch> {
        let (pkt_tx, pkt_rx)     = mpsc::channel::<Packet>(1024);
        let (batch_tx, mut batch_rx) = mpsc::channel::<PacketBatch>(64);
        let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(1);

        let task = tokio::spawn(run_batcher(pkt_rx, batch_tx, ctrl, shutdown_rx));

        for pkt in packets {
            pkt_tx.send(pkt).await.unwrap();
        }

        // Give the batcher a moment to accumulate, then shut it down.
        tokio::time::sleep(Duration::from_millis(600)).await;
        let _ = shutdown_tx.send(());
        let _ = task.await;

        let mut out = Vec::new();
        while let Ok(b) = batch_rx.try_recv() {
            out.push(b);
        }
        out
    }

    fn make_packet() -> Packet {
        Packet {
            timestamp_ms: 0,
            src_ip:  "1.2.3.4".into(),
            dst_ip:  "5.6.7.8".into(),
            src_port: 1234,
            dst_port: 80,
            protocol: 6,
            bytes:    100,
            direction: 1,
        }
    }

    #[tokio::test]
    async fn flushes_on_window_tick() {
        tokio::time::pause();
        let (pkt_tx, pkt_rx)         = mpsc::channel::<Packet>(64);
        let (batch_tx, mut batch_rx) = mpsc::channel::<PacketBatch>(64);
        let (sd_tx, sd_rx)           = broadcast::channel::<()>(1);

        let c = Arc::new(PipelineControl::new());
        tokio::spawn(run_batcher(pkt_rx, batch_tx, c, sd_rx));

        pkt_tx.send(make_packet()).await.unwrap();
        pkt_tx.send(make_packet()).await.unwrap();

        // Yield so the batcher processes the initial immediate tick (empty buf)
        // and then both packets before we advance the clock.
        for _ in 0..4 { tokio::task::yield_now().await; }

        // Advance past the 500ms window.
        tokio::time::advance(Duration::from_millis(600)).await;

        // Yield so the batcher processes the window tick and flushes.
        for _ in 0..4 { tokio::task::yield_now().await; }

        let batch = batch_rx.try_recv().expect("should have flushed on tick");
        assert_eq!(batch.packets.len(), 2);
        let _ = sd_tx.send(());
    }

    #[tokio::test]
    async fn flushes_at_batch_max() {
        // Freeze time so the 500ms window never fires mid-test.
        tokio::time::pause();
        let (pkt_tx, pkt_rx)         = mpsc::channel::<Packet>(BATCH_MAX + 1);
        let (batch_tx, mut batch_rx) = mpsc::channel::<PacketBatch>(64);
        let (_sd_tx, sd_rx)          = broadcast::channel::<()>(1);

        let c = ctrl(1.0);
        let task = tokio::spawn(run_batcher(pkt_rx, batch_tx, c, sd_rx));

        // Yield once so the batcher processes the initial immediate tick before
        // any packets arrive.
        tokio::task::yield_now().await;

        for _ in 0..BATCH_MAX {
            pkt_tx.send(make_packet()).await.unwrap();
        }
        // Dropping the sender closes the channel; the batcher will process all
        // buffered packets, flush at BATCH_MAX, then exit on None.
        drop(pkt_tx);

        let _ = task.await;

        let batch = batch_rx.try_recv().expect("should flush at BATCH_MAX");
        assert_eq!(batch.packets.len(), BATCH_MAX);
        assert!(batch_rx.try_recv().is_err(), "no leftover batches expected");
    }

    #[tokio::test]
    async fn sampling_rate_zero_drops_all() {
        let packets: Vec<Packet> = (0..100).map(|_| make_packet()).collect();
        let batches = collect_batches(packets, ctrl(0.0)).await;
        let total: usize = batches.iter().map(|b| b.packets.len()).sum();
        assert_eq!(total, 0, "rate=0.0 should drop every packet");
    }

    #[tokio::test]
    async fn batch_ids_are_monotonic() {
        let packets: Vec<Packet> = (0..BATCH_MAX * 2 + 1).map(|_| make_packet()).collect();
        let batches = collect_batches(packets, ctrl(1.0)).await;
        for w in batches.windows(2) {
            assert!(w[1].batch_id > w[0].batch_id, "batch_ids must increase");
        }
    }

    #[tokio::test]
    async fn shutdown_flushes_remainder() {
        // Freeze time so the window never fires. Keep _sd_tx alive so the
        // shutdown branch is never ready — closure of pkt_tx is what terminates
        // the loop (rx.recv() returns None), after which the remainder is flushed.
        tokio::time::pause();
        let (pkt_tx, pkt_rx)         = mpsc::channel::<Packet>(64);
        let (batch_tx, mut batch_rx) = mpsc::channel::<PacketBatch>(64);
        let (_sd_tx, sd_rx)          = broadcast::channel::<()>(1);

        let c = Arc::new(PipelineControl::new());
        let task = tokio::spawn(run_batcher(pkt_rx, batch_tx, c, sd_rx));

        pkt_tx.send(make_packet()).await.unwrap();
        pkt_tx.send(make_packet()).await.unwrap();

        // Let the batcher receive both packets before we close the channel.
        for _ in 0..4 { tokio::task::yield_now().await; }

        // Closing the sender makes rx.recv() return None, which breaks the loop
        // and triggers the post-loop remainder flush.
        drop(pkt_tx);

        let _ = task.await;

        let batch = batch_rx.try_recv().expect("remainder must be flushed on channel close");
        assert_eq!(batch.packets.len(), 2);
    }
}
