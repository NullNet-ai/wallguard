# WallGuard

[![Build CI](https://github.com/NullNet-ai/wallguard/actions/workflows/build.yml/badge.svg)](https://github.com/NullNet-ai/wallguard/actions/workflows/build.yml)
[![Server Docker](https://github.com/NullNet-ai/wallguard/actions/workflows/docker.yml/badge.svg)](https://github.com/NullNet-ai/wallguard/actions/workflows/docker.yml)

Once installed, WallGuard continuously collects system and network telemetry: CPU, memory, disk, and process metrics; live network traffic statistics; and configuration file change events. It also exposes secure remote-access capabilities, including SSH, TTY, and graphical UI sessions, so operators can reach any enrolled device directly through the NullNet console without needing to open inbound firewall ports or maintain VPN tunnels. WallGuard is built in Rust for low overhead and ships as a single self-contained binary with no runtime dependencies beyond the platform's standard networking stack.

## Installation

| Platform | Package | Minimum version |
|---|---|---|
| Debian / Ubuntu | `.deb` or `install.sh` | Ubuntu 18.04 / Debian 10 |
| Fedora / CentOS / RHEL | `.rpm` or `install.sh` | CentOS 7 |
| macOS | `.dmg` (universal) | macOS 10.15 |
| Windows | `.msi` | Windows 10 / 11 |
| FreeBSD | `.pkg` | FreeBSD 14 |
| pfSense / OPNsense | manual | — |

**One-line installer (Linux / FreeBSD):**

```sh
curl -fsSL https://github.com/NullNet-ai/wallguard/releases/latest/download/install.sh | sudo bash
```

Pre-built packages for every supported platform are attached to each [GitHub Release](https://github.com/NullNet-ai/wallguard/releases).

## Building from source

You will need Rust (latest stable), `protobuf-compiler`, and `libpcap-dev` (or the platform equivalent). Clone the repository and run:

```sh
cargo build --release -p wallguard -p wallguard-cli
```

The agent binary (`wallguard`) and the control CLI (`wallguard-cli`) will be placed in `target/release/`. See `packbuild.sh` for the full packaging workflow and `CLAUDE.md` for development notes.

WallGuard depends on a separate **datastore** service for persistence. Start that first:
