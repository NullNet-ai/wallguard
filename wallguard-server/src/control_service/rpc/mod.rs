// tonic requires handlers to return `Result<_, tonic::Status>`, and Status
// is large; boxing it is not an option, so the lint does not apply here.
#![allow(clippy::result_large_err)]

mod control_channel;
mod get_device_settings;
mod handle_config_data;
mod handle_connections_data;
mod handle_system_resources_data;
mod report_services;
// mod request_tunnel;
