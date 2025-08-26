use crate::reverse_tunnel::{TunnelInstance, TunnelReader, TunnelWriter};
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use wallguard_common::protobuf::wallguard_tunnel::client_frame::Message as ClientMessage;
use wallguard_common::protobuf::wallguard_tunnel::server_frame::Message as ServerMessage;
use wallguard_common::protobuf::wallguard_tunnel::{ClientFrame, DataFrame};

pub async fn copy_bidirectional_for_tunnel(tunnel: TunnelInstance, stream: TcpStream) {
    let (stream_reader, stream_writer) = tokio::io::split(stream);

    tokio::select! {
        _ = copy_from_tunnel_to_stream(tunnel.reader, stream_writer) => {}
        _ = copy_from_stream_to_tunnel(stream_reader, tunnel.writer) => {}
    }
}

async fn copy_from_tunnel_to_stream(
    reader: TunnelReader,
    mut writer: WriteHalf<TcpStream>,
) -> Result<(), Error> {
    loop {
        let message = reader
            .lock()
            .await
            .message()
            .await
            .handle_err(location!())?
            .ok_or("End of stream")
            .handle_err(location!())?
            .message
            .ok_or("Unexpected empty message")
            .handle_err(location!())?;

        let ServerMessage::Data(data_frame) = message else {
            return Err("Unexpected message type").handle_err(location!())?;
        };

        writer
            .write_all(&data_frame.data)
            .await
            .handle_err(location!())?;
    }
}

async fn copy_from_stream_to_tunnel(
    mut reader: ReadHalf<TcpStream>,
    writer: TunnelWriter,
) -> Result<(), Error> {
    let mut buf = [0u8; 8196];

    loop {
        let bytes = reader.read(&mut buf).await.handle_err(location!())?;

        if bytes == 0 {
            break;
        }

        let message = ClientFrame {
            message: Some(ClientMessage::Data(DataFrame {
                data: buf[..bytes].to_vec(),
            })),
        };

        writer
            .lock()
            .await
            .send(message)
            .await
            .handle_err(location!())?;
    }

    Ok(())
}
