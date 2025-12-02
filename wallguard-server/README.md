# WallGuard Server
[![Server Docker](https://github.com/NullNet-ai/wallguard/actions/workflows/docker.yml/badge.svg)](https://github.com/NullNet-ai/wallguard/actions/workflows/docker.yml)

[![Build CI](https://github.com/NullNet-ai/wallguard/actions/workflows/build.yml/badge.svg)](https://github.com/NullNet-ai/wallguard/actions/workflows/build.yml)

## ⚙️ Configuration — Environment Variables  

| Variable | Description |
|---------|-------------|
| `DATASTORE_HOST` | Hostname or IP of the datastore |
| `DATASTORE_PORT` | Port of the datastore |
| `DATASTORE_TLS` | Whether to use TLS / secure connection to datastore |
| `ROOT_ACCOUNT_ID` | Datastore root account ID |
| `ROOT_ACCOUNT_SECRET` | Datastore root account secret |
| `SYSTEM_ACCOUNT_ID` | Datastore system account ID |
| `SYSTEM_ACCOUNT_SECRET` | Datastore system account secret |
| `CONTROL_SERVICE_ADDR` | Address to bind control service to |
| `CONTROL_SERVICE_PORT` | Port to bind control service to |
| `HTTP_PROXY_HOST` | HTTP proxy host |
| `HTTP_PROXY_PORT` | HTTP proxy port |
| `MCP_SERVER_HOST` | MCP server hostname or IP address |
| `MCP_SERVER_PORT` | MCP server port |
| `IP_INFO_API_KEY` | API key for IP info service |
