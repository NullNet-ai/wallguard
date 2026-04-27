# WallGuard

Remote management and monitoring platform for network firewall appliances.
Agents run on devices (pfSense, OPNsense, nftables hosts) and maintain a
persistent mTLS control channel back to the server. The web UI, REST API,
and CLI are the three operator-facing surfaces.

---

## Building and running locally

### Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Rust toolchain | ≥ 1.85 | [rustup.rs](https://rustup.rs) |
| `wasm32-unknown-unknown` target | — | `rustup target add wasm32-unknown-unknown` |
| `trunk` (WASM bundler) | latest | `cargo install trunk --locked` |
| Docker + Compose | — | for TimescaleDB (Option A) or Postgres only (Option B) |

### Option A — Docker Compose (quickest path)

Runs TimescaleDB, wg-server (with embedded UI), Prometheus, and Grafana.
On first start the server auto-generates dev TLS certificates and seeds an
initial admin user — no manual setup required.

```bash
sudo docker compose up
```

The server is reachable at:
- Web UI / API — http://localhost:4444
- Agent provisioning gRPC — localhost:50051
- Agent control gRPC (mTLS) — localhost:50052
- QUIC reverse tunnel — localhost:7777 (UDP)
- Prometheus metrics — http://localhost:9090
- Grafana — http://localhost:3000  (admin / dev_password)

### Option B — bare-metal local dev

Use this when you are iterating on the server or agent and need faster
recompile cycles.

**Step 1 — Start Postgres only**

```bash
docker compose up -d postgres
```

**Step 2 — Build the web UI**

The server embeds the WASM bundle at compile time via `rust-embed`.  Build it
once before building the server; rebuild whenever the UI changes.

```bash
cd crates/wg-ui
trunk build           # or: trunk build --release
cd -
```

**Step 3 — Run wg-server**

Migrations run automatically on startup.  Dev TLS certificates are generated
and written to `dev-certs/` on first run if they do not already exist.
An initial admin user is seeded if the database is empty.

```bash
DATABASE_URL=postgres://wallguard:dev_password@localhost:5432/wallguard \
CA_CERT_PATH=dev-certs/ca.crt \
CA_KEY_PATH=dev-certs/ca.key \
SERVER_CERT_PATH=dev-certs/server.crt \
SERVER_KEY_PATH=dev-certs/server.key \
RUST_LOG=info,wg_server=debug \
cargo run -p wg-server
```

The server prints the seeded credentials to the log on first run:
```
Email    : admin@wallguard.local
Password : password123
```

**Step 4 — Enroll a device**

Log in to the web UI, create an installation code, then on the device:

```bash
sudo cargo run -p wg-cli -- enroll \
    --server grpc://localhost:50051 \
    --install-code <CODE>
```

The agent config is written to `/etc/wallguard/config.toml` and device
certificates are written to `/etc/wallguard/`.

**Step 5 — Run the agent**

```bash
sudo cargo run -p wg-agent
# or with a custom config:
sudo cargo run -p wg-agent -- --config /etc/wallguard/config.toml
```

### wg-server environment variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | yes | — | PostgreSQL connection string |
| `CA_CERT_PATH` | yes | — | Intermediate CA certificate (PEM) — agents pin this |
| `CA_KEY_PATH` | yes | — | Intermediate CA private key (PEM) — used to sign device certs |
| `SERVER_CERT_PATH` | yes | — | Server TLS leaf certificate (PEM) |
| `SERVER_KEY_PATH` | yes | — | Server TLS private key (PEM) |
| `HTTP_PORT` | no | `8080` | HTTP API + embedded web UI |
| `GRPC_PORT` | no | `50051` | Device provisioning gRPC (plain TLS) |
| `CONTROL_GRPC_PORT` | no | `50052` | Agent control channel gRPC (mTLS) |
| `QUIC_PORT` | no | `7777` | QUIC reverse-tunnel listener (UDP) |
| `TCP_TLS_PORT` | no | `7778` | TCP-TLS reverse-tunnel fallback |
| `METRICS_PORT` | no | `9090` | Prometheus metrics endpoint |
| `LOG_FORMAT` | no | `pretty` | `pretty` or `json` |
| `RUST_LOG` | no | `info` | Tracing filter (e.g. `info,wg_server=debug`) |
| `OTLP_ENDPOINT` | no | — | OTLP collector URL; omit to disable trace export |
