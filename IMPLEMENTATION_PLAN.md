# WallGuard — Implementation Plan

Based on `FINAL_DESIGN.md`. Ordered so the system is enrollable, accessible, and tunnable before any firewall management is introduced.

> **Remote Desktop / FreeBSD:** The design excludes FreeBSD from remote desktop at compile time (`#[cfg(not(target_os = "freebsd"))]`). Per user requirement this is changed: the `remote-desktop` feature compiles on **all** platforms including FreeBSD, and capability advertisement is decided at **runtime** — the agent calls a capture-backend probe at startup; if it fails (headless appliance, no display server) `Feature::RemoteDesktop` is simply not advertised. This lets pfSense/OPNsense installs with an active X display use remote desktop without a separate build.

---

## Phase 0 — Workspace Skeleton

- [ ] `Cargo.toml` workspace root: all member crates, shared `[profile.*]`, `panic = "abort"` for `test-abort` profile
- [ ] `crates/wg-shared/Cargo.toml` — no I/O, no tokio, compiles to both native and `wasm32`
- [ ] `crates/wg-agent/Cargo.toml` — all agent deps; `[features] remote-desktop = ["dep:captis", "dep:openh264", "dep:enigo"]`
- [ ] `crates/wg-cli/Cargo.toml`
- [ ] `crates/wg-server/Cargo.toml`
- [ ] `crates/wg-ui/Cargo.toml` — Leptos + wasm32 target
- [ ] `crates/wg-testkit/Cargo.toml`
- [ ] `proto/` — five `.proto` files: `provisioning.proto`, `control.proto`, `data.proto`, `models.proto`, `cli.proto` (stubs; firewall message types added in Phase 10)
- [ ] `build.rs` in each crate that generates protobuf code via `tonic-build`
- [ ] `migrations/` directory created; migration files added per phase
- [ ] `.github/workflows/ci.yml` skeleton (fmt, clippy, test, buf, trunk, docker build)
- [ ] `docker-compose.yml` (TimescaleDB, server, prometheus, grafana)
- [ ] `Dockerfile.server` and `Dockerfile.ui` multi-stage build stubs

---

## Phase 1 — `wg-shared`: Foundation Types

- [ ] `wg-shared/src/types.rs` — `Device`, `DeviceStatus`, `TunnelSession`, `AgentFailure`, `Organization`, `User`; `serde` derives; compiles to both native and wasm32
- [ ] `wg-shared/src/pki.rs` — `parse_device_id_from_cert(pem)` helper; `write_secret_file(path, data)` (mode 0600 set at `open()` time)
- [ ] `wg-shared/src/capabilities.rs` — `FirewallKind` enum, `Feature` enum, `derive_capabilities()` skeleton (firewall-specific entries added in Phase 10)
- [ ] Protobuf schema: `control.proto` — `Hello`, `Welcome`, `VersionRejected`, `DeviceSettings`, `Heartbeat`, `HeartbeatAck`, `MonitoringStatus`, `AgentFailure`, `CommandResult`, `SetMonitoring`, `ThrottleMonitoring`, tunnel-open commands (`OpenSshTunnel`, `OpenTtyTunnel`, `OpenHttpTunnel`, `OpenRemoteDesktopTunnel`), `ShutdownImminent`, `RenewCertificateRequest/Response`; firewall commands stubbed as reserved fields
- [ ] Protobuf schema: `provisioning.proto` — `EnrollRequest` / `EnrollResponse`
- [ ] Protobuf schema: `data.proto` — telemetry batch RPCs
- [ ] Protobuf schema: `cli.proto` — `StatusRequest/Response`, `GracefulRestart`

---

## Phase 2 — Infrastructure & Dev Environment

