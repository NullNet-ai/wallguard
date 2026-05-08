use std::sync::Arc;

use super::session::{ChannelReader, ChannelWriter, UserDataReceiver, UserDataSender};
use crate::app_context::AppContext;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::broadcast;

pub(crate) struct InternalRelay {
    context: Arc<AppContext>,
    tunnel_id: String,

    channel_reader: ChannelReader,
    channel_writer: ChannelWriter,

    data_sender: UserDataSender,
    data_receiver: UserDataReceiver,

    terminate: broadcast::Receiver<()>,
}

impl InternalRelay {
    pub fn new(
        context: Arc<AppContext>,
        tunnel_id: String,
        channel_reader: ChannelReader,
        channel_writer: ChannelWriter,
        data_sender: UserDataSender,
        data_receiver: UserDataReceiver,
        terminate: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            context,
            tunnel_id,
            channel_reader,
            channel_writer,
            data_sender,
            data_receiver,
            terminate,
        }
    }

    pub fn spawn(self) {
        tokio::spawn(internal_relay_impl(
            self.context,
            self.tunnel_id,
            self.channel_reader,
            self.channel_writer,
            self.data_sender,
            self.data_receiver,
            self.terminate,
        ));
    }
}

async fn internal_relay_impl(
    context: Arc<AppContext>,
    tunnel_id: String,
    channel_reader: ChannelReader,
    channel_writer: ChannelWriter,
    data_sender: UserDataSender,
    data_receiver: UserDataReceiver,
    mut terminate: broadcast::Receiver<()>,
) {
    tokio::select! {
        _ = from_users_to_channel(data_receiver, channel_writer) => {
            log::debug!("RD Internal Relay: input relay finished");
        }
        _ = from_channel_to_users(channel_reader, data_sender) => {
            log::debug!("RD Internal Relay: frame relay finished");
        }
        _ = terminate.recv() => {
            log::debug!("RD Internal Relay: TERM signal received");
        }
    }

    let _ = context
        .tunnels_manager
        .on_tunnel_terminated(&tunnel_id)
        .await;
}

/// Reads input events from viewers and writes them to the agent tunnel.
///
/// Each message is framed as `[u32 LE length][payload bytes]` so the agent
/// can reconstruct complete JSON events regardless of how TCP segments the
/// stream.
async fn from_users_to_channel(
    mut data_receiver: UserDataReceiver,
    mut channel_writer: ChannelWriter,
) {
    while let Some(message) = data_receiver.recv().await {
        let len = (message.len() as u32).to_le_bytes();
        if channel_writer.write_all(&len).await.is_err() {
            break;
        }
        if channel_writer.write_all(&message).await.is_err() {
            break;
        }
    }
}

/// Reads H264 frames from the agent tunnel and broadcasts each complete frame
/// to all connected viewers.
///
/// The agent wraps every frame in a `TimestampedPacket`:
///   bytes  0–15  : duration as u128 LE (16 bytes)
///   bytes 16–19  : payload length as u32 LE (4 bytes)
///   bytes 20..   : H264 payload
///
/// We reassemble the full packet before broadcasting so each item in the
/// channel — and therefore each binary WebSocket message sent to viewers —
/// is exactly one decodable frame.
async fn from_channel_to_users(mut channel_reader: ChannelReader, data_sender: UserDataSender) {
    const HEADER_LEN: usize = 20; // 16-byte duration + 4-byte length field

    loop {
        let mut header = [0u8; HEADER_LEN];
        if channel_reader.read_exact(&mut header).await.is_err() {
            break;
        }

        let data_len = u32::from_le_bytes(header[16..20].try_into().unwrap()) as usize;

        let mut payload = vec![0u8; data_len];
        if channel_reader.read_exact(&mut payload).await.is_err() {
            break;
        }

        let mut packet = Vec::with_capacity(HEADER_LEN + data_len);
        packet.extend_from_slice(&header);
        packet.extend_from_slice(&payload);

        if data_sender.send(packet).is_err() {
            break;
        }
    }
}
