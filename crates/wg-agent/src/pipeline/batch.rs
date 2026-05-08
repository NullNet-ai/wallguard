use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::sync::{broadcast, mpsc};
use tracing::warn;

use crate::pipeline::control::PipelineControl;
use crate::proto::data::{Packet, PacketBatch};

/// Aggregation window: one PacketBatch per second containing one row per
/// unique (src_ip, dst_ip, protocol, interface, direction) tuple seen.
const AGG_WINDOW: Duration = Duration::from_secs(1);

/// Aggregation key: the fields we group by within each window.
#[derive(Eq, PartialEq, Hash)]
struct FlowKey {
    src_ip:         String,
    dst_ip:         String,
    protocol:       u32,
    interface_name: String,
    direction:      i32,
}

/// Accumulates captured packets and emits per-flow byte-count summaries.
///
/// Within each `AGG_WINDOW` bytes are summed per (src, dst, protocol,
/// interface, direction) tuple.  At window expiry one `Packet` row is
/// emitted per tuple that saw any traffic.  This reduces DB write rate
/// from O(raw packets) to O(unique flows per second), which is orders of
/// magnitude smaller on any real network.
///
/// Port information is intentionally dropped: it creates unbounded
/// cardinality and is not needed for the bandwidth chart.
///
/// The server's `sampling_rate` is still honoured.
pub async fn run_batcher(
    mut rx:       mpsc::Receiver<Packet>,
    tx:           mpsc::Sender<PacketBatch>,
    ctrl:         Arc<PipelineControl>,
    mut shutdown: broadcast::Receiver<()>,
) {
    let mut batch_id: u64 = 0;
    let mut flows: HashMap<FlowKey, u64> = HashMap::new();

    let mut window = tokio::time::interval(AGG_WINDOW);
    window.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            biased;

            _ = shutdown.recv() => break,

            _ = window.tick() => {
                if !flows.is_empty() {
                    flush_agg(&mut flows, &tx, &mut batch_id).await;
                }
            }

            maybe = rx.recv() => {
                let Some(pkt) = maybe else { break };

                let rate = ctrl.sampling_rate();
                if rate < 1.0 && rand::random::<f32>() >= rate {
                    metrics::counter!("wg_agent.packets.sampled_out").increment(1);
                    continue;
                }

                let key = FlowKey {
                    src_ip:         pkt.src_ip,
                    dst_ip:         pkt.dst_ip,
                    protocol:       pkt.protocol,
                    interface_name: pkt.interface_name,
                    direction:      pkt.direction,
                };
                *flows.entry(key).or_insert(0) += pkt.bytes as u64;
            }
        }
    }

    if !flows.is_empty() {
        flush_agg(&mut flows, &tx, &mut batch_id).await;
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

async fn flush_agg(
    flows:    &mut HashMap<FlowKey, u64>,
    tx:       &mpsc::Sender<PacketBatch>,
    batch_id: &mut u64,
) {
    let ts = now_ms();
    let packets: Vec<Packet> = flows
        .drain()
        .map(|(key, bytes)| Packet {
            timestamp_ms:   ts,
            src_ip:         key.src_ip,
            dst_ip:         key.dst_ip,
            src_port:       0,
            dst_port:       0,
            protocol:       key.protocol,
            bytes:          bytes as u32,
            direction:      key.direction,
            interface_name: key.interface_name,
        })
        .collect();

    *batch_id += 1;
    let batch = PacketBatch { batch_id: *batch_id, packets };
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

    use crate::proto::data::Direction;

    fn make_packet(direction: i32, bytes: u32) -> Packet {
        Packet {
            timestamp_ms:   0,
            src_ip:         "1.2.3.4".into(),
            dst_ip:         "5.6.7.8".into(),
            src_port:       0,
            dst_port:       0,
            protocol:       6,
            bytes,
            direction,
            interface_name: "eth0".into(),
        }
    }

    async fn collect_batches(packets: Vec<Packet>, ctrl: Arc<PipelineControl>) -> Vec<PacketBatch> {
        let (pkt_tx, pkt_rx)         = mpsc::channel::<Packet>(1024);
        let (batch_tx, mut batch_rx) = mpsc::channel::<PacketBatch>(64);
        let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(1);

        let task = tokio::spawn(run_batcher(pkt_rx, batch_tx, ctrl, shutdown_rx));

        for pkt in packets {
            pkt_tx.send(pkt).await.unwrap();
        }

        tokio::time::sleep(Duration::from_millis(1_100)).await;
        let _ = shutdown_tx.send(());
        let _ = task.await;

        let mut out = Vec::new();
        while let Ok(b) = batch_rx.try_recv() {
            out.push(b);
        }
        out
    }

    #[tokio::test]
    async fn aggregates_bytes_by_direction() {
        let packets = vec![
            make_packet(Direction::In  as i32, 100),
            make_packet(Direction::In  as i32, 200),
            make_packet(Direction::Out as i32, 500),
        ];
        let batches = collect_batches(packets, ctrl(1.0)).await;
        let all: Vec<&Packet> = batches.iter().flat_map(|b| &b.packets).collect();

        let in_total: u64  = all.iter().filter(|p| p.direction == Direction::In  as i32).map(|p| p.bytes as u64).sum();
        let out_total: u64 = all.iter().filter(|p| p.direction == Direction::Out as i32).map(|p| p.bytes as u64).sum();

        assert_eq!(in_total,  300);
        assert_eq!(out_total, 500);
    }

    #[tokio::test]
    async fn sampling_rate_zero_drops_all() {
        let packets: Vec<Packet> = (0..100).map(|_| make_packet(Direction::In as i32, 100)).collect();
        let batches = collect_batches(packets, ctrl(0.0)).await;
        let total: u64 = batches.iter().flat_map(|b| &b.packets).map(|p| p.bytes as u64).sum();
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn batch_ids_are_monotonic() {
        let packets: Vec<Packet> = (0..10).map(|_| make_packet(Direction::In as i32, 1)).collect();
        // Two windows worth
        let (pkt_tx, pkt_rx)         = mpsc::channel::<Packet>(64);
        let (batch_tx, mut batch_rx) = mpsc::channel::<PacketBatch>(64);
        let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(1);
        let task = tokio::spawn(run_batcher(pkt_rx, batch_tx, ctrl(1.0), shutdown_rx));

        for pkt in packets {
            pkt_tx.send(pkt).await.unwrap();
        }
        tokio::time::sleep(Duration::from_millis(2_200)).await;
        let _ = shutdown_tx.send(());
        let _ = task.await;

        let mut ids = Vec::new();
        while let Ok(b) = batch_rx.try_recv() {
            ids.push(b.batch_id);
        }
        for w in ids.windows(2) {
            assert!(w[1] > w[0]);
        }
    }

    #[tokio::test]
    async fn shutdown_flushes_remainder() {
        let (pkt_tx, pkt_rx)         = mpsc::channel::<Packet>(64);
        let (batch_tx, mut batch_rx) = mpsc::channel::<PacketBatch>(64);
        let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(1);

        let task = tokio::spawn(run_batcher(pkt_rx, batch_tx, ctrl(1.0), shutdown_rx));

        pkt_tx.send(make_packet(Direction::Out as i32, 42)).await.unwrap();
        // Give the batcher time to receive the packet, then shut down before the window fires.
        tokio::time::sleep(Duration::from_millis(100)).await;
        let _ = shutdown_tx.send(());
        let _ = task.await;

        let all: Vec<Packet> = {
            let mut v = Vec::new();
            while let Ok(b) = batch_rx.try_recv() { v.extend(b.packets); }
            v
        };
        let total: u64 = all.iter().map(|p| p.bytes as u64).sum();
        assert_eq!(total, 42);
    }
}
