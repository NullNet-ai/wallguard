-- Migration 001 — Initial relational schema.
--
-- Creates the TimescaleDB extension (required before 002 can create hypertables)
-- and all relational tables.
--
-- firewall_rules and config_snapshots are deferred to migration 006 (Phase 12).

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
    password_hash TEXT        NOT NULL,   -- argon2id
    display_name  TEXT        NOT NULL,
    role          TEXT        NOT NULL    CHECK (role IN ('owner','admin','operator','viewer')),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (org_id, email)
);

CREATE INDEX users_org_id_idx ON users (org_id);

-- ---------------------------------------------------------------------------
-- Auth tokens
-- ---------------------------------------------------------------------------

-- JWT refresh tokens (access tokens are stateless; only refresh tokens stored).
CREATE TABLE refresh_tokens (
    jti        UUID        PRIMARY KEY,
    user_id    UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX refresh_tokens_user_id_idx ON refresh_tokens (user_id);

-- Revoked access-token JTIs, checked on every request.
-- Rows are GC'd after expires_at so the table stays bounded.
CREATE TABLE revoked_tokens (
    jti        UUID        PRIMARY KEY,
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX revoked_tokens_expires_at_idx ON revoked_tokens (expires_at);

-- ---------------------------------------------------------------------------
-- API keys (long-lived programmatic access)
-- ---------------------------------------------------------------------------

CREATE TABLE api_keys (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id       UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id      UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key_hash     TEXT        NOT NULL,   -- argon2id of the raw key
    description  TEXT,
    role         TEXT        NOT NULL    CHECK (role IN ('owner','admin','operator','viewer')),
    last_used_at TIMESTAMPTZ,
    expires_at   TIMESTAMPTZ,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX api_keys_org_id_idx ON api_keys (org_id);

-- ---------------------------------------------------------------------------
-- Server-level secrets (JWT signing key, etc.)
-- ---------------------------------------------------------------------------

CREATE TABLE server_secrets (
    key        TEXT        PRIMARY KEY,
    value      BYTEA       NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ---------------------------------------------------------------------------
-- Device installation codes (single-use enrollment tokens)
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
    id             UUID        PRIMARY KEY,         -- matches CN in device cert
    org_id         UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    display_name   TEXT        NOT NULL,
    firewall_kind  TEXT        NOT NULL DEFAULT 'none'
                               CHECK (firewall_kind IN ('pfsense','opnsense','nftables','none')),
    agent_version  TEXT,
    enrolled_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at   TIMESTAMPTZ,
    features       TEXT[]      NOT NULL DEFAULT '{}',  -- negotiated on last Hello
    config_digest  TEXT,                               -- SHA-256 of last applied config
    notes          TEXT
);

CREATE INDEX devices_org_id_idx ON devices (org_id);

-- Per-device permission overrides (supplements org-level role).
CREATE TABLE device_permissions (
    user_id   UUID NOT NULL REFERENCES users(id)   ON DELETE CASCADE,
    device_id UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    access    TEXT NOT NULL CHECK (access IN ('allow','deny')),
    PRIMARY KEY (user_id, device_id)
);

-- ---------------------------------------------------------------------------
-- Device certificates (audit trail for cert rotation)
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
    failure_id  UUID        PRIMARY KEY,   -- from AgentFailure.failure_id; used for dedup
    device_id   UUID        NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    severity    TEXT        NOT NULL CHECK (severity IN ('warning','error','fatal')),
    category    TEXT        NOT NULL,
    message     TEXT        NOT NULL,
    context     JSONB,
    occurred_at TIMESTAMPTZ NOT NULL,      -- time of occurrence on the agent
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_replay   BOOLEAN     NOT NULL DEFAULT FALSE
);

CREATE INDEX device_failures_device_id_idx       ON device_failures (device_id);
CREATE INDEX device_failures_occurred_at_idx     ON device_failures (occurred_at DESC);
CREATE INDEX device_failures_severity_idx        ON device_failures (severity);

-- ---------------------------------------------------------------------------
-- Tunnel session audit log
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

CREATE INDEX tunnel_sessions_device_id_idx ON tunnel_sessions (device_id);
CREATE INDEX tunnel_sessions_started_at_idx ON tunnel_sessions (started_at DESC);

-- ---------------------------------------------------------------------------
-- Command audit log
-- ---------------------------------------------------------------------------

CREATE TABLE command_log (
    id             UUID        PRIMARY KEY,   -- = command_id
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
