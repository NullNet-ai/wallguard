use std::collections::HashSet;
use std::time::{Duration, Instant};

use sqlx::PgPool;
use uuid::Uuid;

use crate::proto::control::MonitoringStatus;

const MISSED_ACK_LIMIT:   usize    = 3;
const DB_WRITE_THROTTLE:  Duration = Duration::from_secs(60);

// ---------------------------------------------------------------------------
// Per-connection heartbeat state (server → agent direction)
// ---------------------------------------------------------------------------

pub struct HeartbeatState {
    /// Next sequence number to send.
    next_seq:      u64,
    /// Sequences sent but not yet acked.
    in_flight:     HashSet<u64>,
    /// Last time we wrote monitoring data to the DB.
    last_db_write: Option<Instant>,
}

impl HeartbeatState {
    pub fn new() -> Self {
        Self {
            next_seq:      0,
            in_flight:     HashSet::new(),
            last_db_write: None,
        }
    }

    /// Returns the next sequence number to use for a Heartbeat, and advances
    /// the internal counter. Also records the seq in `in_flight`.
    pub fn next_seq(&mut self) -> u64 {
        let seq = self.next_seq;
        self.next_seq += 1;
        self.in_flight.insert(seq);
        seq
    }

    /// Record an ack from the agent.
    pub fn on_ack(&mut self, ack_seq: u64) {
        self.in_flight.remove(&ack_seq);
    }

    /// Returns `true` if the connection should be closed due to too many
    /// consecutive missed acks.
    pub fn should_disconnect(&self) -> bool {
        self.in_flight.len() >= MISSED_ACK_LIMIT
    }

    /// Returns `true` if enough time has passed to write monitoring data to
    /// the DB, and resets the timer.
    pub fn should_write_db(&mut self) -> bool {
        let now = Instant::now();
        match self.last_db_write {
            Some(t) if now.duration_since(t) < DB_WRITE_THROTTLE => false,
            _ => {
                self.last_db_write = Some(now);
                true
            }
        }
    }
}

// ---------------------------------------------------------------------------
// DB write helper
// ---------------------------------------------------------------------------

/// Write a monitoring snapshot to the `device_monitoring_status` hypertable.
/// Called at most once per minute per device (caller is responsible for
/// throttling via `HeartbeatState::should_write_db`).
pub async fn record_monitoring_status(
    pool:      &PgPool,
    device_id: Uuid,
    status:    &MonitoringStatus,
) {
    let result = sqlx::query(
        r#"
        INSERT INTO device_monitoring_status
            (time, device_id,
             packet_queue_depth, disk_buffer_bytes, disk_buffer_max_bytes,
             packets_dropped_total, packets_sent_total,
             degraded, active_tunnel_count)
        VALUES (NOW(), $1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(device_id)
    .bind(status.packet_queue_depth  as i32)
    .bind(status.disk_buffer_bytes   as i64)
    .bind(status.disk_buffer_max_bytes as i64)
    .bind(status.packets_dropped_total as i64)
    .bind(status.packets_sent_total    as i64)
    .bind(status.degraded)
    .bind(status.active_tunnel_count as i32)
    .execute(pool)
    .await;

    if let Err(e) = result {
        tracing::warn!(%device_id, "failed to write monitoring status: {e}");
    }
}
