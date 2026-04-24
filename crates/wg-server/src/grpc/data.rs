use std::pin::Pin;

use tonic::{Request, Response, Status, Streaming};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{info, warn};
use uuid::Uuid;

use crate::grpc::extract_device_id;
use crate::proto::data::{
    data_service_server::DataService,
    BatchAck, PacketBatch, ResourceMetricsBatch,
};

pub use crate::proto::data::data_service_server::DataServiceServer as DataServer;

pub struct DataSvc {
    pub pool: sqlx::PgPool,
}

type AckStream = Pin<Box<dyn tonic::codegen::tokio_stream::Stream<
    Item = Result<BatchAck, Status>,
> + Send + 'static>>;

#[tonic::async_trait]
impl DataService for DataSvc {
    type UploadPacketsStream         = AckStream;
    type UploadResourceMetricsStream = AckStream;

    async fn upload_packets(
        &self,
        request: Request<Streaming<PacketBatch>>,
    ) -> Result<Response<AckStream>, Status> {
        let device_id = extract_device_id(&request)
            .ok_or_else(|| Status::unauthenticated("no valid device certificate"))?;

        let pool   = self.pool.clone();
        let mut rx = request.into_inner();

        let (ack_tx, ack_rx) = tokio::sync::mpsc::channel::<Result<BatchAck, Status>>(64);

        tokio::spawn(async move {
            while let Ok(Some(batch)) = rx.message().await {
                let batch_id = batch.batch_id;
                let n        = batch.packets.len();

                if let Err(e) = insert_packets(&pool, device_id, batch).await {
                    warn!(%device_id, "packet insert failed: {e}");
                } else {
                    metrics::counter!("wg_server.packets.received").increment(n as u64);
                    info!(%device_id, batch_id, packets = n, "inserted packet batch");
                }

                if ack_tx.send(Ok(BatchAck { batch_id })).await.is_err() {
                    break;
                }
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(ack_rx))))
    }

    async fn upload_resource_metrics(
        &self,
        request: Request<Streaming<ResourceMetricsBatch>>,
    ) -> Result<Response<AckStream>, Status> {
        // Phase 7 stub: drain and ack. Real ingestion requires nullnet-libresmon.
        let mut rx = request.into_inner();
        let (ack_tx, ack_rx) = tokio::sync::mpsc::channel::<Result<BatchAck, Status>>(64);
        tokio::spawn(async move {
            while let Ok(Some(batch)) = rx.message().await {
                if ack_tx.send(Ok(BatchAck { batch_id: batch.batch_id })).await.is_err() {
                    break;
                }
            }
        });
        Ok(Response::new(Box::pin(ReceiverStream::new(ack_rx))))
    }
}

async fn insert_packets(
    pool:      &sqlx::PgPool,
    device_id: Uuid,
    batch:     PacketBatch,
) -> anyhow::Result<()> {
    use time::OffsetDateTime;

    let mut tx = pool.begin().await?;
    for pkt in batch.packets {
        let ts = OffsetDateTime::from_unix_timestamp_nanos(
            pkt.timestamp_ms as i128 * 1_000_000,
        )
        .unwrap_or_else(|_| OffsetDateTime::now_utc());

        let dir = match pkt.direction {
            1 => "in",
            2 => "out",
            _ => "in",
        };

        sqlx::query(
            "INSERT INTO packets \
             (time, device_id, src_ip, dst_ip, src_port, dst_port, protocol, bytes, direction) \
             VALUES ($1, $2, $3::inet, $4::inet, $5, $6, $7, $8, $9)",
        )
        .bind(ts)
        .bind(device_id)
        .bind(&pkt.src_ip)
        .bind(&pkt.dst_ip)
        .bind(pkt.src_port as i32)
        .bind(pkt.dst_port as i32)
        .bind(pkt.protocol as i16)
        .bind(pkt.bytes as i32)
        .bind(dir)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}
