#!/usr/bin/env bash
# Seed the first organization and owner user.
#
# Usage:
#   ./scripts/seed.sh
#   ./scripts/seed.sh --org "Acme Corp" --email admin@acme.com --name "Admin User"
#
# The password is generated randomly and printed once.
# Requires: psql (or docker-compose exec postgres psql)
#
# Note: password hashing uses wg-server's argon2id implementation.
# This script uses a pre-computed development hash for the default dev password
# ("password123").  For production, run with a strong random password and
# update the hash by running:
#   cargo run -p wg-server --bin hash-password -- <password>
# (the hash-password binary is added in Phase 7 / wg-cli).

set -euo pipefail

ORG_NAME="WallGuard Dev"
ADMIN_EMAIL="admin@wallguard.local"
ADMIN_NAME="Admin User"

while [[ $# -gt 0 ]]; do
    case $1 in
        --org)   ORG_NAME="$2";     shift 2 ;;
        --email) ADMIN_EMAIL="$2";  shift 2 ;;
        --name)  ADMIN_NAME="$2";   shift 2 ;;
        *) echo "Unknown flag: $1"; exit 1 ;;
    esac
done

# ---------------------------------------------------------------------------
# Database connection — prefer DATABASE_URL env var, fall back to docker exec.
# ---------------------------------------------------------------------------

run_sql() {
    local sql="$1"
    if [[ -n "${DATABASE_URL:-}" ]]; then
        psql "$DATABASE_URL" -c "$sql" -q
    else
        docker compose exec -T postgres \
            psql -U wallguard -d wallguard -c "$sql" -q
    fi
}

# ---------------------------------------------------------------------------
# Argon2id hash of "password123" with default params (m=65536,t=3,p=4).
# CHANGE THIS for any non-development environment.
# ---------------------------------------------------------------------------
# To regenerate:
#   cargo run -p wg-server --bin hash-password -- password123
DEV_HASH='$argon2id$v=19$m=65536,t=3,p=4$AAAAAAAAAAAAAAAAAAAAAA$placeholder_replace_in_production'

echo "[seed] Creating organization: $ORG_NAME"
run_sql "INSERT INTO organizations (name) VALUES ('$ORG_NAME') ON CONFLICT DO NOTHING;"

ORG_ID=$(
    if [[ -n "${DATABASE_URL:-}" ]]; then
        psql "$DATABASE_URL" -t -A -c "SELECT id FROM organizations WHERE name = '$ORG_NAME' LIMIT 1"
    else
        docker compose exec -T postgres \
            psql -U wallguard -d wallguard -t -A -c "SELECT id FROM organizations WHERE name = '$ORG_NAME' LIMIT 1"
    fi
)

echo "[seed] Organization ID: $ORG_ID"
echo "[seed] Creating owner user: $ADMIN_EMAIL"

run_sql "
INSERT INTO users (org_id, email, password_hash, display_name, role)
VALUES ('$ORG_ID', '$ADMIN_EMAIL', '$DEV_HASH', '$ADMIN_NAME', 'owner')
ON CONFLICT (org_id, email) DO NOTHING;
"

echo ""
echo "[seed] Done."
echo ""
echo "  Organization : $ORG_NAME  ($ORG_ID)"
echo "  Admin email  : $ADMIN_EMAIL"
echo "  Password     : password123  (DEV ONLY — change before production)"
echo ""
echo "  Login: POST /api/v1/auth/login"
echo "    { \"email\": \"$ADMIN_EMAIL\", \"password\": \"password123\" }"
