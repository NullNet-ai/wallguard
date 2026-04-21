# WallGuard — Final Architecture Design

This document is a complete, unconstrained blueprint for the WallGuard system. It incorporates everything learned from the v1 audit (DESIGN.md §16) and the iterative redesign (DESIGN2.md), then adds the technology choices and system components that were deferred in those documents: the database layer, user authentication, the web UI, and the deployment model.

All external dependencies that created ownership or integration overhead (`nullnet-libdatastore`, `nullnet-libtoken`, the internal UI framework) are replaced with first-party components or well-established open-source crates. Every choice is justified here.

---

## Table of Contents

1. [Design Philosophy](#1-design-philosophy)
2. [Technology Stack](#2-technology-stack)
3. [Workspace & Project Layout](#3-workspace--project-layout)
4. [Security Architecture](#4-security-architecture)
   - 4.1 [PKI & Mutual TLS](#41-pki--mutual-tls)
   - 4.2 [Device Provisioning](#42-device-provisioning)
   - 4.3 [User Authentication & JWT](#43-user-authentication--jwt)
   - 4.4 [Role-Based Access Control](#44-role-based-access-control)
   - 4.5 [CLI Security](#45-cli-security)
   - 4.6 [Command Allow-list](#46-command-allow-list)
   - 4.7 [Secret Storage](#47-secret-storage)
5. [Transport Layer](#5-transport-layer)
   - 5.1 [gRPC Control Channel](#51-grpc-control-channel)
   - 5.2 [Reverse Tunnel over QUIC](#52-reverse-tunnel-over-quic)
   - 5.3 [HTTP API](#53-http-api)
6. [Control Channel Protocol](#6-control-channel-protocol)
   - 6.1 [Handshake & Version Negotiation](#61-handshake--version-negotiation)
   - 6.2 [Bidirectional Heartbeat](#62-bidirectional-heartbeat)
   - 6.3 [Command Acknowledgment](#63-command-acknowledgment)
   - 6.4 [Full Protobuf Schema](#64-full-protobuf-schema)
7. [Agent Architecture](#7-agent-architecture)
   - 7.1 [Platform Model](#71-platform-model)
   - 7.2 [Daemon State Machine](#72-daemon-state-machine)
   - 7.3 [Reconnection Backoff](#73-reconnection-backoff)
   - 7.4 [Data Transmission Pipeline](#74-data-transmission-pipeline)
   - 7.5 [Disk Buffer](#75-disk-buffer)
   - 7.6 [Monitoring Status](#76-monitoring-status)
   - 7.7 [Failure Reporting](#77-failure-reporting)
8. [Server Architecture](#8-server-architecture)
   - 8.1 [Connection Registry](#81-connection-registry)
   - 8.2 [Graceful Restart](#82-graceful-restart)
   - 8.3 [Horizontal Scaling Path](#83-horizontal-scaling-path)
9. [Tunnel Sessions](#9-tunnel-sessions)
   - 9.1 [QUIC Connection Lifecycle](#91-quic-connection-lifecycle)
   - 9.2 [SSH Tunnel](#92-ssh-tunnel)
   - 9.3 [TTY Tunnel](#93-tty-tunnel)
   - 9.4 [HTTP Proxy Tunnel](#94-http-proxy-tunnel)
   - 9.5 [Remote Desktop Tunnel](#95-remote-desktop-tunnel)
10. [Firewall Configuration Management](#10-firewall-configuration-management)
    - 10.1 [Apply-and-Acknowledge](#101-apply-and-acknowledge)
    - 10.2 [Rollback](#102-rollback)
    - 10.3 [State Reconciliation](#103-state-reconciliation)
11. [Database Design](#11-database-design)
    - 11.1 [Engine Choice](#111-engine-choice)
    - 11.2 [Relational Schema](#112-relational-schema)
    - 11.3 [Time-Series Schema](#113-time-series-schema)
    - 11.4 [Migrations](#114-migrations)
12. [Web UI Architecture](#12-web-ui-architecture)
    - 12.1 [Stack Choice: Leptos + WASM](#121-stack-choice-leptos--wasm)
    - 12.2 [Crate Layout](#122-crate-layout)
    - 12.3 [Page Structure](#123-page-structure)
    - 12.4 [Real-time Updates](#124-real-time-updates)
    - 12.5 [Build & Serving](#125-build--serving)
13. [Observability](#13-observability)
    - 13.1 [Structured Logging](#131-structured-logging)
    - 13.2 [Metrics](#132-metrics)
    - 13.3 [Distributed Tracing](#133-distributed-tracing)
14. [Agent Lifecycle](#14-agent-lifecycle)
    - 14.1 [Installation](#141-installation)
    - 14.2 [Autostart](#142-autostart)
    - 14.3 [Upgrades & Cert Renewal](#143-upgrades--cert-renewal)
15. [Infrastructure & Deployment](#15-infrastructure--deployment)
    - 15.1 [Development: Docker Compose](#151-development-docker-compose)
    - 15.2 [Production: Kubernetes + Helm](#152-production-kubernetes--helm)
    - 15.3 [Database Provisioning](#153-database-provisioning)
16. [Configuration Reference](#16-configuration-reference)
17. [Testing Strategy](#17-testing-strategy)
18. [Dependency Inventory](#18-dependency-inventory)

---

## 1. Design Philosophy

Seven principles govern every decision in this document:

| # | Principle | In practice |
|---|---|---|
| P1 | **Security by default** | Every connection is mutually authenticated before any data flows. There is no code path that accepts an unauthenticated connection, even on localhost. |
| P2 | **No silent failures** | Every mutating operation across a process or network boundary has an explicit acknowledgment. The initiator always learns whether an operation succeeded. |
| P3 | **No panics in production paths** | `unwrap()` and `expect()` are banned outside `#[cfg(test)]`. All fallible paths return `Result`. The process never crashes due to a recoverable error. |
| P4 | **Backpressure everywhere** | No pipeline stage accepts work faster than the next stage can consume. Load shedding is explicit, metered, and visible in metrics. |
| P5 | **Observable** | Every component emits structured JSON logs, Prometheus metrics, and OTLP traces. Debugging a production incident never requires a code change. |
| P6 | **Least privilege** | The agent executes only what is on a static typed allow-list. The server never constructs or executes shell commands. |
| P7 | **Own your dependencies** | No external service is a hard runtime dependency for core functionality. The database, auth, and UI are all first-party or vanilla open source with no proprietary SDK overhead. |

---

## 2. Technology Stack

### Why Rust throughout

The entire system — agent, server, CLI, and web UI — is written in Rust. The reasons differ by component but the result is a consistent toolchain with no polyglot operational overhead.

**Agent:** The agent must run on FreeBSD (pfSense/OPNsense), Linux (nftables), and future Windows targets. Rust cross-compiles natively to all three. The compiled binary has no runtime dependency — no JVM, no Python interpreter, no Node.js. A single static binary is dropped onto the device and runs. This is the core constraint that ruled out Go (cgo complications on FreeBSD), Python (interpreter required), and every JVM or .NET option.

**Server:** The server handles high-throughput streaming data (packet telemetry from potentially hundreds of devices), long-lived bidirectional gRPC streams, and QUIC connections. Rust's async model (`tokio`) handles this without a GC pause problem. The entire server, including the HTTP API, gRPC handlers, and QUIC tunnel multiplexer, runs in one binary with predictable latency.

**CLI:** A compiled binary. No runtime. Connects to the agent via a Unix socket. Same codebase as the server for protocol types, so they never drift.

**Web UI (`wg-ui`):** Leptos compiled to WebAssembly. The browser downloads and executes a WASM binary — there is no JavaScript framework runtime. The UI is compiled with `rustc`; type errors in UI code are caught at build time. The same domain types (`Device`, `Rule`, `TunnelSession`) used in the server's API layer are shared with the UI via a `wg-shared` crate that compiles to both native and WASM targets.

### Database: PostgreSQL + TimescaleDB

Relational data (devices, users, roles, config snapshots, audit logs) lives in PostgreSQL accessed via `sqlx` with compile-time query verification. Time-series data (packet telemetry, resource metrics, monitoring status history) lives in TimescaleDB hypertables — a PostgreSQL extension that partitions by time automatically, provides time-bucketing query functions, and handles continuous aggregation. This replaces two separate systems with one engine and one query language.

`nullnet-libdatastore` is removed. The server owns its schema and migration history directly via `sqlx migrate`.

### Authentication: Self-contained JWT

User authentication uses `argon2` for password hashing and `jsonwebtoken` (HMAC-SHA256 or Ed25519 signing) for session tokens. No external auth service or proprietary SDK is required. The token signing key is generated at first server startup and stored in the database. OIDC/SAML integration is a future option; the API layer is structured so auth middleware is pluggable without touching handler code.

---

## 3. Workspace & Project Layout

```
wallguard/
├── Cargo.toml                      # Workspace root
├── proto/
│   ├── provisioning.proto          # Device enrollment (unauthenticated service)
│   ├── control.proto               # Bidirectional control channel
│   ├── data.proto                  # Telemetry RPCs
│   ├── models.proto                # Shared domain types
│   └── cli.proto                   # CLI ↔ agent (Unix socket)
├── migrations/                     # sqlx SQL migration files (applied by wg-server)
│   ├── 001_initial_schema.sql
│   ├── 002_timescale_hypertables.sql
│   └── ...
├── crates/
│   ├── wg-shared/                  # Domain types, proto stubs, PKI helpers
│   │                               # Compiles to both native and wasm32 targets
│   ├── wg-agent/                   # Agent binary
│   ├── wg-cli/                     # CLI binary
│   ├── wg-server/                  # Server binary (HTTP API + gRPC + QUIC)
│   ├── wg-ui/                      # Leptos web UI, compiled to WASM
│   └── wg-testkit/                 # Integration test harness
├── helm/                           # Helm chart for Kubernetes deployment
│   └── wallguard/
├── docker-compose.yml              # Local development stack
├── Dockerfile.server               # Multi-stage build for wg-server
├── Dockerfile.ui                   # Build stage for wg-ui WASM assets
└── .github/workflows/
    ├── ci.yml
    └── release.yml
```

**Crate responsibilities:**

| Crate | Binary | Target | Notes |
|---|---|---|---|
| `wg-shared` | library | `x86_64`, `aarch64`, `wasm32` | Types shared between server and UI; no I/O, no tokio |
| `wg-agent` | `wg-agent` | `x86_64-linux`, `x86_64-freebsd`, `aarch64-linux` | Compiled per target platform |
| `wg-cli` | `wg-cli` | same as agent | Ships alongside agent binary |
| `wg-server` | `wg-server` | `x86_64-linux` only | Embeds compiled `wg-ui` WASM assets |
| `wg-ui` | WASM bundle | `wasm32-unknown-unknown` | Built by trunk; output embedded in `wg-server` |
| `wg-testkit` | test binary | `x86_64-linux` | Used only in integration tests |

---

## 4. Security Architecture

### 4.1 PKI & Mutual TLS

The system operates a two-tier PKI. The Root CA is kept offline. The Intermediate CA lives on the server and signs device certificates during enrollment.

```
Root CA  (offline, long-lived, kept in cold storage)
    └── Intermediate CA  (online, on the server)
            ├── Server leaf cert   (gRPC + QUIC TLS)
            └── Device cert_N      (one per enrolled device)
```

**Server certificate:**
- SAN: server's DNS name(s) and IP addresses.
- Signed by the Intermediate CA.
- Renewed annually.

**Device certificate:**
- Ed25519 key generated on the device; private key never leaves the device.
- Subject CN: `device:<uuid>` — the server extracts device identity from the cert, not from a message field.
- Subject O: `org:<org_id>` — org scoping enforced at the TLS layer.
- 1-year validity; auto-renewed 30 days before expiry.

**What mTLS replaces:**
- `AcceptAllVerifier` is deleted. Standard `rustls` `WebPkiServerVerifier` is used everywhere.
- The `app_id`/`app_secret` message exchange is eliminated. Identity is the TLS handshake.
- The SHA-256 token trick on the raw TCP tunnel port is replaced by the device certificate on the QUIC connection.

**CA paths on server:**

| Path | Content | Mode |
|---|---|---|
| `/etc/wallguard-server/ca.crt` | Intermediate CA cert | 0644 |
| `/etc/wallguard-server/ca.key` | Intermediate CA private key | 0600 |
| `/etc/wallguard-server/server.crt` | Server leaf cert | 0644 |
| `/etc/wallguard-server/server.key` | Server private key | 0600 |

**CA paths on agent:**

| Path | Content | Mode |
|---|---|---|
| `/etc/wallguard/ca.crt` | Pinned Intermediate CA cert | 0644 |
| `/etc/wallguard/device.crt` | Signed device cert | 0644 |
| `/etc/wallguard/device.key` | Device Ed25519 private key | 0600 |

---

### 4.2 Device Provisioning

Provisioning is a one-time operation triggered by `wg-cli enroll <code> --server <url>`. It runs over a separate `Provisioning` gRPC service so unenrolled agents never reach the control plane.

```
Agent                                   Server :50051 (Provisioning service)
  │                                             │
  │  1. Generate Ed25519 keypair                │
  │     Write device.key (mode 0600)            │
  │                                             │
  │  2. Build CSR:                              │
  │     CN = "device:<new-uuid>"                │
  │     O  = "org:<pending>"                    │
  │                                             │
  │── TLS connect (server cert vs pinned CA) ──►│
  │                                             │
  │── EnrollRequest {                          ►│  3. Validate code (atomic mark-used)
  │     installation_code,                      │  4. Sign CSR → device cert
  │     csr_pem,                                │  5. Create Device row
  │     firewall_kind,                          │  6. Write audit log entry
  │     agent_version                           │
  │   }                                         │
  │                                             │
  │◄─ EnrollResponse {                          │
  │     device_id,                              │
  │     device_cert_pem,                        │
  │     ca_cert_pem,                            │
  │     server_name                             │
  │   }                                         │
  │                                             │
  │  7. Write device.crt (0644), ca.crt (0644)  │
  │  8. Write config.toml (0644)                │
  │  9. Connect to Control service (mTLS)       │
```

Properties:
- Private key is generated on the device; never transmitted.
- Installation codes are single-use; the server marks them consumed atomically in a transaction before signing — a replay cannot succeed.
- Device identity is extracted from the signed cert's CN; forgery requires compromising the Intermediate CA.

---

### 4.3 User Authentication & JWT

User-facing authentication is entirely self-contained. There is no external auth service dependency.

**Password storage:** `argon2` (Argon2id, m=64MiB, t=3, p=4). Passwords are never stored in plaintext or as bcrypt/SHA hashes.

**Session tokens:** JWT signed with HMAC-SHA256. The signing secret (32 random bytes) is generated at first server startup and stored in the `server_secrets` table. Token claims:

```json
{
  "sub":   "user-uuid",
  "org":   "org-uuid",
  "role":  "operator",
  "iat":   1700000000,
  "exp":   1700086400,
  "jti":   "token-uuid"
}
```

Token lifetime: 24 hours (configurable). Refresh tokens (30-day lifetime) are stored in the `refresh_tokens` table; rotation on each use. Revocation is by deleting the `jti` from the `revoked_tokens` table (checked on every request).

**API key authentication:** Long-lived tokens for programmatic access. Stored as `argon2(key)` in `api_keys` table. Presented as `Authorization: Bearer wg_<base64url(key)>`. Same RBAC checks as JWT sessions.

**Login flow:**

```
POST /api/v1/auth/login  { email, password }
  → verify argon2 hash
  → issue JWT (24h) + refresh token (30d)
  → 200 { access_token, refresh_token, expires_in }

POST /api/v1/auth/refresh  { refresh_token }
  → validate, rotate, issue new JWT
  → 200 { access_token, refresh_token, expires_in }

POST /api/v1/auth/logout
  → add jti to revoked_tokens
  → delete refresh_token
  → 204
```

---

### 4.4 Role-Based Access Control

Four roles per organization. Every API endpoint declares the minimum role required.

| Role | Description | Example capabilities |
|---|---|---|
| `owner` | Full control including user management and org deletion | Everything |
| `admin` | Manage devices, rules, users (not org deletion) | Add devices, manage rules, add users |
| `operator` | Day-to-day operations | Open tunnels, push rules, read telemetry |
| `viewer` | Read-only | View device status, telemetry, failures |

RBAC is enforced in a middleware layer applied to all API routes before handlers execute. The middleware extracts the JWT, validates it, and attaches a `RequestContext { user_id, org_id, role }` to the request. Handlers that need elevated privileges call `ctx.require_role(Role::Admin)?`.

Device-level permission overrides are supported for fine-grained access: a user can be granted or denied access to specific devices independently of their org-level role. This is stored in the `device_permissions` table.

---

### 4.5 CLI Security

The CLI communicates with the local agent via a **Unix domain socket**:

- Path: `/run/wallguard/agent.sock`
- Ownership: `root:root`
- Mode: `0600`

The kernel enforces access control. Only root (or a process with `CAP_DAC_OVERRIDE`) can connect. No application-layer authentication is needed.

```rust
// Agent startup
let socket_path = Path::new("/run/wallguard/agent.sock");
if socket_path.exists() {
    std::fs::remove_file(socket_path)?;
}
let listener = tokio::net::UnixListener::bind(socket_path)?;
std::fs::set_permissions(socket_path, Permissions::from_mode(0o600))?;
```

```rust
// CLI connection
let channel = tonic::transport::Endpoint::try_from("http://[::]:0")?
    .connect_with_connector(tower::service_fn(|_| {
        tokio::net::UnixStream::connect("/run/wallguard/agent.sock")
    }))
    .await?;
```

The old TCP listener on `127.0.0.1:54056` is deleted.

---

### 4.6 Command Allow-list

The freeform `execute_cli_command(command: String, arguments: Vec<String>)` RPC is replaced with a typed enum. Adding a new command requires a `.proto` schema change and a new match arm in the agent — there is no code path that passes arbitrary strings to a shell.

```protobuf
enum NamedCommand {
  RELOAD_FIREWALL_RULES       = 0;
  RESTART_NETWORK_INTERFACE   = 1;
  FLUSH_CONNECTION_TRACKING   = 2;
  SHOW_ARP_TABLE              = 3;
  SHOW_ROUTING_TABLE          = 4;
  SHOW_ACTIVE_CONNECTIONS     = 5;
}
```

Each variant dispatches to a hardcoded function that calls a specific binary with hardcoded arguments. No caller-supplied paths or shell metacharacters are ever involved.

---

### 4.7 Secret Storage

After provisioning, the agent's only secret is the device private key at `/etc/wallguard/device.key` (mode 0600). There is no `AppSecret` file. All secret files are written with the mode set at `open()` time, before any data is written:

```rust
fn write_secret_file(path: &Path, data: &[u8]) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true).create(true).truncate(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(data)?;
    file.sync_all()?;
    Ok(())
}
```

---

## 5. Transport Layer

### 5.1 gRPC Control Channel

All gRPC services (Provisioning, Control, Data) are served on **port 50051** over TLS. After provisioning, every connection uses mTLS.

Services:
- `Provisioning` — exempt from client cert requirement (unenrolled agents). Interceptor enforces this exemption explicitly.
- `Control` — bidirectional streaming; requires valid device cert.
- `Data` — streaming telemetry upload; requires valid device cert.

```rust
let tls_config = ServerTlsConfig::new()
    .identity(Identity::from_pem(server_cert_pem, server_key_pem))
    .client_ca_root(Certificate::from_pem(ca_cert_pem))
    .client_auth_optional(false);
```

The agent pins the CA cert and presents its device cert:

```rust
let tls_config = ClientTlsConfig::new()
    .ca_certificate(Certificate::from_pem(ca_cert_pem))
    .identity(Identity::from_pem(device_cert_pem, device_key_pem));
```

---

### 5.2 Reverse Tunnel over QUIC

Port 7777 is a **QUIC/UDP endpoint**. TCP (port 7778 TLS) is a fallback for networks where UDP is blocked.

QUIC over TCP for this use case:

| Property | TCP + TLS | QUIC |
|---|---|---|
| Authentication | mTLS | mTLS (TLS 1.3 built in) |
| Head-of-line blocking | Per connection | None — streams independent |
| Stream multiplexing | No | Yes, unlimited |
| Unreliable datagrams | No | Yes (RFC 9221) — for video frames |
| 0-RTT reconnect | No | Yes |
| Connection migration | No | Yes |

The SHA-256 token trick on the old raw TCP port is entirely removed. Authentication is the QUIC handshake with the device certificate.

**Persistent multiplexed connection:** The agent maintains one QUIC connection for its entire `Connected` lifetime. All tunnel sessions — SSH, TTY, HTTP, Remote Desktop — are independent streams or datagrams on this single connection. When the server sends `OpenSshTunnel { tunnel_id }`, the agent opens a new QUIC stream and sends `TunnelHello { tunnel_id }` on it — no new connection establishment.

**QUIC connection parameters:**

| Parameter | Value |
|---|---|
| `max_concurrent_bidi_streams` | 64 |
| `max_idle_timeout` | 60s |
| `keep_alive_interval` | 15s |
| `datagram_receive_buffer_size` | 4 MiB |

**TCP fallback:** The agent tries QUIC first. If connection fails within 3 seconds, it retries over TLS/TCP on port 7778. Under TCP fallback, Remote Desktop is unavailable (no unreliable datagrams). The fallback preference is recorded in `config.toml` and can be forced.

---

### 5.3 HTTP API

The HTTP API runs on **port 4444** (`axum`, replacing `actix-web`). `axum` is chosen because it integrates cleanly with `tower` middleware, uses the same `tokio` + `hyper` stack as `tonic`, and eliminates an `actix-web` dependency that duplicates the async runtime machinery.

Every request:
1. Passes through `AuthMiddleware` — validates JWT or API key, attaches `RequestContext`.
2. Passes through `RbacMiddleware` — checks minimum role for the route.
3. Receives an auto-generated `X-Request-Id` header if absent.
4. Has all log lines within the handler carry the request ID.
5. Returns `{ "error": { "code": "...", "message": "...", "request_id": "..." } }` on failure.

WebSocket endpoints for tunnel sessions (SSH, TTY, Remote Desktop) are served on the same port via `axum`'s `WebSocketUpgrade` handler.

Server-Sent Events (SSE) endpoints for real-time UI updates are served on the same port.

---

## 6. Control Channel Protocol

### 6.1 Handshake & Version Negotiation

The connection always starts with a formal three-message exchange before any commands flow. The old `Authentication` message exchange (which echoed credentials back to their issuer) is removed; identity is established by the TLS handshake.

```
Agent                                   Server
  │                                         │
  │  [QUIC/mTLS handshake — device cert]    │
  │                                         │
  │── Hello {                             ──►
  │     protocol_version: 2,               │
  │     min_compatible_version: 1,         │
  │     supported_features: [...],         │  from derive_capabilities()
  │     agent_version: "0.2.0",            │
  │     firewall_kind: OPNSENSE,           │  display only
  │   }                                    │
  │                                         │
  │◄─ Welcome {                         ───
  │     protocol_version: 2,               │
  │     negotiated_features: [...],        │  agent ∩ server_config
  │     initial_settings: { ... },         │  replaces GetDeviceSettings RPC
  │     server_version: "1.0.0",           │
  │   }                                    │
  │                                         │
  │  OR on version mismatch:               │
  │◄─ VersionRejected {                 ───
  │     min_required_version: 2,           │
  │     message: "Upgrade required",       │
  │   }                                    │
  │  (server closes stream; agent stops retrying)
```

`Welcome.initial_settings` is delivered atomically with handshake completion, eliminating the race condition in v1 where `SetMonitoring` commands sent immediately after connection could arrive before the agent was ready.

**Version policy:** `protocol_version` is a monotonically increasing integer. The server's `MIN_AGENT_PROTOCOL_VERSION` config rejects agents below the minimum with `VersionRejected`. Agents receiving `VersionRejected` do not retry — they log and stop. A human upgrade is required.

---

### 6.2 Bidirectional Heartbeat

Both sides send heartbeats. The v1 unidirectional (agent→server only) design left the agent unable to detect server-side failures.

```
Interval:         10 seconds
Miss threshold:   3 consecutive misses
Dead window:      10s × (3 + 1) = 40 seconds
```

Agent heartbeats carry a `MonitoringStatus` snapshot — queue depths, drop counters, disk buffer usage. This gives the server continuous visibility into agent health without polling.

On 3 consecutive missed heartbeats from either side, the stream is closed and the reconnection loop begins.

---

### 6.3 Command Acknowledgment

Every command that mutates state on the agent carries a `command_id` (UUID v4). The agent always returns a `CommandResult` after execution. The server's HTTP API awaits the result with a 30-second timeout before responding to the HTTP caller.

```
Server                                  Agent
  │── CreateFilterRule {              ──►
  │     command_id: "abc-123",          │
  │     rule: FilterRule { ... }        │
  │   }                                 │
  │                                     │  apply rule
  │◄─ CommandResult {               ───
  │     command_id: "abc-123",          │
  │     status: SUCCESS,                │
  │     applied_digest: "sha256:...",   │
  │   }                                 │

HTTP response: 200 { applied_digest }
```

On failure: HTTP 422. On 30s timeout: HTTP 504. The HTTP caller is never left waiting indefinitely.

A background sweeper runs every 5 seconds on the server and sends `TIMEOUT` errors to any `PendingCommand` older than 30 seconds.

---

### 6.4 Full Protobuf Schema

```protobuf
syntax = "proto3";
package wallguard.control;

// The firewall software installed on the device.
// Used by the server for display only — never for capability decisions.
enum FirewallKind {
  FIREWALL_KIND_NONE     = 0;
  FIREWALL_KIND_PFSENSE  = 1;
  FIREWALL_KIND_OPNSENSE = 2;
  FIREWALL_KIND_NFTABLES = 3;
}

// Capabilities declared by the agent at handshake time.
// Derived by derive_capabilities(FirewallKind).
// The server computes the intersection with its own config as negotiated_features.
enum Feature {
  NETWORK_MONITORING   = 0;
  TELEMETRY_MONITORING = 1;
  CONFIG_MONITORING    = 2;
  SSH_TUNNEL           = 3;
  TTY_TUNNEL           = 4;
  HTTP_TUNNEL          = 5;
  NAMED_COMMANDS       = 6;
  REMOTE_DESKTOP       = 7;  // Only present if compiled with remote-desktop feature
}

enum FailureSeverity { WARNING = 0; ERROR = 1; FATAL = 2; }

enum FailureCategory {
  MONITORING   = 0;
  TUNNEL       = 1;
  DISK_BUFFER  = 2;
  FIREPARSE    = 3;
  AGENT_CRASH  = 4;
  CONNECTIVITY = 5;
  SYSTEM       = 6;
}

enum CommandStatus { SUCCESS = 0; FAILURE = 1; TIMEOUT = 2; }

enum NamedCommand {
  RELOAD_FIREWALL_RULES     = 0;
  RESTART_NETWORK_INTERFACE = 1;
  FLUSH_CONNECTION_TRACKING = 2;
  SHOW_ARP_TABLE            = 3;
  SHOW_ROUTING_TABLE        = 4;
  SHOW_ACTIVE_CONNECTIONS   = 5;
}

// --- Handshake ---

message Hello {
  uint32           protocol_version       = 1;
  uint32           min_compatible_version = 2;
  repeated Feature supported_features     = 3;
  string           agent_version          = 4;
  FirewallKind     firewall_kind          = 5;
}

message Welcome {
  uint32           protocol_version    = 1;
  repeated Feature negotiated_features = 2;
  DeviceSettings   initial_settings    = 3;
  string           server_version      = 4;
}

message VersionRejected {
  uint32 min_required_version = 1;
  string message              = 2;
}

message DeviceSettings {
  bool   traffic_monitoring_enabled  = 1;
  bool   telemetry_monitoring_enabled = 2;
  bool   config_monitoring_enabled   = 3;
  float  packet_sampling_rate        = 4;  // 1.0 = full rate
}

// --- Heartbeat ---

message Heartbeat {
  uint64           seq               = 1;
  uint64           sent_at_unix_ms   = 2;
  MonitoringStatus monitoring_status = 3;  // Agent only; server leaves empty
}

message HeartbeatAck {
  uint64 ack_seq          = 1;
  uint64 acked_at_unix_ms = 2;
}

message MonitoringStatus {
  uint32 packet_queue_depth    = 1;
  uint64 disk_buffer_bytes     = 2;
  uint64 disk_buffer_max_bytes = 3;
  uint64 packets_dropped_total = 4;
  uint64 packets_sent_total    = 5;
  bool   degraded              = 6;
  uint32 active_tunnel_count   = 7;
}

// --- Failure reporting ---

message AgentFailure {
  string          failure_id  = 1;  // UUID v4; stable across replays; used for dedup
  FailureSeverity severity    = 2;
  FailureCategory category    = 3;
  string          message     = 4;
  string          context     = 5;  // JSON with category-specific fields
  uint64          occurred_at = 6;  // Unix ms at time of occurrence, not delivery
  bool            is_replay   = 7;  // True when sent from local buffer after reconnect
}

// --- Command result ---

message CommandResult {
  string        command_id         = 1;
  CommandStatus status             = 2;
  string        error_message      = 3;
  string        applied_digest     = 4;  // SHA-256 of resulting firewall config
  string        output             = 5;  // Named command stdout (max 64 KiB)
  uint64        applied_at_unix_ms = 6;
}

// --- Server → Agent commands ---

message SetMonitoring {
  string command_id         = 1;
  bool   traffic_enabled    = 2;
  bool   telemetry_enabled  = 3;
  bool   config_enabled     = 4;
}

message ThrottleMonitoring {
  float packet_sampling_rate = 1;
}

message OpenSshTunnel {
  string command_id = 1;
  string tunnel_id  = 2;
  string public_key = 3;
  string username   = 4;
}

message OpenTtyTunnel {
  string command_id = 1;
  string tunnel_id  = 2;
}

message OpenHttpTunnel {
  string command_id  = 1;
  string tunnel_id   = 2;
  string target_host = 3;
  uint32 target_port = 4;
}

message OpenRemoteDesktopTunnel {
  string command_id  = 1;
  string tunnel_id   = 2;
  uint32 width       = 3;
  uint32 height      = 4;
  uint32 target_fps  = 5;
  uint32 target_kbps = 6;
}

message CreateFilterRule { string command_id = 1; FilterRule rule  = 2; }
message CreateNatRule    { string command_id = 1; NatRule    rule  = 2; }
message CreateAlias      { string command_id = 1; Alias      alias = 2; }
message DeleteRule       { string command_id = 1; string     rule_id = 2; }

message ApplyRuleSet {
  string        command_id       = 1;
  repeated Rule rules            = 2;
  string        rollback_digest  = 3;
}

message ExecuteNamedCommand {
  string       command_id = 1;
  NamedCommand command    = 2;
  oneof params {
    RestartInterfaceParams restart_interface = 3;
  }
}

message RequestConfigSnapshot { string command_id = 1; }

message RenewCertificateRequest {}

message ShutdownImminent {
  uint32 reconnect_after_ms = 1;
}

// --- Agent → Server messages ---

message ConfigSnapshot {
  string command_id     = 1;
  string digest         = 2;
  bytes  configuration  = 3;  // Serialized firewall config
}

message RenewCertificateResponse {
  string csr_pem = 1;
}

// --- Stream multiplexers ---

message ServerMessage {
  oneof message {
    Welcome                   welcome                    = 1;
    VersionRejected           version_rejected           = 2;
    HeartbeatAck              heartbeat_ack              = 3;
    Heartbeat                 server_heartbeat           = 4;
    SetMonitoring             set_monitoring             = 5;
    ThrottleMonitoring        throttle_monitoring        = 6;
    OpenSshTunnel             open_ssh_tunnel            = 7;
    OpenTtyTunnel             open_tty_tunnel            = 8;
    OpenHttpTunnel            open_http_tunnel           = 9;
    OpenRemoteDesktopTunnel   open_remote_desktop_tunnel = 10;
    CreateFilterRule          create_filter_rule         = 11;
    CreateNatRule             create_nat_rule            = 12;
    CreateAlias               create_alias               = 13;
    DeleteRule                delete_rule                = 14;
    ApplyRuleSet              apply_rule_set             = 15;
    ExecuteNamedCommand       execute_named_command      = 16;
    RequestConfigSnapshot     request_config_snapshot    = 17;
    RenewCertificateRequest   renew_certificate_request  = 18;
    ShutdownImminent          shutdown_imminent          = 19;
  }
}

message ClientMessage {
  oneof message {
    Hello                     hello                      = 1;
    Heartbeat                 heartbeat                  = 2;
    HeartbeatAck              heartbeat_ack              = 3;
    CommandResult             command_result             = 4;
    ConfigSnapshot            config_snapshot            = 5;
    AgentFailure              agent_failure              = 6;
    RenewCertificateResponse  renew_certificate_response = 7;
  }
}
```

---

## 7. Agent Architecture

### 7.1 Platform Model

#### The v1 Problem

The v1 `--platform` argument was doing three unrelated jobs:

| Job | Example | Problem |
|---|---|---|
| Identify firewall software | `PfSense` → XML parser | The one thing that genuinely needs to be a runtime argument |
| Hint at OS | `PfSense` → assume FreeBSD | OS is knowable at compile time |
| Define the feature set | `Desktop` → enable remote desktop | Features should be derived, not labeled |

`Generic` was a catch-all. `Desktop` was not a firewall at all — it was an attempt to enable a capability by abusing a platform label.

#### The Fix

**One runtime argument:**

```
wg-agent --firewall <pfsense|opnsense|nftables|none>
          (default: none)
```

This answers exactly one question: which config file format does this device use?

**OS at compile time:**

```rust
pub const TARGET_OS: TargetOs = {
    #[cfg(target_os = "linux")]   { TargetOs::Linux   }
    #[cfg(target_os = "freebsd")] { TargetOs::FreeBsd }
    #[cfg(target_os = "windows")] { TargetOs::Windows }
};
```

**Remote desktop as a Cargo feature flag:**

```toml
[features]
remote-desktop = ["dep:captis", "dep:openh264", "dep:enigo"]
```

Firewall appliance builds omit `--features remote-desktop`. The binary does not contain screen capture code.

#### Capability Derivation

```rust
pub fn derive_capabilities(firewall: FirewallKind) -> Vec<Feature> {
    let mut caps = vec![
        Feature::NetworkMonitoring,
        Feature::TelemetryMonitoring,
        Feature::SshTunnel,
        Feature::TtyTunnel,
        Feature::HttpTunnel,
        Feature::NamedCommands,
    ];

    if firewall != FirewallKind::None {
        caps.push(Feature::ConfigMonitoring);
    }

    #[cfg(feature = "remote-desktop")]
    #[cfg(not(target_os = "freebsd"))]
    caps.push(Feature::RemoteDesktop);

    caps
}
```

**Capability matrix:**

| Feature | `pfsense` | `opnsense` | `nftables` | `none` |
|---|:---:|:---:|:---:|:---:|
| NetworkMonitoring | ✓ | ✓ | ✓ | ✓ |
| TelemetryMonitoring | ✓ | ✓ | ✓ | ✓ |
| ConfigMonitoring | ✓ | ✓ | ✓ | ✗ |
| SshTunnel | ✓ | ✓ | ✓ | ✓ |
| TtyTunnel | ✓ | ✓ | ✓ | ✓ |
| HttpTunnel | ✓ | ✓ | ✓ | ✓ |
| NamedCommands | ✓ | ✓ | ✓ | ✓ |
| RemoteDesktop* | ✗ | ✗ | ✗ | ✓ |

\* Only when `--features remote-desktop` on a non-FreeBSD target.

The server receives `Hello.firewall_kind` for display purposes and `Hello.supported_features` for all capability decisions. The server never inspects `firewall_kind` to derive what it can do with a device.

**Config parser dispatch:**

```rust
pub fn fireparse_for(kind: FirewallKind) -> Option<Box<dyn FirewallConfig>> {
    match kind {
        FirewallKind::PfSense  => Some(Box::new(pfsense::Parser::new())),
        FirewallKind::OPNSense => Some(Box::new(opnsense::Parser::new())),
        FirewallKind::NFTables => Some(Box::new(nftables::Parser::new())),
        FirewallKind::None     => None,
    }
}
```

---

### 7.2 Daemon State Machine

```
            start
         ─────────►  Provisioning  (no device cert on disk)
                          │ cert issued
                          ▼
                        Idle  ◄────────────────────────────┐
                          │ cert valid                      │
                          ▼                                 │
                      Connecting  ──── backoff loop ────────┘
                      (on failure)
                          │ Hello/Welcome exchanged
                          ▼
                      Connected  ──── disconnect / 3 missed heartbeats ──► Connecting
```

There is no `Error` state. Transient errors go back to `Connecting` with backoff. Permanent errors (`VersionRejected`, cert revoked) go to `Idle` with a log line and a metric. The daemon never requires a human restart for a transient network failure.

---

### 7.3 Reconnection Backoff

```rust
pub struct Backoff {
    base:       Duration,  // 1s
    max:        Duration,  // 300s (5 minutes)
    multiplier: f64,       // 2.0
    jitter:     f64,       // ±20% of current delay
    current:    Duration,
}

impl Backoff {
    pub fn next_delay(&mut self) -> Duration {
        let jitter_range  = self.current.mul_f64(self.jitter);
        let jitter_offset = jitter_range.mul_f64(rand::random::<f64>() * 2.0 - 1.0);
        let delay = self.current.saturating_add(jitter_offset).min(self.max);
        self.current = self.current.mul_f64(self.multiplier).min(self.max);
        delay
    }

    pub fn reset(&mut self) { self.current = self.base; }
}
```

Delay sequence (without jitter): 1s, 2s, 4s, 8s, 16s, 32s, 64s, 128s, 256s, 300s, 300s, …

The ±20% jitter prevents reconnection thundering herds when a server restarts and many agents try to reconnect simultaneously.

---

### 7.4 Data Transmission Pipeline

```
libpcap capture task
    │
    │  bounded channel (cap = 50_000)
    │  [drop + increment wg_agent_capture_drops_total if full]
    ▼
Batch assembler task
    │  accumulate until: 1_000 packets OR 500ms elapsed
    │  apply sampling rate from ThrottleMonitoring command
    ▼
Transmit task
    ├─ connected:   send batch via Data gRPC RPC
    │               on failure → write to DiskBuffer
    │               DiskBuffer full → drop + metric
    └─ disconnected: write to DiskBuffer
                    DiskBuffer full → drop + metric

On reconnect:
    ├─ drain DiskBuffer (all buffered batches sent before live data resumes)
    └─ emit: "Resuming live capture; replayed N buffered batches"
```

The server can remotely throttle a high-traffic device by sending `ThrottleMonitoring { packet_sampling_rate: 0.1 }`. The batch assembler samples every N-th packet atomically via an `Arc<AtomicU32>`. The server can also stop monitoring entirely via `SetMonitoring { traffic_enabled: false }`.

---

### 7.5 Disk Buffer

```rust
const MAX_DISK_BUFFER_BYTES: u64 = 256 * 1024 * 1024;  // 256 MiB
const MIN_FREE_DISK_BYTES:   u64 = 512 * 1024 * 1024;  // Always leave 512 MiB

pub struct DiskBuffer { dir: PathBuf, max_bytes: u64 }

impl DiskBuffer {
    /// Returns true if written. Never panics.
    pub async fn try_write(&self, batch_id: u64, data: &[u8]) -> bool {
        let avail = match available_space(&self.dir).await {
            Ok(n)  => n,
            Err(e) => { tracing::error!("disk stat failed: {e}"); return false; }
        };
        if avail < MIN_FREE_DISK_BYTES { return false; }

        let current = self.used_bytes().await.unwrap_or(u64::MAX);
        if current >= self.max_bytes { return false; }

        tokio::fs::write(
            self.dir.join(format!("{batch_id:016x}.bin")),
            data,
        ).await.is_ok()
    }
}
```

`try_write` checks available disk space dynamically on every write. It never panics and never blocks. A `false` return is treated as a drop event with a metric increment.

---

### 7.6 Monitoring Status

Every agent heartbeat includes a `MonitoringStatus` snapshot. This gives the server continuous visibility into device health without polling:

```rust
fn build_monitoring_status(pipeline: &Pipeline, buf: &DiskBuffer) -> MonitoringStatus {
    MonitoringStatus {
        packet_queue_depth:    pipeline.capture_queue_depth(),
        disk_buffer_bytes:     buf.used_bytes_cached(),
        disk_buffer_max_bytes: buf.max_bytes,
        packets_dropped_total: pipeline.drop_counter(),
        packets_sent_total:    pipeline.sent_counter(),
        degraded:              pipeline.is_degraded(),
        active_tunnel_count:   pipeline.active_tunnels(),
    }
}
```

The server stores the latest `MonitoringStatus` per device in memory and:
- Emits a `wg_agent_degraded{device_id=...}` gauge when `degraded = true`.
- Writes to the `device_monitoring_status` TimescaleDB hypertable (throttled to once per minute).
- Surfaces the status via `GET /api/v1/devices/{id}/status`.

---

### 7.7 Failure Reporting

`tracing::error!` on a remote firewall appliance is not a reliable failure signal. This section defines a two-layer mechanism: a persistent local ring buffer that survives process restarts, and remote delivery over the control channel.

**Scope:** `AgentFailure` covers background failures — things outside the command request/response cycle. Command-specific outcomes use `CommandResult`.

| Failure type | Transport |
|---|---|
| Rule failed to apply | `CommandResult { status: FAILURE }` |
| Packet capture task crashed | `AgentFailure { category: MONITORING }` |
| QUIC transport repeatedly failing | `AgentFailure { category: CONNECTIVITY }` |
| Disk buffer write rejected | `AgentFailure { category: DISK_BUFFER }` |
| Agent panic | `AgentFailure { category: AGENT_CRASH, severity: FATAL }` |
| OS-level problem (disk full, OOM) | `AgentFailure { category: SYSTEM }` |

**Local ring buffer:**

```
Path:     /var/lib/wallguard/failures.jsonl
Format:   Newline-delimited JSON (one AgentFailure per line)
Capacity: 500 entries; oldest dropped when cap is reached
fsync:    On FATAL severity only
```

```rust
pub struct FailureBuffer { path: PathBuf, max_entries: usize }

impl FailureBuffer {
    /// Synchronous. Safe to call from panic hook. Never panics.
    pub fn append_sync(&self, failure: &AgentFailure) {
        let Ok(line) = serde_json::to_string(failure) else { return };
        if let Ok(mut f) = OpenOptions::new().append(true).create(true).open(&self.path) {
            let _ = writeln!(f, "{line}");
            if failure.severity == FailureSeverity::Fatal { let _ = f.sync_all(); }
        }
    }

    pub async fn append(&self, failure: &AgentFailure) -> Result<()> { ... }
    pub async fn read_all(&self) -> Result<Vec<AgentFailure>> { ... }
    pub async fn trim_delivered(&self, up_to_unix_ms: u64) -> Result<()> { ... }
}
```

**Panic hook** — installed at startup before any async work:

```rust
fn install_panic_hook(buf: Arc<FailureBuffer>) {
    std::panic::set_hook(Box::new(move |info| {
        let location = info.location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown".to_string());
        let failure = AgentFailure {
            failure_id:  Uuid::new_v4().to_string(),
            severity:    FailureSeverity::Fatal,
            category:    FailureCategory::AgentCrash,
            message:     info.to_string(),
            context:     json!({ "location": location }).to_string(),
            occurred_at: unix_ms_now_sync(),
            is_replay:   false,
        };
        buf.append_sync(&failure);
        eprintln!("FATAL [wg-agent] panic at {location}: {info}");
    }));
}
```

**Live delivery:**

```rust
pub async fn report_failure(ctx: &Context, failure: AgentFailure) {
    ctx.failure_buffer.append(&failure).await.ok();
    if ctx.server.is_connected() {
        ctx.server.send_failure(failure).await.ok();
    }
}
```

**Replay on reconnect** — after `Welcome`, before monitoring starts:

```rust
async fn replay_buffered_failures(ctx: &Context) -> Result<()> {
    let failures = ctx.failure_buffer.read_all().await?;
    if failures.is_empty() { return Ok(()); }
    tracing::info!(count = failures.len(), "Replaying buffered failures");
    let mut latest_ts = 0u64;
    for mut f in failures {
        f.is_replay = true;
        ctx.server.send_failure(f.clone()).await?;
        latest_ts = latest_ts.max(f.occurred_at);
    }
    ctx.failure_buffer.trim_delivered(latest_ts).await
}
```

Server deduplicates by `(device_id, failure_id)`. The 500-entry cap bounds replay time.

---

## 8. Server Architecture

### 8.1 Connection Registry

One live connection per device. No `InstanceId`, no `Vec<Instance>`.

```rust
type ConnectionMap = Arc<RwLock<HashMap<DeviceId, DeviceConnection>>>;

struct DeviceConnection {
    device_id:         DeviceId,
    connected_at:      Instant,
    shutdown_tx:       oneshot::Sender<()>,
    command_tx:        mpsc::Sender<ServerMessage>,
    monitoring_status: Arc<RwLock<MonitoringStatus>>,
}
```

On a new connection with the same `DeviceId` (agent restart before TCP timeout), the stale connection receives a shutdown signal and is immediately replaced. There is no collision, no duplicate, and no database cleanup needed for live connection state.

```rust
async fn on_new_connection(ctx: &AppContext, device_id: DeviceId, stream: BidirectionalStream) {
    let mut map = ctx.connections.write().await;
    if let Some(old) = map.remove(&device_id) {
        old.shutdown_tx.send(()).ok();  // Stale handler exits on its own
    }
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let (command_tx, command_rx)   = mpsc::channel(256);
    map.insert(device_id.clone(), DeviceConnection {
        device_id: device_id.clone(),
        connected_at: Instant::now(),
        shutdown_tx,
        command_tx,
        monitoring_status: Default::default(),
    });
    drop(map);  // Release write lock before async work
    run_connection_handler(ctx, device_id, stream, command_rx, shutdown_rx).await
}
```

Live connection state is purely in-memory. There is no `instances` table. On server restart, `ConnectionMap` starts empty and agents reconnect automatically.

---

### 8.2 Graceful Restart

**Shutdown sequence (on SIGTERM):**

```
1. Stop accepting new gRPC connections.
2. Broadcast ShutdownImminent { reconnect_after_ms: 3000 } to all connected agents.
3. Wait up to 5s for active tunnel sessions to close.
4. Send Timeout errors to all pending commands.
5. Exit.
```

**Startup sequence:**

```
1. Start with empty ConnectionMap.
2. Mark any tunnel_sessions rows in "active" state older than 60s as "abandoned"
   (agents haven't reconnected yet; the sessions are gone).
3. Begin accepting connections normally.
```

No instance records are wiped on startup. The startup race condition from v1 is eliminated because there are no instance records.

---

### 8.3 Horizontal Scaling Path

The v2 design targets a single-server deployment. The path to horizontal scale when needed:

| In-memory state | External store target |
|---|---|
| `ConnectionMap` | Redis hash; key = `device_id`; value = `{ node_addr, connected_at }`; TTL = 2× heartbeat interval |
| `TunnelsManager` (active sessions) | Redis hash; key = `tunnel_id`; value = `{ node_addr, type }` |
| Pending tunnel streams | Redis pub/sub: `tunnel:{id}:ready` |

Tunnel routing across nodes: if a tunnel request arrives at node B but the agent is connected to node A, node B proxies over an internal `ForwardTunnel` gRPC channel to node A. No protocol change to the agent is required.

---

## 9. Tunnel Sessions

### 9.1 QUIC Connection Lifecycle

The agent establishes one persistent QUIC connection immediately after the gRPC control channel handshake completes.

```
Agent state: Connected
  │
  ├── gRPC control channel   → :50051 (TCP + mTLS)
  └── QUIC tunnel connection → :7777  (QUIC/UDP + mTLS, device cert)
        │
        ├── Stream N   (each new SSH / TTY / HTTP session)
        └── Datagrams  (Remote Desktop video frames)
```

If the QUIC connection drops (network interruption, server restart), the agent re-establishes it using the backoff algorithm. Active tunnel sessions backed by the lost connection are terminated.

---

### 9.2 SSH Tunnel

1. Server sends `OpenSshTunnel { command_id, tunnel_id, public_key, username }`.
2. Agent sends `CommandResult` immediately (before opening the QUIC stream) so the server knows early if the session cannot be serviced.
3. Agent opens a reliable bidirectional QUIC stream.
4. Agent sends `TunnelHello { tunnel_id }` as the first frame.
5. Server routes the stream to the `russh` SSH handler registered for that `tunnel_id`.
6. The HTTP API WebSocket endpoint relays frames between the browser and the `russh` session.

Reliability is essential for SSH: QUIC reliable streams provide the same ordering and delivery guarantees as TCP.

**Session audit:** `device_id`, `initiated_by`, `started_at`, `ended_at`, `bytes_sent`, `bytes_received` written to the `tunnel_sessions` table.

**Idle timeout:** 30 minutes of no WebSocket frames (configurable).

---

### 9.3 TTY Tunnel

Identical to SSH in transport terms (reliable bidirectional QUIC stream). A PTY is spawned on the agent; the stream carries raw terminal bytes. `russh` is not involved.

Security note: TTY runs as root. Disabled per-device by removing `Feature::TtyTunnel` from the server-side negotiated feature set in the database.

---

### 9.4 HTTP Proxy Tunnel

Each HTTP proxy session opens a reliable bidirectional QUIC stream, sends `TunnelHello { tunnel_id }`, and passes HTTP bytes through. The proxy strips the outer tunnel path prefix before forwarding to the device's local web interface (e.g., pfSense's web UI).

`pingora` is replaced with a lighter custom proxy built directly on `axum` + `hyper`. `pingora` is Linux-only and couples the server to a Linux-specific dependency. The replacement is a small `hyper` client that forwards requests over the QUIC stream with header rewriting. This handles the actual use case (proxying to a local web UI) without the full reverse-proxy machinery.

---

### 9.5 Remote Desktop Tunnel

Remote Desktop uses two QUIC primitives simultaneously:

| Channel | QUIC primitive | Direction | Content |
|---|---|---|---|
| Control stream | Reliable bidi stream | Both | Session setup, PLI, resize |
| Input stream | Reliable uni stream | Server → Agent | Keyboard/mouse events |
| Video datagrams | Unreliable (RFC 9221) | Agent → Server | H.264 NAL units |

**Why datagrams for video:** TCP and QUIC reliable streams retransmit lost packets. For video, a retransmitted 30ms-old frame arrives too late and meanwhile blocks subsequent frames (head-of-line blocking). Datagrams are dropped on loss; the decoder requests a new keyframe via PLI. The result is a brief visual artifact instead of a frozen screen.

**Video frame datagram format:**

```
VideoFrameDatagram {
  tunnel_id:    [16 bytes]  UUID
  seq:          [8 bytes]   monotonic frame counter
  frag_index:   [2 bytes]   fragment index
  frag_total:   [2 bytes]   total fragments for this frame
  keyframe:     [1 byte]    1 = IDR frame
  timestamp_ms: [8 bytes]   capture timestamp
  data:         [remaining] H.264 NAL unit bytes
}
```

Maximum datagram payload: `min(path_MTU − 40, 1200)` bytes.

**Loss recovery:** Server tracks last received `seq` per tunnel. Gap detected → PLI sent on control stream → agent forces IDR keyframe in next encode cycle (≤33ms at 30fps).

**Congestion adaptation:** If packet loss exceeds 5% or jitter exceeds 80ms over a 500ms window, the server sends `ThrottleRemoteDesktop { target_kbps }` on the control stream. The encoder reduces bitrate by 25% per command, floor 256kbps. On recovery (loss < 1% for 2s), bitrate ramps up 10% per second back to the original target.

**`webrtc` crate is removed.** ICE/STUN/TURN/SDP are not needed because the tunnel is always agent-initiated to a known server address. The `webrtc` crate added ~400 transitive dependencies for machinery this architecture never uses.

---

## 10. Firewall Configuration Management

### 10.1 Apply-and-Acknowledge

```
HTTP Client           Server                      Agent
    │                    │                           │
    │── POST /rule ──────►│                           │
    │                    │── CreateFilterRule ───────►│
    │                    │   { command_id, rule }     │  validate + apply
    │                    │◄─ CommandResult ───────────│
    │                    │   { SUCCESS, digest }      │
    │◄── 200 { digest } ─│                           │
    │                    │                           │
    │   (on failure)     │◄─ CommandResult ───────────│
    │◄── 422 { error } ──│   { FAILURE, message }    │
    │                    │                           │
    │   (on timeout)     │  (no CommandResult in 30s) │
    │◄── 504 ────────────│                           │
```

The server never reports success to the HTTP caller until it has received `CommandResult { status: SUCCESS }` from the agent.

---

### 10.2 Rollback

The server stores every `applied_digest` alongside the rule in the `firewall_rules` table.

For atomic multi-rule transactions:

```protobuf
message ApplyRuleSet {
  string         command_id      = 1;
  repeated Rule  rules           = 2;
  string         rollback_digest = 3;
}
```

The agent applies all rules in a single nftables/pf batch. On partial failure, it restores from `rollback_digest`. No partial-apply states.

---

### 10.3 State Reconciliation

On connection (after `Welcome`), the server sends `RequestConfigSnapshot`. The agent responds with the current config hash and body. The server compares against its last recorded `applied_digest`:

- If equal: no action.
- If different: emit a `config_drift` event. **Do not automatically overwrite.** Surface the drift in the API (`GET /api/v1/devices/{id}/config/drift`) so an operator can decide whether to re-push the intended config or accept the device's state.

---

## 11. Database Design

### 11.1 Engine Choice

**PostgreSQL 16 + TimescaleDB 2.x extension.**

| Need | Solution |
|---|---|
| Relational data (devices, users, rules, audit) | PostgreSQL tables, foreign keys, transactions |
| Time-series data (packet telemetry, resource metrics) | TimescaleDB hypertables (partitioned by time) |
| Aggregation queries ("average CPU over last 24h per device") | TimescaleDB continuous aggregates |
| Migrations | `sqlx migrate` — SQL files in `migrations/`, applied at server startup |
| Query correctness | `sqlx` compile-time verification (`query!` macro) |
| Multi-tenancy | `org_id` column present on every tenant-scoped table; enforced via Row-Level Security policies |

TimescaleDB is a PostgreSQL extension — same connection pool, same `sqlx` driver, same migration toolchain. No separate time-series database to operate.

---

### 11.2 Relational Schema

```sql
-- Multi-tenant root
CREATE TABLE organizations (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Human operators
CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id          UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    email           TEXT NOT NULL,
    password_hash   TEXT NOT NULL,         -- argon2id
    display_name    TEXT NOT NULL,
    role            TEXT NOT NULL,         -- 'owner' | 'admin' | 'operator' | 'viewer'
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (org_id, email)
);

-- JWT refresh tokens (access tokens are stateless; only refresh tokens stored)
CREATE TABLE refresh_tokens (
    jti         UUID PRIMARY KEY,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at  TIMESTAMPTZ NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Revoked access token JTIs (checked on every request; entries expire naturally)
CREATE TABLE revoked_tokens (
    jti         UUID PRIMARY KEY,
    expires_at  TIMESTAMPTZ NOT NULL  -- GC'd after this time
);

-- API keys for programmatic access
CREATE TABLE api_keys (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id          UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key_hash        TEXT NOT NULL,         -- argon2id of the raw key
    description     TEXT,
    role            TEXT NOT NULL,
    last_used_at    TIMESTAMPTZ,
    expires_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Server-level secrets (JWT signing key, etc.)
CREATE TABLE server_secrets (
    key     TEXT PRIMARY KEY,
    value   BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- One-time installation codes for device enrollment
CREATE TABLE installation_codes (
    code        TEXT PRIMARY KEY,
    org_id      UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    created_by  UUID NOT NULL REFERENCES users(id),
    used_at     TIMESTAMPTZ,
    expires_at  TIMESTAMPTZ NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Enrolled devices
CREATE TABLE devices (
    id              UUID PRIMARY KEY,           -- Matches CN in device cert
    org_id          UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    display_name    TEXT NOT NULL,
    firewall_kind   TEXT NOT NULL,              -- 'pfsense' | 'opnsense' | 'nftables' | 'none'
    agent_version   TEXT,
    enrolled_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at    TIMESTAMPTZ,
    features        TEXT[] NOT NULL DEFAULT '{}',  -- Negotiated on last Hello
    config_digest   TEXT,                       -- Last known applied config digest
    notes           TEXT
);

-- Per-device permission overrides (optional; supplements org-level role)
CREATE TABLE device_permissions (
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id   UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    access      TEXT NOT NULL,  -- 'allow' | 'deny'
    PRIMARY KEY (user_id, device_id)
);

-- Device certificate history (for cert rotation audit)
CREATE TABLE device_certificates (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id       UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    cert_pem        TEXT NOT NULL,
    issued_at       TIMESTAMPTZ NOT NULL,
    expires_at      TIMESTAMPTZ NOT NULL,
    revoked_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Agent failure events delivered via the control channel
CREATE TABLE device_failures (
    failure_id   UUID PRIMARY KEY,             -- From AgentFailure.failure_id
    device_id    UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    severity     TEXT NOT NULL,                -- 'warning' | 'error' | 'fatal'
    category     TEXT NOT NULL,
    message      TEXT NOT NULL,
    context      JSONB,
    occurred_at  TIMESTAMPTZ NOT NULL,
    received_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_replay    BOOLEAN NOT NULL DEFAULT FALSE
);

-- Firewall rules pushed by the server
CREATE TABLE firewall_rules (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id       UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    rule_type       TEXT NOT NULL,             -- 'filter' | 'nat' | 'alias'
    rule_data       JSONB NOT NULL,
    applied_digest  TEXT,                      -- SHA-256 of config after application
    applied_at      TIMESTAMPTZ,
    created_by      UUID REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ               -- Soft delete; rule removal via DeleteRule
);

-- Config snapshots (from RequestConfigSnapshot)
CREATE TABLE config_snapshots (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id   UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    digest      TEXT NOT NULL,
    content     BYTEA NOT NULL,
    captured_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Tunnel session audit log
CREATE TABLE tunnel_sessions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id       UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    tunnel_type     TEXT NOT NULL,             -- 'ssh' | 'tty' | 'http' | 'remote_desktop'
    initiated_by    UUID REFERENCES users(id),
    status          TEXT NOT NULL DEFAULT 'active',  -- 'active' | 'closed' | 'abandoned'
    started_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at        TIMESTAMPTZ,
    bytes_sent      BIGINT NOT NULL DEFAULT 0,
    bytes_received  BIGINT NOT NULL DEFAULT 0
);

-- Command audit log
CREATE TABLE command_log (
    id              UUID PRIMARY KEY,          -- = command_id
    device_id       UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    command_type    TEXT NOT NULL,
    initiated_by    UUID REFERENCES users(id),
    status          TEXT NOT NULL,             -- 'success' | 'failure' | 'timeout'
    error_message   TEXT,
    applied_digest  TEXT,
    sent_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    result_at       TIMESTAMPTZ
);
```

---

### 11.3 Time-Series Schema

TimescaleDB hypertables partition automatically by time. The chunk interval is chosen based on expected data rates.

```sql
-- Network packet telemetry
CREATE TABLE packets (
    time            TIMESTAMPTZ NOT NULL,
    device_id       UUID NOT NULL,
    src_ip          INET,
    dst_ip          INET,
    src_port        INTEGER,
    dst_port        INTEGER,
    protocol        SMALLINT,
    bytes           INTEGER,
    direction       TEXT         -- 'in' | 'out'
);
SELECT create_hypertable('packets', 'time', chunk_time_interval => INTERVAL '1 hour');
CREATE INDEX ON packets (device_id, time DESC);

-- Continuous aggregate: bytes per device per 5-minute bucket
CREATE MATERIALIZED VIEW packets_5m
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('5 minutes', time) AS bucket,
    device_id,
    SUM(bytes)  AS total_bytes,
    COUNT(*)    AS packet_count
FROM packets
GROUP BY bucket, device_id;

-- Resource metrics (CPU, memory, disk, network interface stats)
CREATE TABLE resource_metrics (
    time            TIMESTAMPTZ NOT NULL,
    device_id       UUID NOT NULL,
    cpu_percent     REAL,
    mem_used_bytes  BIGINT,
    mem_total_bytes BIGINT,
    disk_used_bytes BIGINT,
    disk_total_bytes BIGINT,
    load_1m         REAL,
    load_5m         REAL
);
SELECT create_hypertable('resource_metrics', 'time', chunk_time_interval => INTERVAL '4 hours');
CREATE INDEX ON resource_metrics (device_id, time DESC);

-- Monitoring status snapshots (from heartbeats, written at most once per minute per device)
CREATE TABLE device_monitoring_status (
    time                    TIMESTAMPTZ NOT NULL,
    device_id               UUID NOT NULL,
    packet_queue_depth      INTEGER,
    disk_buffer_bytes       BIGINT,
    disk_buffer_max_bytes   BIGINT,
    packets_dropped_total   BIGINT,
    packets_sent_total      BIGINT,
    degraded                BOOLEAN,
    active_tunnel_count     INTEGER
);
SELECT create_hypertable('device_monitoring_status', 'time', chunk_time_interval => INTERVAL '1 day');

-- Retention policies (data automatically dropped after configured retention)
SELECT add_retention_policy('packets',                  INTERVAL '30 days');
SELECT add_retention_policy('resource_metrics',         INTERVAL '90 days');
SELECT add_retention_policy('device_monitoring_status', INTERVAL '90 days');
```

---

### 11.4 Migrations

`sqlx migrate` runs at server startup. Migration files live in `migrations/` at the workspace root. Naming convention: `<NNN>_<description>.sql`.

```
migrations/
├── 001_initial_schema.sql          -- All relational tables
├── 002_timescale_hypertables.sql   -- Hypertable creation (requires TimescaleDB extension)
├── 003_timescale_aggregates.sql    -- Continuous aggregates
├── 004_retention_policies.sql      -- TimescaleDB retention policies
└── 005_rls_policies.sql            -- Row-Level Security for multi-tenancy
```

Migration verification at startup:

```rust
sqlx::migrate!("../../migrations")
    .run(&pool)
    .await
    .expect("Database migration failed");
```

In development (Docker Compose), TimescaleDB is available from the `timescale/timescaledb` image. In production (Kubernetes), it is the `timescale/timescaledb-ha` image for high availability.

---

## 12. Web UI Architecture

### 12.1 Stack Choice: Leptos + WASM

The web UI is built with **Leptos** — a Rust framework that compiles to WebAssembly.

**Why Leptos:**

| Requirement | Leptos | React/Vue/Svelte |
|---|---|---|
| No client-side runtime | ✓ (compiled WASM) | ✗ (requires JS framework runtime) |
| Compiled language | ✓ (Rust → WASM) | ✗ (TypeScript transpiled, not compiled) |
| Shared domain types with server | ✓ (`wg-shared` crate, same types) | ✗ (would require TypeScript types that drift from Rust) |
| Type-safe API calls | ✓ (same types in server and UI) | ✗ (OpenAPI-generated types add a code generation step) |
| Binary distribution | Single WASM + minimal JS | Multiple NPM bundles |

The "no runtime on the client" requirement means: no Node.js on the server side, and no JavaScript framework runtime in the browser. Leptos/WASM satisfies this — the browser loads `wasm32` machine code, not an interpreted framework.

**What Leptos does not eliminate:** A small JS bootstrap (~2 KiB, generated by `trunk`) is still needed to load the WASM module. This is unavoidable in any browser WASM application. There is no React runtime, no virtual DOM library, no npm package resolving at runtime.

---

### 12.2 Crate Layout

```
crates/wg-ui/
├── Cargo.toml
├── Trunk.toml             # trunk build tool config
├── index.html             # WASM entry point
├── src/
│   ├── main.rs            # app bootstrap
│   ├── app.rs             # root component, router
│   ├── api/               # Type-safe API client (uses reqwest/wasm)
│   │   ├── mod.rs
│   │   ├── devices.rs
│   │   ├── auth.rs
│   │   └── ...
│   ├── pages/
│   │   ├── login.rs
│   │   ├── dashboard.rs
│   │   ├── devices/
│   │   │   ├── list.rs
│   │   │   ├── detail.rs
│   │   │   ├── tunnels.rs
│   │   │   └── failures.rs
│   │   ├── rules.rs
│   │   └── settings.rs
│   └── components/
│       ├── device_card.rs
│       ├── status_badge.rs
│       ├── terminal.rs    # xterm.js WebSocket wrapper
│       ├── packet_chart.rs
│       └── ...
└── public/
    └── style.css          # Plain CSS; no Tailwind or CSS-in-JS runtime
```

`wg-shared` is a dependency of both `wg-server` and `wg-ui`. Domain types like `Device`, `DeviceStatus`, `TunnelSession`, `AgentFailure`, `FirewallRule` are defined once and used in both crates. API response structs are defined in `wg-shared` with `serde` derives, so the serialization format is guaranteed to match.

---

### 12.3 Page Structure

| Page | Route | Min Role |
|---|---|---|
| Login | `/login` | — |
| Dashboard | `/` | viewer |
| Device list | `/devices` | viewer |
| Device detail | `/devices/{id}` | viewer |
| Device tunnels | `/devices/{id}/tunnels` | operator |
| Device failures | `/devices/{id}/failures` | operator |
| Device config | `/devices/{id}/config` | operator |
| Firewall rules | `/devices/{id}/rules` | operator |
| Tunnel session (SSH) | `/devices/{id}/ssh` | operator |
| Tunnel session (TTY) | `/devices/{id}/tty` | operator |
| Tunnel session (HTTP proxy) | `/devices/{id}/http` | operator |
| Remote Desktop | `/devices/{id}/desktop` | operator |
| Org settings | `/settings` | admin |
| User management | `/settings/users` | admin |

**Dashboard** shows:
- Connected device count, degraded device count.
- Recent failures across all devices.
- Active tunnel session count.
- Time-series chart: total bytes/s across all devices (5-minute aggregates, last 24h).

**Device detail** shows:
- Connection status (live, last seen).
- CPU/memory/disk charts (resource_metrics, last 1h).
- Packet rate chart (packets_5m, last 1h).
- Active tunnel sessions.
- Recent failures.
- Quick-launch tunnel buttons.
- Config drift indicator (if `config_digest` mismatches).

---

### 12.4 Real-time Updates

**Server-Sent Events (SSE):** Used for dashboard and device list live updates. The server pushes device status change events (connected/disconnected, degraded/recovered, new failure) over an SSE endpoint. The Leptos UI subscribes with `EventSource` and updates reactive signals.

```
GET /api/v1/events
  Content-Type: text/event-stream
  Authorization: Bearer <token>

event: device_status
data: {"device_id":"...","status":"degraded","occurred_at":...}

event: device_connected
data: {"device_id":"...","agent_version":"0.2.0"}

event: new_failure
data: {"device_id":"...","severity":"fatal","message":"..."}
```

**WebSocket:** Used for SSH, TTY, and Remote Desktop sessions. Each tunnel type has its own WebSocket endpoint:

```
WS /api/v1/devices/{id}/ssh/{session_id}
WS /api/v1/devices/{id}/tty/{session_id}
WS /api/v1/devices/{id}/desktop/{session_id}
```

The SSH and TTY sessions relay raw terminal bytes. The UI uses `xterm.js` (a standalone JS terminal emulator, not a framework) loaded as a static asset.

The Remote Desktop session uses a simple binary protocol over WebSocket: H.264 NAL units arriving at the server over QUIC datagrams are forwarded to the WebSocket. The browser decodes using the WebCodecs API (available in all modern browsers).

---

### 12.5 Build & Serving

**Build:** `trunk build --release` compiles `wg-ui` to WASM and generates the asset bundle in `dist/`.

**Integration with server:** The server embeds the compiled WASM assets via `rust-embed`:

```rust
#[derive(RustEmbed)]
#[folder = "../../crates/wg-ui/dist/"]
struct UiAssets;
```

The `axum` router serves UI assets at `/` and all non-API paths. API routes are served at `/api/v1/`. This means the entire system — API and UI — is a single binary with no separate web server needed.

**Cache headers:** WASM and JS assets are served with `Cache-Control: max-age=31536000, immutable` (filenames are content-hashed by trunk). `index.html` is served with `Cache-Control: no-cache`.

**CI build:** The CI pipeline builds the WASM bundle in one stage (`Dockerfile.ui`) and the server binary in another (`Dockerfile.server`), then combines them in the final image. This keeps WASM toolchain dependencies out of the server build image.

---

## 13. Observability

### 13.1 Structured Logging

All components use `tracing` + `tracing-subscriber`. In development: pretty-printed with color. In production: JSON (machine-parseable for log aggregation).

```rust
tracing_subscriber::fmt()
    .json()
    .with_env_filter(EnvFilter::from_default_env())
    .with_current_span(true)
    .with_span_events(FmtSpan::CLOSE)
    .init();
```

**Span hierarchy (server):**

```
request{request_id="...", method="POST", path="/api/v1/devices/..."}
  └── connection{device_id="...", conn_id="...", remote_addr="..."}
        ├── command{command_id="...", type="CreateFilterRule"}
        └── tunnel{tunnel_id="...", type="SSH"}
```

Every log line inside a span automatically carries the span's context fields. `device_id` is attached at connection establishment and propagates to every log line for that connection without per-call boilerplate.

---

### 13.2 Metrics

`metrics` facade with `metrics-exporter-prometheus`.

**Server metrics:**

```
wg_connected_agents_total                            gauge
wg_active_tunnels_total{type}                        gauge
wg_commands_sent_total{type,result}                  counter
wg_packets_received_total                            counter
wg_rpc_duration_seconds{method}                      histogram
wg_http_request_duration_seconds{method,route,code}  histogram
wg_agent_heartbeat_age_seconds{device_id}            gauge
wg_agent_degraded{device_id}                         gauge (0 or 1)
wg_agent_disk_buffer_bytes{device_id}                gauge
wg_agent_packets_dropped_total{device_id}            counter
wg_agent_failures_total{device_id,severity,category} counter
wg_agent_crash_total{device_id}                      counter  (FATAL only; alert on this)
wg_db_query_duration_seconds{query}                  histogram
```

**Agent metrics** (scraped by Prometheus from `:9090/metrics`):

```
wg_agent_capture_queue_depth         gauge
wg_agent_packets_sent_total          counter
wg_agent_packets_dropped_total       counter
wg_agent_disk_buffer_bytes           gauge
wg_agent_reconnect_attempts_total    counter
wg_agent_connection_state{state}     gauge (1 for active state)
wg_agent_heartbeat_rtt_ms            histogram
wg_agent_failures_total{sev,cat}     counter
wg_agent_failure_buffer_entries      gauge
```

Prometheus scraping of agents is optional and disabled by default (`metrics_port = 0`). For firewall appliances, the agent's Prometheus endpoint can be scraped by a Prometheus instance on the same management network.

---

### 13.3 Distributed Tracing

End-to-end tracing across HTTP API → server → agent → firewall:

- Each HTTP request generates a `trace_id` (W3C Trace Context format).
- The `trace_id` is embedded in the `command_id` prefix so it can be correlated in logs even without a trace collector.
- The server attaches `trace_id` to its outbound command span.
- The agent attaches it to its execution span.
- All spans exported via `opentelemetry-otlp` to a configurable collector (Jaeger, Grafana Tempo, or any OTLP-compatible backend).
- If `OTLP_ENDPOINT` is empty, tracing is a no-op (zero overhead via `tracing`/`opentelemetry` integration).

---

## 14. Agent Lifecycle

### 14.1 Installation

After installation, the filesystem contains:

```
/usr/sbin/wg-agent           (binary, 0755)
/usr/sbin/wg-cli             (binary, 0755)
/etc/wallguard/              (directory, 0700, root)
/var/lib/wallguard/          (directory, 0700, root)
/var/lib/wallguard/buffer/   (directory, 0700, root)
```

On first run with no `/etc/wallguard/device.crt`:
1. Agent starts in `Provisioning` state.
2. Waits for `wg-cli enroll <code> --server <url> --firewall <kind>`.
3. Enrollment flow (§4.2) runs.
4. On success, `config.toml` is written and the agent transitions to `Connecting`.

---

### 14.2 Autostart

`wg-cli autostart enable` installs and starts the service:

| Platform | Service manager | File installed |
|---|---|---|
| Linux | systemd | `/etc/systemd/system/wg-agent.service` |
| FreeBSD | rc.d | `/usr/local/etc/rc.d/wg_agent` |

The systemd unit:

```ini
[Unit]
Description=WallGuard Agent
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/sbin/wg-agent
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

`wg-cli autostart enable` calls `systemctl enable --now wg-agent` and polls `wg-cli status` for up to 10 seconds to confirm the service is running.

---

### 14.3 Upgrades & Cert Renewal

**Binary upgrade:**

1. New binary placed at `/usr/sbin/wg-agent.new`.
2. `wg-cli upgrade`:
   a. Sends `GracefulRestart` via Unix socket.
   b. Daemon finishes in-flight commands (max 10s) then exits with code 0.
   c. Package post-install script renames `.new` to live binary.
   d. Service manager restarts the daemon.
3. `/etc/wallguard/` and device cert are untouched.

**Certificate renewal** (30 days before expiry):

1. Server sends `RenewCertificateRequest`.
2. Agent generates a new Ed25519 keypair and CSR.
3. Agent sends `RenewCertificateResponse { csr_pem }`.
4. Server signs the new cert and sends `SetCertificate { cert_pem }` (a `ServerMessage` variant).
5. Agent writes the new cert + key atomically: write to `.new` files, `fsync`, rename over the old files.
6. Agent reconnects using the new cert on next cycle (existing connection continues until natural disconnect).

---

## 15. Infrastructure & Deployment

### 15.1 Development: Docker Compose

```yaml
# docker-compose.yml
services:

  postgres:
    image: timescale/timescaledb:latest-pg16
    environment:
      POSTGRES_DB:       wallguard
      POSTGRES_USER:     wallguard
      POSTGRES_PASSWORD: dev_password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

  server:
    build:
      context: .
      dockerfile: Dockerfile.server
    environment:
      DATABASE_URL:              postgres://wallguard:dev_password@postgres:5432/wallguard
      CONTROL_SERVICE_ADDR:      0.0.0.0:50051
      HTTP_API_ADDR:             0.0.0.0:4444
      REVERSE_TUNNEL_QUIC_ADDR:  0.0.0.0:7777
      REVERSE_TUNNEL_TCP_ADDR:   0.0.0.0:7778
      TLS_CA_CERT_PATH:          /etc/wallguard-server/ca.crt
      TLS_CA_KEY_PATH:           /etc/wallguard-server/ca.key
      TLS_SERVER_CERT_PATH:      /etc/wallguard-server/server.crt
      TLS_SERVER_KEY_PATH:       /etc/wallguard-server/server.key
      RUST_LOG:                  info,wg_server=debug
      LOG_FORMAT:                pretty
    ports:
      - "4444:4444"
      - "50051:50051"
      - "7777:7777/udp"
      - "7778:7778"
      - "9090:9090"
    depends_on:
      - postgres
    volumes:
      - ./dev-certs:/etc/wallguard-server:ro

  prometheus:
    image: prom/prometheus:latest
    volumes:
      - ./dev/prometheus.yml:/etc/prometheus/prometheus.yml:ro
    ports:
      - "9091:9090"

  grafana:
    image: grafana/grafana:latest
    environment:
      GF_SECURITY_ADMIN_PASSWORD: dev_password
    ports:
      - "3000:3000"
    depends_on:
      - prometheus

volumes:
  postgres_data:
```

The `dev-certs/` directory is generated by `scripts/dev-certs.sh` which uses `rcgen` (via a small Rust binary) to generate the CA, intermediate CA, and server cert for local development.

---

### 15.2 Production: Kubernetes + Helm

The Helm chart at `helm/wallguard/` deploys:

```
Deployments:
  wallguard-server     (replicas: 2+)  — gRPC + HTTP API + QUIC
    Anti-affinity: spread across nodes

Services:
  wallguard-grpc       ClusterIP   :50051
  wallguard-api        LoadBalancer :4444    (or Ingress)
  wallguard-quic       LoadBalancer :7777/UDP
  wallguard-tcp-tunnel LoadBalancer :7778

ConfigMaps:
  wallguard-config     (non-secret config values)

Secrets:
  wallguard-tls        (CA cert, server cert, server key)
  wallguard-db         (DATABASE_URL)

PersistentVolumeClaims:
  (none — server is stateless; DB is external)
```

**Database:** TimescaleDB runs separately, either as a managed PostgreSQL service with TimescaleDB extension enabled, or as the `timescale/timescaledb-ha` StatefulSet inside the cluster.

**TLS termination:** QUIC cannot be terminated by an L7 load balancer (it is end-to-end authenticated via device certs). The `wallguard-quic` service uses a `LoadBalancer` with UDP support, passing traffic directly to the server pods. The gRPC service also uses pass-through TLS (mTLS to device certs). The HTTP API for the web UI can be placed behind an L7 ingress that terminates TLS.

**Config values:**

```yaml
# helm/wallguard/values.yaml
replicaCount: 2

image:
  repository: ghcr.io/your-org/wallguard-server
  tag: ""  # defaults to chart appVersion

server:
  logFormat: json
  minAgentProtocolVersion: 2
  commandTimeoutSecs: 30
  tunnelConnectTimeoutSecs: 5
  shutdownGracePeriodSecs: 10

tls:
  existingSecret: wallguard-tls  # Kubernetes Secret with ca.crt, server.crt, server.key

database:
  existingSecret: wallguard-db   # Kubernetes Secret with DATABASE_URL

ports:
  grpc: 50051
  api: 4444
  quicUdp: 7777
  tcpTunnel: 7778
  metrics: 9090
```

**Rolling updates:** The `ShutdownImminent { reconnect_after_ms: 3000 }` broadcast lets agents reconnect to the new pod before the old one exits. Combined with 2+ replicas and the default Kubernetes rolling update strategy, agent connections are not interrupted by server deployments.

---

### 15.3 Database Provisioning

Migrations run automatically at server startup:

```rust
sqlx::migrate!("../../migrations").run(&pool).await?;
```

`sqlx migrate` is idempotent — already-applied migrations are skipped. If a migration fails, the server exits immediately rather than starting in a partially migrated state.

For the TimescaleDB extension, the `CREATE EXTENSION IF NOT EXISTS timescaledb;` is the first statement in `migrations/001_initial_schema.sql`. If TimescaleDB is not installed in the PostgreSQL cluster, this fails and the server exits with a clear error. This is intentional — the system requires TimescaleDB.

**Initial data:** A `scripts/seed.sh` script creates the first organization and owner user for initial setup. On Kubernetes, this is a one-time `Job`.

---

## 16. Configuration Reference

### Agent (`/etc/wallguard/config.toml`)

```toml
[server]
url      = "https://server:50051"   # gRPC control channel
firewall = "opnsense"               # "pfsense" | "opnsense" | "nftables" | "none"

[tls]
ca_cert_path     = "/etc/wallguard/ca.crt"
device_cert_path = "/etc/wallguard/device.crt"
device_key_path  = "/etc/wallguard/device.key"

[tunnel]
quic_addr         = "server:7777"
tcp_fallback_addr = "server:7778"
prefer            = "quic"          # "quic" | "tcp"
quic_idle_timeout_s = 60
quic_keepalive_s    = 15
max_concurrent      = 64

[transmission]
capture_queue_cap  = 50000
batch_size         = 1000
batch_interval_ms  = 500
disk_buffer_path   = "/var/lib/wallguard/buffer"
disk_buffer_max_mb = 256

[observability]
metrics_port  = 0            # 0 = disabled; set to 9090 to enable Prometheus scraping
log_format    = "json"       # "json" | "pretty"
otlp_endpoint = ""           # empty = tracing disabled

# Only present in --features remote-desktop builds:
# [remote_desktop]
# max_fps        = 30
# max_kbps       = 4000
# capture_cursor = true
```

### Server (environment variables)

```
DATABASE_URL                      postgres://user:pass@host:5432/wallguard

TLS_CA_CERT_PATH                  /etc/wallguard-server/ca.crt
TLS_CA_KEY_PATH                   /etc/wallguard-server/ca.key
TLS_SERVER_CERT_PATH              /etc/wallguard-server/server.crt
TLS_SERVER_KEY_PATH               /etc/wallguard-server/server.key

CONTROL_SERVICE_ADDR              0.0.0.0:50051
HTTP_API_ADDR                     0.0.0.0:4444
REVERSE_TUNNEL_QUIC_ADDR          0.0.0.0:7777
REVERSE_TUNNEL_TCP_ADDR           0.0.0.0:7778
METRICS_ADDR                      0.0.0.0:9090

MIN_AGENT_PROTOCOL_VERSION        2
COMMAND_TIMEOUT_SECS              30
TUNNEL_CONNECT_TIMEOUT_SECS       5
SHUTDOWN_GRACE_PERIOD_SECS        10

RUST_LOG                          info,wg_server=debug
LOG_FORMAT                        json
OTLP_ENDPOINT                     (empty = disabled)
```

---

## 17. Testing Strategy

### Unit Tests

Every module with logic has colocated `#[cfg(test)]` tests. No unit test touches the database or the network.

| Module | Key tests |
|---|---|
| `Backoff::next_delay()` | Delay distribution; jitter bounds; reset; proptest fuzz |
| `DiskBuffer::try_write()` | Write success; full buffer; low-disk guard; never panics |
| `derive_capabilities()` | Every `FirewallKind` produces the correct feature set; feature flags respected |
| `fireparse::*` | Round-trip tests for all create operations; parser accepts valid config; rejects corrupted config |
| `CommandResult` tracking | Timeout sweep; correct wakeup on ack; no panic on double-delivery |
| JWT auth | Valid token accepted; expired token rejected; revoked token rejected; role insufficient rejected |
| RBAC middleware | All roles against all routes; device permission overrides |
| `FailureBuffer` | Append; cap enforcement; read_all; trim_delivered |

**No-panic policy:** The workspace `Cargo.toml` includes a `test-abort` profile:

```toml
[profile.test-abort]
inherits = "test"
panic    = "abort"
```

Any `.unwrap()` or `.expect()` reached during tests aborts the process, making panics immediately visible.

---

### Integration Tests (`wg-testkit`)

`wg-testkit` provides:

- `TestServer` — spawns a real `wg-server` against a test PostgreSQL database (spun up via `testcontainers`).
- `TestAgent` — spawns a real `wg-agent` with a cert signed by a test CA.
- `TestCa` — generates an in-memory CA and issues device certs for test agents.
- `CapturedPackets` — a fake libpcap feed that injects synthetic packets.

Key integration tests:

| Test | Verifies |
|---|---|
| `enrollment_flow` | CSR signing, cert file permissions, agent reconnects with issued cert |
| `monitoring_enable` | `SetMonitoring` → agent starts capture → packets arrive at server → stored in DB |
| `rule_push_success` | `CreateFilterRule` → `CommandResult::SUCCESS` → HTTP 200 with digest |
| `rule_push_failure` | Invalid rule → `CommandResult::FAILURE` → HTTP 422 with message |
| `rule_push_timeout` | Agent disconnected mid-command → HTTP 504 after 30s |
| `atomic_rule_set` | `ApplyRuleSet` with rollback_digest → all-or-nothing application |
| `config_drift_detection` | Config changed out of band → server surfaces drift after reconnect |
| `ssh_tunnel` | Full tunnel open → QUIC stream established → SSH handshake completes |
| `tty_tunnel` | Full tunnel open → PTY spawned → bytes flow bidirectionally |
| `tunnel_multiplexing` | SSH + TTY opened simultaneously → neither blocks the other |
| `remote_desktop_tunnel` | Control stream + datagram flow → server receives frames |
| `remote_desktop_pli` | Server detects datagram gap → PLI sent → agent emits IDR within 2 cycles |
| `quic_fallback_to_tcp` | QUIC UDP blocked → agent falls back → SSH works; RD unavailable |
| `reconnect_backoff` | Server killed → agent reconnects with increasing delays |
| `reconnect_replaces_stale` | Agent reconnects with same device_id → stale connection closed cleanly |
| `disk_buffer_drain` | Server disconnects → agent buffers to disk → reconnects → buffer drained first |
| `heartbeat_timeout` | Server stops heartbeats → agent tears down stream within 40s |
| `version_rejection` | Server sets `MIN_AGENT_PROTOCOL_VERSION=99` → agent stops retrying |
| `cert_renewal` | `RenewCertificateRequest` → new cert issued → agent reconnects with new cert |
| `failure_live_delivery` | Background task fails → `AgentFailure` arrives at server → stored in DB → HTTP API returns it |
| `failure_buffer_replay` | Server down while agent fails → reconnect → all buffered failures delivered with `is_replay=true` |
| `failure_buffer_cap` | 600 failures injected offline → only 500 delivered (oldest dropped) |
| `panic_hook_records` | Synthetic panic in test subprocess → `failures.jsonl` has FATAL entry → next start replays it |
| `failure_dedup` | Same `failure_id` delivered twice → server stores one record |
| `auth_login_logout` | Valid credentials → JWT issued → logout → token revoked |
| `auth_rbac` | Viewer cannot push rules; operator can; admin can manage users |
| `api_key_auth` | API key accepted; expired key rejected; wrong key rejected |

### Contract Tests

Protobuf breaking-change detection via `buf breaking` run in CI against the baseline descriptor in `wg-shared/tests/descriptors/`. A protobuf schema change that breaks wire compatibility fails CI.

### CI Pipeline

```
cargo fmt --check
cargo clippy -- -D warnings
cargo test                                    # unit tests
cargo test --profile test-abort               # panic policy check
cargo nextest run -p wg-testkit               # integration tests
buf breaking --against .git#branch=main       # proto breaking change check
trunk build --release -p wg-ui               # UI WASM build
docker build -f Dockerfile.server .          # server image smoke test
```

---

## 18. Dependency Inventory

### Agent (`wg-agent`)

| Crate | Purpose |
|---|---|
| `tokio` | Async runtime |
| `tonic` + `prost` | gRPC client |
| `quinn` | QUIC transport for reverse tunnel (replaces raw TCP port 7777) |
| `rustls` | TLS; `AcceptAllVerifier` deleted |
| `rcgen` | CSR generation at enrollment time |
| `tracing` + `tracing-subscriber` | Structured logging |
| `metrics` + `metrics-exporter-prometheus` | Agent-side Prometheus metrics |
| `serde` + `serde_json` | JSON serialization for failure buffer |
| `toml` | `config.toml` parsing |
| `uuid` | Command IDs, tunnel IDs, failure IDs |
| `rand` | Jitter in backoff |
| `nullnet-traffic-monitor` | Packet capture (libpcap wrapper) |
| `nullnet-libresmon` | Resource monitoring |
| `nftables` | nftables JSON ruleset |
| `xmltree` | pfSense/OPNsense `config.xml` |
| `portable-pty` | PTY for TTY sessions |
| `russh` | SSH server for SSH tunnel sessions |
| `captis` | Screen capture (remote-desktop feature only) |
| `openh264` | H.264 encoding (remote-desktop feature only) |
| `enigo` | Keyboard/mouse injection (remote-desktop feature only) |
| `proptest` | Property-based testing |

**Removed from agent:**
- `webrtc` — eliminated entirely; QUIC datagrams replace the WebRTC transport layer
- `nullnet-libtoken` — JWT logic is no longer needed on the agent; identity is the device cert
- `nullnet-libdatastore` — the agent has no direct database access

### Server (`wg-server`)

| Crate | Purpose |
|---|---|
| `tokio` | Async runtime |
| `tonic` + `prost` | gRPC server |
| `axum` | HTTP API (replaces `actix-web`) |
| `tower` | Middleware stack (auth, RBAC, request ID) |
| `hyper` | HTTP client for HTTP proxy tunnel |
| `quinn` | QUIC endpoint for reverse tunnel |
| `rustls` | TLS |
| `rcgen` | Signing device CSRs |
| `sqlx` | PostgreSQL driver with compile-time query verification |
| `argon2` | Password hashing |
| `jsonwebtoken` | JWT issuance and validation |
| `tracing` + `tracing-subscriber` | Structured logging |
| `metrics` + `metrics-exporter-prometheus` | Server-side Prometheus metrics |
| `opentelemetry-otlp` | Distributed tracing export |
| `russh` | SSH server for SSH tunnel sessions |
| `serde` + `serde_json` | JSON |
| `uuid` | IDs |
| `rust-embed` | Embeds compiled `wg-ui` WASM assets into server binary |
| `tokio-stream` | SSE streaming |

**Removed from server:**
- `actix-web` → replaced by `axum`
- `pingora` → replaced by a small `hyper`-based proxy (fewer dependencies, no Linux-only constraint)
- `webrtc` → eliminated; QUIC datagrams replace WebRTC
- `nullnet-libdatastore` → replaced by `sqlx` + first-party schema
- `nullnet-libtoken` → replaced by `jsonwebtoken` + `argon2`

### UI (`wg-ui`)

| Crate | Purpose |
|---|---|
| `leptos` | Reactive UI framework (compiles to WASM) |
| `leptos_router` | Client-side routing |
| `reqwest` (wasm feature) | HTTP API calls from WASM |
| `web-sys` | Browser APIs (WebSocket, SSE, WebCodecs) |
| `gloo` | Idiomatic wrappers for browser APIs |
| `serde` + `serde_json` | JSON deserialization of API responses |
| `wg-shared` | Domain types (shared with server) |

### Shared (`wg-shared`)

| Crate | Purpose |
|---|---|
| `serde` | Serialization (no_std compatible subset for wasm) |
| `uuid` | Domain IDs |
| `prost-types` | Protobuf generated types |
| `time` or `chrono` | Timestamp types (wasm-compatible) |
