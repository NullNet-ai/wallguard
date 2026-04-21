-- Migration 004 — TimescaleDB data retention policies.
--
-- Data older than the configured interval is dropped automatically.
-- Adjust these values to match your storage budget before deploying.

-- Raw packet telemetry — high volume; 30-day window.
SELECT add_retention_policy('packets', INTERVAL '30 days');

-- Resource metrics — lower volume; 90-day window.
SELECT add_retention_policy('resource_metrics', INTERVAL '90 days');

-- Monitoring status snapshots — low volume; 90-day window.
SELECT add_retention_policy('device_monitoring_status', INTERVAL '90 days');
