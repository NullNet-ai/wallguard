use nullnet_liberror::{location, Error, ErrorHandler, Location};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use uuid::Uuid;

pub fn generate_uuid_str() -> String {
    Uuid::new_v4().to_string()
}

pub async fn generate_addr() -> Result<SocketAddr, Error> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .handle_err(location!())?;
    let addr = listener.local_addr().handle_err(location!())?;
    Ok(addr)
}
