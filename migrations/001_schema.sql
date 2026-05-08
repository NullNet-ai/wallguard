-- WallGuard — complete database schema.
-- Apply to a fresh database; requires TimescaleDB extension.

CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;

-- ---------------------------------------------------------------------------
-- Organizations
-- ---------------------------------------------------------------------------

CREATE TABLE organizations (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    name       TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ---------------------------------------------------------------------------
-- Users
-- ---------------------------------------------------------------------------

CREATE TABLE users (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id        UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    email         TEXT        NOT NULL,
    password_hash TEXT        NOT NULL,
    display_name  TEXT        NOT NULL,
    role          TEXT        NOT NULL CHECK (role IN ('owner','admin','operator','viewer')),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (org_id, email)
);

CREATE INDEX users_org_id_idx ON users (org_id);

-- ---------------------------------------------------------------------------
-- Auth tokens
-- ---------------------------------------------------------------------------

CREATE TABLE refresh_tokens (
    jti        UUID        PRIMARY KEY,
    user_id    UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX refresh_tokens_user_id_idx ON refresh_tokens (user_id);

CREATE TABLE revoked_tokens (
    jti        UUID        PRIMARY KEY,
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX revoked_tokens_expires_at_idx ON revoked_tokens (expires_at);

-- ---------------------------------------------------------------------------
-- API keys
-- ---------------------------------------------------------------------------

CREATE TABLE api_keys (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id       UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id      UUID        NOT NULL REFERENCES users(id)         ON DELETE CASCADE,
    key_hash     TEXT        NOT NULL,
    description  TEXT,
    role         TEXT        NOT NULL CHECK (role IN ('owner','admin','operator','viewer')),
    last_used_at TIMESTAMPTZ,
    expires_at   TIMESTAMPTZ,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX api_keys_org_id_idx ON api_keys (org_id);

-- ---------------------------------------------------------------------------
-- Server secrets (JWT signing key, etc.)
-- ---------------------------------------------------------------------------

CREATE TABLE server_secrets (
    key        TEXT  PRIMARY KEY,
    value      BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ---------------------------------------------------------------------------
-- Installation codes (single-use enrollment tokens)
-- ---------------------------------------------------------------------------

CREATE TABLE installation_codes (
    code        TEXT        PRIMARY KEY,
    org_id      UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    created_by  UUID        NOT NULL REFERENCES users(id),
    used_at     TIMESTAMPTZ,
    expires_at  TIMESTAMPTZ NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX installation_codes_org_id_idx ON installation_codes (org_id);

-- ---------------------------------------------------------------------------
-- Devices
-- ---------------------------------------------------------------------------

CREATE TABLE devices (
    id             UUID        PRIMARY KEY,
    org_id         UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    display_name   TEXT        NOT NULL,
    firewall_kind  TEXT        NOT NULL DEFAULT 'none'
                               CHECK (firewall_kind IN ('pfsense','opnsense','nftables','iptables','none')),
    agent_version  TEXT,
    enrolled_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at   TIMESTAMPTZ,
    features       TEXT[]      NOT NULL DEFAULT '{}',
    config_digest  TEXT,
    notes          TEXT,
    system_info    JSONB                              -- populated on first Hello handshake
);

CREATE INDEX devices_org_id_idx ON devices (org_id);

CREATE TABLE device_permissions (
    user_id   UUID NOT NULL REFERENCES users(id)   ON DELETE CASCADE,
    device_id UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    access    TEXT NOT NULL CHECK (access IN ('allow','deny')),
    PRIMARY KEY (user_id, device_id)
);

-- ---------------------------------------------------------------------------
-- Device certificates
-- ---------------------------------------------------------------------------

CREATE TABLE device_certificates (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id  UUID        NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    cert_pem   TEXT        NOT NULL,
    issued_at  TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX device_certificates_device_id_idx ON device_certificates (device_id);

-- ---------------------------------------------------------------------------
-- Agent failures
-- ---------------------------------------------------------------------------

CREATE TABLE device_failures (
    failure_id  UUID        PRIMARY KEY,
    device_id   UUID        NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    severity    TEXT        NOT NULL CHECK (severity IN ('warning','error','fatal')),
    category    TEXT        NOT NULL,
    message     TEXT        NOT NULL,
    context     JSONB,
    occurred_at TIMESTAMPTZ NOT NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_replay   BOOLEAN     NOT NULL DEFAULT FALSE
);

CREATE INDEX device_failures_device_id_idx   ON device_failures (device_id);
CREATE INDEX device_failures_occurred_at_idx ON device_failures (occurred_at DESC);
CREATE INDEX device_failures_severity_idx    ON device_failures (severity);

-- ---------------------------------------------------------------------------
-- Tunnel sessions
-- ---------------------------------------------------------------------------

CREATE TABLE tunnel_sessions (
    id             UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id      UUID        NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    tunnel_type    TEXT        NOT NULL CHECK (tunnel_type IN ('ssh','tty','http','remote_desktop')),
    initiated_by   UUID        REFERENCES users(id),
    status         TEXT        NOT NULL DEFAULT 'active'
                               CHECK (status IN ('active','closed','abandoned')),
    started_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at       TIMESTAMPTZ,
    bytes_sent     BIGINT      NOT NULL DEFAULT 0,
    bytes_received BIGINT      NOT NULL DEFAULT 0
);

CREATE INDEX tunnel_sessions_device_id_idx  ON tunnel_sessions (device_id);
CREATE INDEX tunnel_sessions_started_at_idx ON tunnel_sessions (started_at DESC);

-- ---------------------------------------------------------------------------
-- Command log
-- ---------------------------------------------------------------------------

CREATE TABLE command_log (
    id             UUID        PRIMARY KEY,
    device_id      UUID        NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    command_type   TEXT        NOT NULL,
    initiated_by   UUID        REFERENCES users(id),
    status         TEXT        NOT NULL CHECK (status IN ('success','failure','timeout')),
    error_message  TEXT,
    applied_digest TEXT,
    sent_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    result_at      TIMESTAMPTZ
);

CREATE INDEX command_log_device_id_idx ON command_log (device_id);
CREATE INDEX command_log_sent_at_idx   ON command_log (sent_at DESC);

-- ---------------------------------------------------------------------------
-- Time-series: packet telemetry
-- ---------------------------------------------------------------------------

CREATE TABLE packets (
    time           TIMESTAMPTZ NOT NULL,
    device_id      UUID        NOT NULL,
    src_ip         INET,
    dst_ip         INET,
    src_port       INTEGER,
    dst_port       INTEGER,
    protocol       SMALLINT,
    bytes          INTEGER     NOT NULL,
    direction      TEXT        NOT NULL CHECK (direction IN ('in','out')),
    interface_name TEXT
);

SELECT create_hypertable('packets', 'time', chunk_time_interval => INTERVAL '1 hour');

CREATE INDEX ON packets (device_id, time DESC);

-- ---------------------------------------------------------------------------
-- Time-series: resource metrics
-- ---------------------------------------------------------------------------

CREATE TABLE resource_metrics (
    time             TIMESTAMPTZ NOT NULL,
    device_id        UUID        NOT NULL,
    cpu_percent      REAL,
    mem_used_bytes   BIGINT,
    mem_total_bytes  BIGINT,
    disk_used_bytes  BIGINT,
    disk_total_bytes BIGINT,
    load_1m          REAL,
    load_5m          REAL
);

SELECT create_hypertable('resource_metrics', 'time', chunk_time_interval => INTERVAL '4 hours');

CREATE INDEX ON resource_metrics (device_id, time DESC);

-- ---------------------------------------------------------------------------
-- Time-series: device monitoring status (from heartbeats)
-- ---------------------------------------------------------------------------

CREATE TABLE device_monitoring_status (
    time                  TIMESTAMPTZ NOT NULL,
    device_id             UUID        NOT NULL,
    packet_queue_depth    INTEGER,
    disk_buffer_bytes     BIGINT,
    disk_buffer_max_bytes BIGINT,
    packets_dropped_total BIGINT,
    packets_sent_total    BIGINT,
    degraded              BOOLEAN,
    active_tunnel_count   INTEGER
);

SELECT create_hypertable('device_monitoring_status', 'time', chunk_time_interval => INTERVAL '1 day');

CREATE INDEX ON device_monitoring_status (device_id, time DESC);

-- ---------------------------------------------------------------------------
-- Continuous aggregates
-- ---------------------------------------------------------------------------

CREATE MATERIALIZED VIEW packets_5m
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('5 minutes', time) AS bucket,
    device_id,
    SUM(bytes) AS total_bytes,
    COUNT(*)   AS packet_count
FROM packets
GROUP BY bucket, device_id
WITH NO DATA;

SELECT add_continuous_aggregate_policy(
    'packets_5m',
    start_offset      => INTERVAL '1 hour',
    end_offset        => INTERVAL '1 minute',
    schedule_interval => INTERVAL '5 minutes'
);

-- ---------------------------------------------------------------------------
-- Retention policies
-- ---------------------------------------------------------------------------

SELECT add_retention_policy('packets',                  INTERVAL '30 days');
SELECT add_retention_policy('resource_metrics',         INTERVAL '90 days');
SELECT add_retention_policy('device_monitoring_status', INTERVAL '90 days');

-- ---------------------------------------------------------------------------
-- Row-level security (multi-tenancy defence-in-depth)
-- ---------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION current_org_id() RETURNS UUID
    LANGUAGE sql STABLE
AS $$
    SELECT NULLIF(current_setting('app.current_org_id', true), '')::uuid
$$;

ALTER TABLE devices             ENABLE ROW LEVEL SECURITY;
ALTER TABLE users               ENABLE ROW LEVEL SECURITY;
ALTER TABLE api_keys            ENABLE ROW LEVEL SECURITY;
ALTER TABLE installation_codes  ENABLE ROW LEVEL SECURITY;
ALTER TABLE device_failures     ENABLE ROW LEVEL SECURITY;
ALTER TABLE tunnel_sessions     ENABLE ROW LEVEL SECURITY;
ALTER TABLE command_log         ENABLE ROW LEVEL SECURITY;

CREATE POLICY org_isolation ON devices
    USING (current_org_id() IS NULL OR org_id = current_org_id());

CREATE POLICY org_isolation ON users
    USING (current_org_id() IS NULL OR org_id = current_org_id());

CREATE POLICY org_isolation ON api_keys
    USING (current_org_id() IS NULL OR org_id = current_org_id());

CREATE POLICY org_isolation ON installation_codes
    USING (current_org_id() IS NULL OR org_id = current_org_id());

CREATE POLICY org_isolation ON device_failures
    USING (
        current_org_id() IS NULL
        OR device_id IN (SELECT id FROM devices WHERE org_id = current_org_id())
    );

CREATE POLICY org_isolation ON tunnel_sessions
    USING (
        current_org_id() IS NULL
        OR device_id IN (SELECT id FROM devices WHERE org_id = current_org_id())
    );

CREATE POLICY org_isolation ON command_log
    USING (
        current_org_id() IS NULL
        OR device_id IN (SELECT id FROM devices WHERE org_id = current_org_id())
    );
