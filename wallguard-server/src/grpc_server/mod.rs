use crate::cli::Args;
use crate::parser::ip_info::ip_info_handler;
use crate::{app_context::AppContext, proto::wallguard::wall_guard_server::WallGuardServer};
use crate::{datastore::DatastoreWrapper, proto::wallguard::wall_guard_server::WallGuardServer};
use server::WallGuardImpl;
use std::net::ToSocketAddrs;
use std::sync::mpsc;
use std::thread;
use tonic::transport::Server;

mod endpoints;
mod request_log;
mod server;

pub(crate) const ADDR: &str = "0.0.0.0";
pub(crate) const PORT: u16 = 50051;

pub async fn run_grpc_server(context: AppContext, args: Args) {
    let addr = format!("{ADDR}:{PORT}")
        .to_socket_addrs()
        .expect("Failed to resolve address")
        .next()
        .expect("Failed to get address");

    // let cert =
    //     std::fs::read_to_string("./tls/wallmon.pem").expect("Failed to read certificate file");
    // let key = std::fs::read_to_string("./tls/wallmon-key.pem").expect("Failed to read key file");
    // let identity = Identity::from_pem(cert, key);

    let datastore_2 = context.datastore.clone();

    let (ip_info_tx, ip_info_rx) = mpsc::channel();
    let rt_handle = tokio::runtime::Handle::current();
    thread::spawn(move || {
        ip_info_handler(
            &ip_info_rx,
            args.ip_info_cache_size,
            &rt_handle,
            &datastore_2,
        );
    });

    Server::builder()
        // .tls_config(ServerTlsConfig::new().identity(identity))
        // .expect("Failed to set up TLS")
        .add_service(
            WallGuardServer::new(WallGuardImpl {
                context,
                ip_info_tx,
            })
            .max_decoding_message_size(50 * 1024 * 1024),
        )
        .serve(addr)
        .await
        .expect("Failed to start gRPC server");
}
