#!/usr/bin/env bash
# Generate a three-tier PKI for local development.
#
# Output (written to $OUT, default: dev-certs/):
#   root-ca.key / root-ca.crt      — Root CA  (offline in production; used only to sign Intermediate CA)
#   ca.key / ca.crt                — Intermediate CA  (agents pin ca.crt)
#   server.key / server.crt        — Server leaf cert  (gRPC + QUIC TLS)
#
# The dev-certs/ directory is git-ignored.
# Production cert signing uses rcgen inside wg-server (Phase 3/4).
#
# Requirements: openssl ≥ 1.1.1

set -euo pipefail

OUT="${1:-dev-certs}"
mkdir -p "$OUT"

log() { echo "[dev-certs] $*"; }

# ---------------------------------------------------------------------------
# Root CA  (4096-bit RSA, 10-year validity)
# ---------------------------------------------------------------------------
log "Generating Root CA..."
openssl genrsa -out "$OUT/root-ca.key" 4096 2>/dev/null
openssl req -new -x509 \
    -days 3650 \
    -key "$OUT/root-ca.key" \
    -subj "/CN=WallGuard Dev Root CA/O=WallGuard Dev/C=CA" \
    -out "$OUT/root-ca.crt"

# ---------------------------------------------------------------------------
# Intermediate CA  (4096-bit RSA, 5-year validity, signed by Root CA)
# Agents pin this cert, not the Root CA.
# ---------------------------------------------------------------------------
log "Generating Intermediate CA..."
openssl genrsa -out "$OUT/ca.key" 4096 2>/dev/null

openssl req -new \
    -key "$OUT/ca.key" \
    -subj "/CN=WallGuard Dev Intermediate CA/O=WallGuard Dev/C=CA" \
    -out "$OUT/ca.csr"

openssl x509 -req \
    -days 1825 \
    -in "$OUT/ca.csr" \
    -CA "$OUT/root-ca.crt" \
    -CAkey "$OUT/root-ca.key" \
    -CAcreateserial \
    -extfile <(printf 'basicConstraints=critical,CA:TRUE,pathlen:0\nkeyUsage=critical,keyCertSign,cRLSign') \
    -out "$OUT/ca.crt" 2>/dev/null

# ---------------------------------------------------------------------------
# Server leaf cert  (ECDSA P-256, 1-year validity, signed by Intermediate CA)
# SANs cover localhost and the docker-compose service name.
# ---------------------------------------------------------------------------
log "Generating server leaf cert..."
openssl ecparam -name prime256v1 -genkey -noout -out "$OUT/server.key" 2>/dev/null

openssl req -new \
    -key "$OUT/server.key" \
    -subj "/CN=wallguard-server/O=WallGuard Dev/C=CA" \
    -out "$OUT/server.csr"

openssl x509 -req \
    -days 365 \
    -in "$OUT/server.csr" \
    -CA "$OUT/ca.crt" \
    -CAkey "$OUT/ca.key" \
    -CAcreateserial \
    -extfile <(printf 'subjectAltName=DNS:localhost,DNS:wallguard-server,IP:127.0.0.1\nkeyUsage=critical,digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth') \
    -out "$OUT/server.crt" 2>/dev/null

# Clean up CSRs
rm -f "$OUT"/*.csr "$OUT"/*.srl

log ""
log "Done. Files written to $OUT/:"
log "  root-ca.crt     — Root CA (keep offline in production)"
log "  ca.crt          — Intermediate CA  ← agents pin this"
log "  ca.key          — Intermediate CA private key (mount into server container)"
log "  server.crt      — Server TLS cert"
log "  server.key      — Server TLS private key"
log ""
log "Mount into docker-compose:"
log "  volumes:"
log "    - ./dev-certs:/etc/wallguard-server:ro"
