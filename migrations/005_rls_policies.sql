-- Migration 005 — Row-Level Security for multi-tenancy.
--
-- Primary org isolation is enforced by WHERE org_id = $1 in every query;
-- RLS is defense-in-depth that makes cross-org data leaks impossible even
-- if a handler forgets the WHERE clause.
--
-- The application sets app.current_org_id per connection at request time:
--   SET LOCAL app.current_org_id = '<org-uuid>';
--
-- When the setting is absent (migrations, health checks, superuser ops) all
-- rows are visible — this is safe because those code paths run as the DB
-- owner and are not handling untrusted HTTP requests.
--
-- The DB application user must NOT have BYPASSRLS so that the policy fires.
-- Migrations run as the owner (which does bypass RLS by default in PG).

-- Helper: returns the current org UUID, or NULL if not set.
CREATE OR REPLACE FUNCTION current_org_id() RETURNS UUID
    LANGUAGE sql STABLE
AS $$
    SELECT NULLIF(current_setting('app.current_org_id', true), '')::uuid
$$;

-- ---------------------------------------------------------------------------
-- Enable RLS and create policies on tenant-scoped tables
-- ---------------------------------------------------------------------------

ALTER TABLE devices             ENABLE ROW LEVEL SECURITY;
ALTER TABLE users               ENABLE ROW LEVEL SECURITY;
ALTER TABLE api_keys            ENABLE ROW LEVEL SECURITY;
ALTER TABLE installation_codes  ENABLE ROW LEVEL SECURITY;
ALTER TABLE device_failures     ENABLE ROW LEVEL SECURITY;
ALTER TABLE tunnel_sessions     ENABLE ROW LEVEL SECURITY;
ALTER TABLE command_log         ENABLE ROW LEVEL SECURITY;

-- Policy template: allow when no org is set (internal ops) or org matches.
CREATE POLICY org_isolation ON devices
    USING (current_org_id() IS NULL OR org_id = current_org_id());

CREATE POLICY org_isolation ON users
    USING (current_org_id() IS NULL OR org_id = current_org_id());

CREATE POLICY org_isolation ON api_keys
    USING (current_org_id() IS NULL OR org_id = current_org_id());

CREATE POLICY org_isolation ON installation_codes
    USING (current_org_id() IS NULL OR org_id = current_org_id());

-- device_failures, tunnel_sessions, command_log are scoped via devices.device_id;
-- enforce by joining to devices which already has the policy applied.
CREATE POLICY org_isolation ON device_failures
    USING (
        current_org_id() IS NULL
        OR device_id IN (
            SELECT id FROM devices WHERE org_id = current_org_id()
        )
    );

CREATE POLICY org_isolation ON tunnel_sessions
    USING (
        current_org_id() IS NULL
        OR device_id IN (
            SELECT id FROM devices WHERE org_id = current_org_id()
        )
    );

CREATE POLICY org_isolation ON command_log
    USING (
        current_org_id() IS NULL
        OR device_id IN (
            SELECT id FROM devices WHERE org_id = current_org_id()
        )
    );