- [ ] `scripts/dev-certs.sh` — `rcgen`-based script generating dev Root CA, Intermediate CA, and server cert; output to `dev-certs/`
- [ ] `docker-compose.yml` filled in: TimescaleDB (`timescale/timescaledb:latest-pg16`), `wg-server`, Prometheus, Grafana with correct ports and volume mounts
- [ ] `migrations/001_initial_schema.sql` — all relational tables from §11.2 **except** `firewall_rules` and `config_snapshots` (deferred to Phase 10)
- [ ] `migrations/002_timescale_hypertables.sql` — `CREATE EXTENSION IF NOT EXISTS timescaledb`; hypertables for `packets`, `resource_metrics`, `device_monitoring_status`
- [ ] `migrations/003_timescale_aggregates.sql` — `packets_5m` continuous aggregate
- [ ] `migrations/004_retention_policies.sql` — retention policies (packets 30d, resource_metrics 90d, monitoring status 90d)
- [ ] `migrations/005_rls_policies.sql` — Row-Level Security `org_id` policies
- [ ] `db/pool.rs` in `wg-server` — `PgPool` from `DATABASE_URL`; run `sqlx::migrate!` at startup; exit on failure
- [ ] `Dockerfile.server` complete multi-stage build; `Dockerfile.ui` WASM build stage
- [ ] `scripts/seed.sh` — create first org + owner user

---

## Phase 3 — Security Foundation

### 3a — PKI / CA
- [ ] `pki/ca.rs` — load Intermediate CA cert+key; `sign_csr(csr_pem) -> (cert_pem, device_id)`; validate `CN=device:<uuid>` and `O=org:<org_id>` format

### 3b — Auth
- [ ] `auth/password.rs` — `hash_password` / `verify_password` (argon2id, m=64MiB, t=3, p=4)
- [ ] `auth/jwt.rs` — `issue_jwt` / `validate_jwt`; load/generate signing key from `server_secrets` table; check `revoked_tokens` on every validation
- [ ] `auth/refresh.rs` — issue + rotate refresh tokens; `POST /api/v1/auth/refresh`
- [ ] `auth/api_key.rs` — generate raw key; store `argon2(key)` in `api_keys`; validate on request
- [ ] Unit tests: valid JWT accepted; expired rejected; revoked rejected; role mismatch rejected

### 3c — RBAC Middleware
- [ ] `middleware/auth.rs` — extract JWT or API key; attach `RequestContext { user_id, org_id, role }`
- [ ] `middleware/rbac.rs` — `ctx.require_role(Role)` helper used by handlers
- [ ] `middleware/request_id.rs` — inject `X-Request-Id`; propagate to all log spans

---

## Phase 4 — Device Provisioning

- [ ] `grpc/provisioning.rs` — `Provisioning` gRPC service on `:50051`; no client cert required; validate + atomically mark installation code used (transaction); sign CSR via PKI; create `Device` row; write audit log entry
- [ ] `wg-cli enroll` — generate Ed25519 keypair; build CSR (`CN=device:<uuid>`, `O=org:<pending>`); connect to Provisioning service (server cert vs pinned CA); send `EnrollRequest`; write `device.key` (0600), `device.crt` (0644), `ca.crt` (0644), `config.toml`
- [ ] `installation_codes` API — `POST /api/v1/installation-codes`, `GET /api/v1/installation-codes` (admin+)
- [ ] Integration test: `enrollment_flow` — CSR signing, cert file permissions, agent connects with issued cert

---

## Phase 5 — Agent Core

### 5a — Config & State Machine
- [ ] `config.rs` — parse `/etc/wallguard/config.toml`; all fields from §16 (server URL, TLS paths, tunnel addresses, transmission settings, observability)
- [ ] `state.rs` — `DaemonState` enum (`Provisioning`, `Idle`, `Connecting`, `Connected`); typed transitions; no `Error` state
- [ ] `backoff.rs` — `Backoff`: base 1s, max 300s, multiplier 2.0, ±20% jitter; proptest fuzz for delay distribution

### 5b — Platform & Capability Detection
- [ ] `platform.rs` — `TARGET_OS` compile-time const (`Linux`, `FreeBsd`, `Windows`)
- [ ] `capabilities.rs` — `derive_capabilities()`: base feature set; remote desktop runtime probe:
  - [ ] Attempt `captis::Display::open()` (or equivalent backend init)
  - [ ] `Ok` → push `Feature::RemoteDesktop`
  - [ ] `Err` → log `info "remote desktop unavailable: {reason}"`; do not push feature
  - [ ] No compile-time `#[cfg(not(target_os = "freebsd"))]` exclusion

### 5c — Failure Reporting
- [ ] `failure_buffer.rs` — `FailureBuffer`: `append` (sync + async), `read_all`, `trim_delivered`; 500-entry cap ring rotation; `fsync` on FATAL only
- [ ] `panic_hook.rs` — sync-safe panic hook; writes FATAL `AgentFailure` to buffer; `eprintln!` fallback
- [ ] Unit tests: cap at exactly 500; trim_delivered; never panics on corrupt file

