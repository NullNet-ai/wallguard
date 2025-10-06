use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::{Mutex, mpsc};
use tonic::Streaming;
use wallguard_common::protobuf::wallguard_tunnel::client_frame::Message as ClientMessage;
use wallguard_common::protobuf::wallguard_tunnel::server_frame::Message as ServerMessage;
use wallguard_common::protobuf::wallguard_tunnel::{AuthFrame, ClientFrame, ServerFrame};
use wallguard_common::wallguard_tunnel_interface::WallGuardTunnelGrpcInterface;

use crate::utilities;

#[derive(Debug, Clone, Copy)]
pub struct ReverseTunnel {
    addr: SocketAddr,
}

pub(crate) type TunnelReader = Arc<Mutex<Streaming<ServerFrame>>>;
pub(crate) type TunnelWriter = Arc<Mutex<Sender<ClientFrame>>>;

pub struct TunnelInstance {
    pub(crate) reader: TunnelReader,
    pub(crate) writer: TunnelWriter,
}

impl ReverseTunnel {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }

    pub async fn request_channel(&self, token: &str) -> Result<TunnelInstance, Error> {
        let interface = WallGuardTunnelGrpcInterface::from_sockaddr(self.addr).await?;

        let (sender, receiver) = mpsc::channel(1024);

        let mut stream = interface.request_control_channel(receiver).await?;

        sender
            .send(ClientFrame {
                message: Some(ClientMessage::Authentication(AuthFrame {
                    token: utilities::hash::sha256_digest_bytes(token).into(),
                })),
            })
            .await
            .handle_err(location!())?;

        let message = stream
            .message()
            .await
            .handle_err(location!())?
            .ok_or("Unexpected end of stream")
            .handle_err(location!())?
            .message
            .ok_or("Received an empty message instead of a verdict")
            .handle_err(location!())?;

        let ServerMessage::Verdict(verdict) = message else {
            return Err("Unexpected message: expected a verdict").handle_err(location!());
        };

        if !verdict.allowed {
            return Err("The server rejected the tunnel request").handle_err(location!());
        }

        Ok(TunnelInstance {
            reader: Arc::new(Mutex::new(stream)),
            writer: Arc::new(Mutex::new(sender)),
        })
    }
}
