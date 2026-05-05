use std::sync::Arc;
use std::time::Duration;

use sysinfo::{Disks, System};
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::ReceiverStream;
use tracing::warn;

use crate::config::Config;
use crate::pipeline::control::PipelineControl;
use crate::pipeline::grpc::connect_with_retry;
use crate::proto::data::{
    data_service_client::DataServiceClient, ResourceMetrics, ResourceMetricsBatch,
};
use crate::proto_conv::unix_ms_now;

const COLLECT_INTERVAL: Duration = Duration::from_secs(30);

pub async fn run_metrics_pipeline(
    config:       Arc<Config>,
    ctrl:         Arc<PipelineControl>,
    mut shutdown: broadcast::Receiver<()>,
) {
    let mut sys = System::new_all();

    loop {
        let Some(mut client) = connect_with_retry(&config, &mut shutdown).await else { return };
        run_session(&mut client, &mut sys, &ctrl, &mut shutdown).await;
    }
}

async fn run_session(
    client:   &mut DataServiceClient<tonic::transport::Channel>,
    sys:      &mut System,
    ctrl:     &Arc<PipelineControl>,
    shutdown: &mut broadcast::Receiver<()>,
) {
    let (stream_tx, stream_rx) = mpsc::channel::<ResourceMetricsBatch>(8);
    let upload_stream = ReceiverStream::new(stream_rx);

    let rsp = match client.upload_resource_metrics(upload_stream).await {
        Ok(r)  => r,
        Err(e) => { warn!("upload_resource_metrics RPC failed: {e}"); return; }
    };
    let mut acks = rsp.into_inner();

    let mut batch_id: u64 = 0;
    let mut ticker = tokio::time::interval(COLLECT_INTERVAL);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            biased;

            _ = shutdown.recv() => return,

            result = acks.message() => {
                match result {
                    Ok(Some(_)) => {}
                    Ok(None)    => { warn!("metrics gRPC: server closed ack stream"); return; }
                    Err(e)      => { warn!("metrics gRPC ack error: {e}"); return; }
                }
            }

            _ = ticker.tick() => {
                if !ctrl.telemetry_enabled() {
                    continue;
                }

                let metrics = collect(sys);
                batch_id   += 1;
                let batch   = ResourceMetricsBatch { batch_id, metrics: vec![metrics] };
                if stream_tx.send(batch).await.is_err() {
                    return;
                }
            }
        }
    }
}

fn collect(sys: &mut System) -> ResourceMetrics {
    sys.refresh_cpu_all();
    sys.refresh_memory();

    let disks = Disks::new_with_refreshed_list();
    let (disk_used, disk_total) = disks.iter().fold((0u64, 0u64), |(u, t), d| {
        (
            u + d.total_space().saturating_sub(d.available_space()),
            t + d.total_space(),
        )
    });

    let load = System::load_average();

    ResourceMetrics {
        timestamp_ms:     unix_ms_now(),
        cpu_percent:      sys.global_cpu_usage(),
        mem_used_bytes:   sys.used_memory(),
        mem_total_bytes:  sys.total_memory(),
        disk_used_bytes:  disk_used,
        disk_total_bytes: disk_total,
        load_1m:          load.one  as f32,
        load_5m:          load.five as f32,
    }
}