### 5d — Agent `main.rs`
- [ ] Parse `--firewall` arg (default `none`); load config; install panic hook; run state machine loop
- [ ] `Connecting` → connect gRPC (backoff) → open QUIC tunnel connection (backoff) → send `Hello` / receive `Welcome` → `Connected`
- [ ] `VersionRejected` → log + go to `Idle`; do not retry
- [ ] On `Connected`: replay buffered failures → drain disk buffer → start heartbeat task → start monitoring (if enabled)
- [ ] Heartbeat task: send every 10s with `MonitoringStatus`; 3 consecutive missed acks → reconnect
- [ ] CLI Unix socket: `UnixListener` on `/run/wallguard/agent.sock`; mode `0600`; serve `cli.proto` gRPC

---

## Phase 6 — Server Control Channel

- [ ] `grpc/control.rs` — bidirectional streaming; extract `device_id` from TLS peer cert CN; call `on_new_connection()`
- [ ] `connection_registry.rs` — `ConnectionMap: Arc<RwLock<HashMap<DeviceId, DeviceConnection>>>`; on duplicate `DeviceId` → signal stale connection shutdown → replace immediately
- [ ] `command_tracker.rs` — pending command map keyed by `command_id`; background sweeper every 5s sends `TIMEOUT` to commands older than 30s; resolves to HTTP 200 / 422 / 504 correctly
- [ ] `heartbeat.rs` — server sends heartbeats every 10s; per-device 3-miss threshold; update `MonitoringStatus` in registry; write to `device_monitoring_status` (throttled once/min)
- [ ] Graceful shutdown: on `SIGTERM` → broadcast `ShutdownImminent { reconnect_after_ms: 3000 }` → wait ≤5s for tunnel close → send `TIMEOUT` to all pending commands → exit

---

## Phase 7 — Data Transmission Pipeline

- [ ] `pipeline/capture.rs` — libpcap capture task via `nullnet-traffic-monitor`; bounded channel cap=50_000; drop + metric on full
- [ ] `pipeline/batch.rs` — accumulate until 1_000 packets or 500ms; apply `Arc<AtomicU32>` sampling rate from `ThrottleMonitoring`
- [ ] `pipeline/transmit.rs` — send batch via Data gRPC; on failure → `DiskBuffer`; on reconnect → drain buffer before live data resumes
- [ ] `disk_buffer.rs` — `try_write()`: check available space; 256 MiB cap; 512 MiB minimum free guard; `{id:016x}.bin` filenames; never panics
- [ ] `grpc/data.rs` (server) — receive batches; bulk-insert into `packets` hypertable; emit counter metrics
- [ ] Unit tests: `DiskBuffer` write/full/low-disk/never-panics; pipeline backpressure

---

## Phase 8 — Tunnel Sessions

### 8a — QUIC / TCP Endpoint
- [ ] `tunnel/quic.rs` (server) — `quinn` endpoint on `:7777` UDP; mTLS device cert; `max_concurrent_bidi_streams=64`, `max_idle_timeout=60s`, `keep_alive_interval=15s`, `datagram_receive_buffer_size=4MiB`
- [ ] TCP fallback endpoint on `:7778` TLS; same mTLS config
- [ ] Agent: try QUIC first (3s timeout); fall back to TCP; record preference in `config.toml`
- [ ] `tunnel/stream_router.rs` — read `TunnelHello { tunnel_id }` from each new stream; dispatch to registered handler

### 8b — SSH Tunnel
- [ ] Server: on `OpenSshTunnel` → register handler → wait for QUIC stream with matching `TunnelHello`; hand to `russh` server; relay to WebSocket endpoint
- [ ] Agent: send `CommandResult` immediately (before opening stream) → open QUIC stream → send `TunnelHello` → accept SSH connection
- [ ] `tunnel_sessions` row created on open; updated on close (bytes, `ended_at`)
- [ ] Idle timeout: 30 minutes of no WebSocket frames

### 8c — TTY Tunnel
- [ ] Server: same transport as SSH; relay raw bytes to WebSocket
- [ ] Agent: spawn PTY via `portable-pty`; pipe to QUIC stream
- [ ] Disable per-device by removing `Feature::TtyTunnel` from DB negotiated set

### 8d — HTTP Proxy Tunnel
- [ ] Server: on `OpenHttpTunnel` → open QUIC stream → lightweight `hyper`-based proxy (no `pingora`); strip tunnel path prefix; rewrite `Host` header
- [ ] Agent: accept bytes from stream; forward to `target_host:target_port`

### 8e — Remote Desktop Tunnel
- [ ] Server:
  - [ ] Control stream (bidi reliable): session setup, resize, PLI requests
  - [ ] Input stream (uni reliable, server→agent): keyboard/mouse events
  - [ ] Datagram receiver: H.264 NAL units; track `seq` per tunnel; gap → PLI on control stream
  - [ ] Congestion: loss >5% or jitter >80ms over 500ms → `ThrottleRemoteDesktop`; bitrate floor 256kbps; ramp 10%/s on recovery
  - [ ] Forward NAL units to WebSocket at `WS /api/v1/devices/{id}/desktop/{session_id}`
- [ ] Agent (compiled under `remote-desktop` feature):
  - [ ] Runtime check: if probe failed at startup → `CommandResult::FAILURE` with clear message; no crash
  - [ ] Screen capture loop via `captis`; H.264 encode via `openh264`; fragment to `min(MTU−40, 1200)` bytes; send as QUIC datagrams in `VideoFrameDatagram` format
  - [ ] On PLI → force IDR keyframe within next encode cycle
  - [ ] Input events: decode from input stream; inject via `enigo`
  - [ ] **FreeBSD**: verify `captis` X11 backend works; document any gaps; runtime probe must correctly return `Err` on headless systems
- [ ] TCP fallback: remote desktop returns `CommandResult::FAILURE` with reason "datagrams unavailable on TCP fallback"

---

## Phase 9 — HTTP API & Web UI

### 9a — HTTP API
- [ ] `api/router.rs` — `axum::Router`; `AuthMiddleware` + `RbacMiddleware` on all non-auth routes; error format `{ error: { code, message, request_id } }`
- [ ] `api/auth.rs` — `POST /login`, `POST /refresh`, `POST /logout`
- [ ] `api/devices.rs` — `GET /devices`, `GET /devices/{id}`, `GET /devices/{id}/status`
- [ ] `api/tunnels.rs` — `POST /devices/{id}/tunnels/ssh`, `.../tty`, `.../http`, `.../desktop`
- [ ] `api/failures.rs` — `GET /devices/{id}/failures`
- [ ] `api/users.rs` — `GET/POST/DELETE /users` (admin+)
- [ ] `api/sse.rs` — `GET /api/v1/events` SSE stream; events: `device_status`, `device_connected`, `device_disconnected`, `new_failure`
- [ ] WebSocket endpoints: `WS /api/v1/devices/{id}/ssh/{session_id}`, `.../tty/{session_id}`, `.../desktop/{session_id}`
- [ ] `GET /metrics` on `:9090` — Prometheus export
- [ ] Serve `wg-ui` WASM via `rust-embed`; `Cache-Control: immutable` on hashed assets; `no-cache` on `index.html`

### 9b — `wg-ui`
- [ ] `Trunk.toml` and `index.html`
- [ ] `app.rs` — root component; `leptos_router` for all pages
- [ ] `api/` — type-safe client modules (`auth.rs`, `devices.rs`, `tunnels.rs`, `failures.rs`, `users.rs`) using `reqwest` wasm + `wg-shared` types; JWT stored in `localStorage`
- [ ] `pages/login.rs`
- [ ] `pages/dashboard.rs` — connected/degraded counts, recent failures, active tunnels, bytes/s chart (SSE-driven)
- [ ] `pages/devices/list.rs` — live status via SSE
- [ ] `pages/devices/detail.rs` — CPU/memory/disk charts, packet rate chart, active tunnels, recent failures, quick-launch tunnel buttons
- [ ] `pages/devices/tunnels.rs` — active sessions
- [ ] `pages/devices/failures.rs` — failure log with severity filter
- [ ] `components/terminal.rs` — `xterm.js` WebSocket wrapper for SSH/TTY
- [ ] `components/remote_desktop.rs` — WebSocket + WebCodecs H.264 decoder
- [ ] `components/packet_chart.rs`, `device_card.rs`, `status_badge.rs`
- [ ] `pages/settings.rs`, `pages/settings/users.rs`
- [ ] `public/style.css` — plain CSS, no runtime framework

