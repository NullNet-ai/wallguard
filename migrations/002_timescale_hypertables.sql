-- Migration 002 — TimescaleDB time-series tables and hypertables.
--
-- Requires: timescaledb extension (created in 001).
-- Chunk intervals are sized for expected data rates:
--   packets            — high write rate; 1-hour chunks
--   resource_metrics   — moderate rate;   4-hour chunks
--   device_monitoring_status — low rate;  1-day chunks

-- ---------------------------------------------------------------------------
-- Network packet telemetry
-- ---------------------------------------------------------------------------

CREATE TABLE packets (
    time       TIMESTAMPTZ NOT NULL,
    device_id  UUID        NOT NULL,
    src_ip     INET,
    dst_ip     INET,
    src_port   INTEGER,
    dst_port   INTEGER,
    protocol   SMALLINT,              -- IANA protocol number
    bytes      INTEGER,
    direction  TEXT CHECK (direction IN ('in','out'))
);

SELECT create_hypertable(
    'packets',
    'time',
    chunk_time_interval => INTERVAL '1 hour'
);

-- Composite index: device + time allows efficient per-device time-range queries.
CREATE INDEX ON packets (device_id, time DESC);

-- ---------------------------------------------------------------------------
-- Resource metrics (CPU, memory, disk, load)
-- ---------------------------------------------------------------------------

CREATE TABLE resource_metrics (
    time              TIMESTAMPTZ NOT NULL,
    device_id         UUID        NOT NULL,
    cpu_percent       REAL,
    mem_used_bytes    BIGINT,
    mem_total_bytes   BIGINT,
    disk_used_bytes   BIGINT,
    disk_total_bytes  BIGINT,
    load_1m           REAL,
    load_5m           REAL
);

SELECT create_hypertable(
    'resource_metrics',
    'time',
    chunk_time_interval => INTERVAL '4 hours'
);

CREATE INDEX ON resource_metrics (device_id, time DESC);

-- ---------------------------------------------------------------------------
-- Device monitoring status snapshots (from heartbeats; at most once/minute/device)
-- ---------------------------------------------------------------------------

CREATE TABLE device_monitoring_status (
    time                    TIMESTAMPTZ NOT NULL,
    device_id               UUID        NOT NULL,
    packet_queue_depth      INTEGER,
    disk_buffer_bytes       BIGINT,
    disk_buffer_max_bytes   BIGINT,
    packets_dropped_total   BIGINT,
    packets_sent_total      BIGINT,
    degraded                BOOLEAN,
    active_tunnel_count     INTEGER
);

SELECT create_hypertable(
    'device_monitoring_status',
    'time',
    chunk_time_interval => INTERVAL '1 day'
);

CREATE INDEX ON device_monitoring_status (device_id, time DESC);
