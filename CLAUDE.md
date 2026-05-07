# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
# Build all native crates (agent, server, cli, shared)
cargo build --release

# Build specific crate
cargo build -p wg-agent --release
cargo build -p wg-server --release
cargo build -p wg-cli --release

# Build WASM UI (requires trunk: cargo install trunk)
cd crates/wg-ui && trunk build --release

# Full dev stack (includes TimescaleDB, Prometheus, Grafana)
docker compose up --build
```

Protobuf code is generated automatically during `cargo build` via each crate's `build.rs` using `tonic_build`. No manual `protoc` invocation needed.

## Testing

```bash
# Unit tests (all crates except WASM UI)
cargo test --workspace --exclude wg-ui

# Same but with panic-abort to catch .unwrap()/.expect() failures
cargo test --workspace --exclude wg-ui --profile test-abort

# Integration tests (requires running TimescaleDB — use Docker Compose first)
cargo nextest run -p wg-testkit

# Proto breaking-change check against main
buf breaking --against '.git#branch=main'
```

## Linting

```bash
cargo fmt --all
cargo clippy --workspace --exclude wg-ui -- -D warnings
buf lint proto/
```

CI enforces `cargo fmt --check`, `clippy -D warnings`, both test profiles, integration tests, proto breaking-change detection, and UI WASM build.

## Architecture Overview

WallGuard is a distributed remote management and monitoring platform for firewall appliances (pfSense, OPNsense, Linux nftables). It follows a **control-plane + agent** architecture.

### Crate Layout

| Crate | Role |
|-------|------|
| `wg-agent` | Daemon that runs on managed firewall devices |
| `wg-server` | Central control plane: gRPC, HTTP REST API, tunnel relay |
| `wg-cli` | Device enrollment and lifecycle CLI (`enroll`, `status`, `upgrade`) |
| `wg-ui` | Web dashboard — Leptos SSR compiled to WASM (`wasm32-unknown-unknown`) |
| `wg-shared` | Common types with no I/O; compiles to both native and WASM |
| `wg-testkit` | Integration tests (uses testcontainers for TimescaleDB) |

`wg-ui` is excluded from the default workspace build because it targets WASM; build it with `trunk` from `crates/wg-ui/`.

### Protocol Layer (proto/)

Five `.proto` files define the full API surface:

- **`control.proto`** — Bidirectional gRPC streaming between agent and server. Agent sends `Hello`, `Heartbeat`, `CommandResult`, `AgentFailure`, `ConfigSnapshot`. Server sends `Welcome`, `SetMonitoring`, `OpenSshTunnel`, `OpenTtyTunnel`, `OpenHttpTunnel`, `OpenRemoteDesktopTunnel`, `ApplyRuleSet`, etc.
- **`provisioning.proto`** — One-shot `Enroll(CSR) → (device_id, signed_cert, ca_cert)`. Uses server-auth TLS only (no client cert yet).
- **`data.proto`** — Bidirectional streaming for `UploadPackets` and `UploadResourceMetrics` with `BatchAck` flow control.
- **`cli.proto`** — Local Unix socket API for `wg-cli` to query agent `Status` or trigger `GracefulRestart`.
- **`models.proto`** — Shared domain types: `FilterRule`, `NatRule`, `Alias`, `Rule`.

### Transport Channels

| Channel | Protocol | Auth | Default Port | Direction |
|---------|----------|------|-------------|-----------|
| Provisioning gRPC | gRPC + TLS | Server-auth | 50051 | Agent → Server (one-shot) |
| Control gRPC | gRPC + mTLS | Mutual (device cert) | 50052 | Bidirectional |
| Data gRPC | gRPC + mTLS | Mutual | 50051 | Agent → Server |
| Reverse tunnel | QUIC + mTLS | Mutual | 7777/UDP | Bidirectional |
| TCP fallback | TLS | mTLS | 7778/TCP | Bidirectional (when UDP blocked) |
| Web API / UI | HTTP + WebSocket | JWT | 4444 | Browser ↔ Server |
| Agent CLI socket | Unix socket | N/A | `/run/wallguard/agent.sock` | Local only |

### Agent Internals (`wg-agent`)

The agent is a Tokio async state machine: **Provisioning → Idle ↔ Connecting → Connected** with exponential backoff on failures.

Data pipelines are staged:
```
[pcap Capture] → [Sampler/Throttle] → [DiskBuffer] → [Batcher] → [gRPC Upload]
[System Metrics]                    → [DiskBuffer] → [Batcher] → [gRPC Upload]
[AgentFailures]                     → [FailureBuffer]           → [Control Channel]
```

Key modules: `state_machine.rs`, `control_channel.rs`, `pipeline/`, `capture/`, `tunnel/`, `disk_buffer.rs`, `failure_buffer.rs`.

### Server Internals (`wg-server`)

The server runs three gRPC listeners and one Axum HTTP listener concurrently. Key components:

- `connection_registry.rs` — Maps `device_id → active gRPC stream` for routing server-initiated messages.
- `command_tracker.rs` — Maps `command_id → oneshot channel` for request-response pairing on async commands.
- `pki/` — Intermediate CA that signs device certificates during enrollment.
- `grpc/{provisioning,control,data}.rs` — Three independent gRPC service implementations.
- `api/` — Axum REST routes + embedded WASM UI serving.
- `tunnel/` — QUIC and TCP-TLS reverse tunnel listener and relay registry.

### Database

PostgreSQL 16 + TimescaleDB extension. Migrations in `migrations/` are applied at server startup via SQLx.

- Relational: `organizations`, `users`, `devices`, `installation_codes`, `device_certificates`, `tunnel_sessions`, `command_log`, `device_failures`
- TimescaleDB hypertables: `packets`, `resource_metrics` (with continuous aggregates and retention policies)

### UI (`wg-ui`)

Built with Leptos (Rust SSR framework) compiled to WASM. The server embeds the compiled `dist/` at build time and serves it as static files. Auth tokens are stored in `localStorage`; real-time updates use SSE.

## Key Design Constraints

- **No shell execution on devices** — all agent operations are done via Rust code, never `Command::new("sh")`. Firewall rule changes use native library calls or structured APIs.
- **`wg-shared` must stay I/O-free** — it compiles to both native and WASM; adding `tokio` or `std::net` breaks WASM.
- **mTLS everywhere after enrollment** — the control and data channels require a valid device certificate. Provisioning is the only unauthenticated path.
- **Disk buffer as reliability guarantee** — agents buffer telemetry to disk before transmitting so data survives network interruptions and restarts.
- **Rust 1.88 stable** — pinned in `rust-toolchain.toml`; do not require nightly features.
