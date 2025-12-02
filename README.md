# WallGuard

[![Build CI](https://github.com/NullNet-ai/wallguard/actions/workflows/build.yml/badge.svg)](https://github.com/NullNet-ai/wallguard/actions/workflows/build.yml)
[![Server Docker](https://github.com/NullNet-ai/wallguard/actions/workflows/docker.yml/badge.svg)](https://github.com/NullNet-ai/wallguard/actions/workflows/docker.yml)

**WallGuard** is a part of **Nullnet**, built to work with firewalls and other network-facing systems. It consists of a server and a set of agents installed on target machines. The server manages the agents, collects data, and provides access to remote systems.

## Overivew

WallGuard helps monitor system state and network activity, and allows secure remote access to devices. It's useful for managing machines that are part of a firewall setup or are otherwise not easy to reach directly.

### Features

1. Configuration Monitoring
   Watches for changes in system or network configuration files.

2. Network Traffic Monitoring
   Tracks basic traffic information.

3. System Monitoring
   Gathers CPU, memory, disk, and process data.

4. Remote Access
   Supports remote sessions through:

- SSH – Secure shell

- TTY – Command-line terminal access

- UI – Graphical remote access (only on some systems)

### Supported platform

- PfSense
- OPNSense

## Development

WallGuard is an active work in progress. Some features may be incomplete or unavailable, and APIs may change between versions.

### Prerequisites

To build and develop WallGuard, you'll need the following:

- A Linux system (Debian/Ubuntu recommended)
- Rust (latest stable edition)
- Required development packages:
  - `libpcap-dev`
  - `protobuf-compiler`
  - `libprotobuf-dev`

> Note: Package names may vary slightly depending on your distribution.

### Datastore Dependency

WallGuard relies on a separate service called **datastore** for database operations. Make sure it's installed and running before starting the server.

You can find the datastore project here:  
🔗 [https://github.com/NullNet-ai/datastore](https://github.com/NullNet-ai/datastore)

## Contributing

If you'd like to help improve **WallGuard**, you're welcome to open issues or submit pull requests.

## License

This project is licensed under the **GNU Affero General Public License v3.0**.  
See the [LICENSE](LICENSE) file in this repository for the full license text.