### 9c — `wg-cli`
- [ ] `wg-cli status` — connect to Unix socket; pretty-print `StatusResponse`
- [ ] `wg-cli autostart enable/disable`:
  - [ ] Linux: write + enable systemd unit; poll `wg-cli status` up to 10s
  - [ ] FreeBSD: write + enable rc.d script
- [ ] `wg-cli upgrade` — send `GracefulRestart` via Unix socket; wait max 10s for clean exit

---

## Phase 10 — Observability

- [ ] Both agent and server: `tracing-subscriber` JSON in production, pretty in dev; `with_current_span(true)`, `FmtSpan::CLOSE`
- [ ] Server span hierarchy: `request{}` → `connection{}` → `command{}` / `tunnel{}`; `device_id` attached at connection establishment
- [ ] All server metrics from §13.2: `wg_connected_agents_total`, `wg_active_tunnels_total`, `wg_commands_sent_total`, `wg_rpc_duration_seconds`, `wg_agent_degraded`, etc.
- [ ] All agent metrics from §13.2: `wg_agent_capture_queue_depth`, `wg_agent_packets_sent_total`, `wg_agent_reconnect_attempts_total`, etc.; Prometheus endpoint optional (`metrics_port = 0` disables)
- [ ] OTLP tracing: `trace_id` (W3C) embedded in `command_id` prefix; `opentelemetry-otlp` export when `OTLP_ENDPOINT` set; no-op when empty

---

## Phase 11 — Agent Lifecycle & Cert Renewal

- [ ] `lifecycle/upgrade.rs` — handle `GracefulRestart` from CLI: finish in-flight commands (max 10s), exit cleanly; package post-install renames `.new` binary over live binary
- [ ] `lifecycle/cert_renewal.rs` — on `RenewCertificateRequest`: generate new keypair + CSR; send `RenewCertificateResponse`; on `SetCertificate`: write `.new` files, `fsync`, rename atomically
- [ ] Integration test: `cert_renewal` — full round-trip; agent reconnects with new cert

---

## Phase 12 — Firewall Config Management

*Everything below this line is firewall-specific. All prior phases work without it.*

### 12a — Firewall Parsers
- [ ] `fireparse/mod.rs` — `FirewallConfig` trait; `fireparse_for(FirewallKind) -> Option<Box<dyn FirewallConfig>>`
- [ ] `fireparse/pfsense.rs` — parse/serialize pfSense `config.xml` via `xmltree`; round-trip tests; reject corrupted input
- [ ] `fireparse/opnsense.rs` — parse/serialize OPNsense config via `xmltree`; same test requirements
- [ ] `fireparse/nftables.rs` — parse/serialize nftables JSON ruleset via `nftables` crate; same test requirements
- [ ] Add `Feature::ConfigMonitoring` to `derive_capabilities()` when `firewall != FirewallKind::None`

### 12b — Rule Protocol
- [ ] `models.proto` — `FilterRule`, `NatRule`, `Alias`, `Rule` union; `CreateFilterRule`, `CreateNatRule`, `CreateAlias`, `DeleteRule`, `ApplyRuleSet`, `RequestConfigSnapshot`, `ConfigSnapshot` messages
- [ ] Add these to `ServerMessage` / `ClientMessage` oneof in `control.proto`
- [ ] `wg-shared/src/types.rs` — `FirewallRule` type

### 12c — Agent Rule Application
- [ ] Agent: handle `CreateFilterRule`, `CreateNatRule`, `CreateAlias`, `DeleteRule`, `ApplyRuleSet` commands; dispatch through `fireparse_for(kind)`; return `CommandResult { applied_digest }` or `FAILURE`
- [ ] `ApplyRuleSet`: apply all rules in a single batch; on partial failure restore from `rollback_digest`; no partial-apply states

### 12d — Server Rule Management
- [ ] `migrations/006_firewall_tables.sql` — `firewall_rules`, `config_snapshots` tables
- [ ] `api/rules.rs` — `POST /devices/{id}/rules`, `DELETE /devices/{id}/rules/{id}`, `POST /devices/{id}/ruleset`
- [ ] `api/devices.rs` additions — `GET /devices/{id}/config/drift`
- [ ] On `Welcome` → send `RequestConfigSnapshot`; compare `applied_digest`; on mismatch → emit `config_drift` SSE event; surface via drift endpoint

### 12e — Named Commands (Firewall Operations)
- [ ] `named_commands.rs` — `NamedCommand` enum dispatch; hardcoded binary + args; no shell metacharacters; stdout max 64 KiB returned in `CommandResult.output`
- [ ] `api/commands.rs` — `POST /devices/{id}/commands/named`
- [ ] Add `ExecuteNamedCommand` to `ServerMessage` oneof

### 12f — Firewall UI
- [ ] `pages/rules.rs` — firewall rule editor
- [ ] `pages/devices/detail.rs` — config drift indicator (added to existing page)

---

## Phase 13 — `wg-testkit` & Integration Tests

### 13a — Test Harness
- [ ] `TestCa` — in-memory CA; `issue_device_cert(device_id, org_id)`
- [ ] `TestServer` — spawn real `wg-server` against `testcontainers` PostgreSQL+TimescaleDB
- [ ] `TestAgent` — spawn real `wg-agent` with temp config + cert from `TestCa`
- [ ] `CapturedPackets` — fake libpcap feed injecting synthetic packets

### 13b — Connectivity & Tunnels
- [ ] `enrollment_flow`
- [ ] `monitoring_enable`
- [ ] `ssh_tunnel`
- [ ] `tty_tunnel`
- [ ] `tunnel_multiplexing`
- [ ] `remote_desktop_tunnel`
- [ ] `remote_desktop_pli`
- [ ] `quic_fallback_to_tcp` — SSH works; remote desktop returns FAILURE
- [ ] `reconnect_backoff`
- [ ] `reconnect_replaces_stale`
- [ ] `disk_buffer_drain`
- [ ] `heartbeat_timeout`
- [ ] `version_rejection`
- [ ] `cert_renewal`
- [ ] `failure_live_delivery`
- [ ] `failure_buffer_replay`
- [ ] `failure_buffer_cap`
- [ ] `panic_hook_records`
- [ ] `failure_dedup`
- [ ] `auth_login_logout`
- [ ] `auth_rbac`
- [ ] `api_key_auth`
- [ ] `remote_desktop_headless` — mock probe returns `Err`; `Feature::RemoteDesktop` absent from `Hello`
- [ ] `remote_desktop_freebsd_with_display` — mock probe returns `Ok`; feature advertised

### 13c — Firewall Tests (after Phase 12)
- [ ] `rule_push_success`
- [ ] `rule_push_failure`
- [ ] `rule_push_timeout`
- [ ] `atomic_rule_set`
- [ ] `config_drift_detection`

### 13d — Contract Tests
- [ ] `buf breaking` baseline descriptor in `wg-shared/tests/descriptors/`
- [ ] CI step: `buf breaking --against .git#branch=main` — fails on wire-breaking proto change

---

## Phase 14 — CI / CD & Production Infrastructure

- [ ] `ci.yml` complete: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo test --profile test-abort`, `cargo nextest run -p wg-testkit`, `buf breaking`, `trunk build --release -p wg-ui`, `docker build -f Dockerfile.server .`
- [ ] `release.yml`: cross-compile agent to `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `x86_64-unknown-freebsd`; release archives; push `ghcr.io/.../wallguard-server` image
- [ ] `Dockerfile.server` complete: stage 1 WASM UI via `trunk`; stage 2 server binary; stage 3 minimal final image
- [ ] `helm/wallguard/` chart: Deployment (2+ replicas, anti-affinity), Services (ClusterIP gRPC, LoadBalancer API/QUIC-UDP/TCP-tunnel), ConfigMap, Secrets, optional TimescaleDB StatefulSet
- [ ] `helm/wallguard/values.yaml` — all values from §15.2
- [ ] Kubernetes rolling update: `ShutdownImminent` + 2+ replicas ensures zero-downtime deploys

---

## Dependency Notes

- `captis` must be evaluated for FreeBSD X11 support before Phase 8e begins; if unsupported, evaluate `scrap` crate or direct Xlib capture as replacement
- `nullnet-traffic-monitor` and `nullnet-libresmon` are kept from v1; verify they cross-compile to FreeBSD before Phase 7
- `enigo` keyboard/mouse injection: verify FreeBSD X11 backend; document any gaps before Phase 8e
- `pingora` is **not** used; HTTP proxy tunnel uses lightweight `hyper`-based proxy (no Linux-only dependency)
