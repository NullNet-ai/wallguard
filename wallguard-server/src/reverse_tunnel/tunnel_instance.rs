use nullnet_liberror::{Error, ErrorHandler, Location, location};
use prost::bytes::Bytes;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc::Sender};
use tonic::{Status, Streaming};
use wallguard_common::protobuf::wallguard_tunnel::{ClientFrame, DataFrame, ServerFrame};
use wallguard_common::protobuf::wallguard_tunnel::server_frame::Message as ServerMessage;

type TunnelWriter = Sender<Result<ServerFrame, Status>>;
type TunnelReader = Streaming<ClientFrame>;

#[derive(Debug, Clone)]
pub struct TunnelInstance {
    pub writer: Arc<Mutex<TunnelWriter>>,
    pub reader: Arc<Mutex<TunnelReader>>,
}

impl TunnelInstance {
    pub fn new(reader: TunnelReader, writer: TunnelWriter) -> Self {
        Self {
            writer: Arc::new(Mutex::new(writer)),
            reader: Arc::new(Mutex::new(reader)),
        }
    }

    pub async fn read(&self) -> Result<ClientFrame, Error> {
        self.reader
            .lock()
            .await
            .message()
            .await
            .handle_err(location!())?
            .ok_or("TunnelInstance: Read error, client has sent an empty message")
            .handle_err(location!())
    }

    pub async fn write(&self, message: ServerFrame) -> Result<(), Error> {
        self.writer
            .lock()
            .await
            .send(Ok(message))
            .await
            .handle_err(location!())
    }

    pub fn make_data_frame(bytes: &Bytes) -> ServerFrame {
        ServerFrame { message: Some(ServerMessage::Data(DataFrame {
            data: bytes.to_vec()
        })) }
    }
}
