-- Migration 003 — TimescaleDB continuous aggregates.
--
-- Continuous aggregates are maintained automatically by TimescaleDB as new
-- data arrives.  They replace expensive on-the-fly GROUP BY queries in the
-- dashboard and device-detail views.
--
-- NOTE: CREATE MATERIALIZED VIEW ... WITH (timescaledb.continuous) cannot run
-- inside a transaction block on some TimescaleDB versions.  If sqlx wraps
-- this migration in a transaction and it fails, move the statement to a
-- manual step or run it directly via psql.  In development the
-- timescale/timescaledb Docker image typically supports this fine.

-- ---------------------------------------------------------------------------
-- 5-minute packet aggregate
-- Used by the dashboard (total bytes/s over last 24 h) and device-detail
-- (packet rate chart over last 1 h).
-- ---------------------------------------------------------------------------

CREATE MATERIALIZED VIEW packets_5m
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('5 minutes', time) AS bucket,
    device_id,
    SUM(bytes)  AS total_bytes,
    COUNT(*)    AS packet_count
FROM packets
GROUP BY bucket, device_id
WITH NO DATA;

-- Refresh policy: keep the aggregate up to date automatically.
-- Lag of 1 minute avoids refreshing chunks that are still receiving writes.
SELECT add_continuous_aggregate_policy(
    'packets_5m',
    start_offset => INTERVAL '1 hour',
    end_offset   => INTERVAL '1 minute',
    schedule_interval => INTERVAL '5 minutes'
);
