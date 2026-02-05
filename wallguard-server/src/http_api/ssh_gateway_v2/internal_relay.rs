use crate::app_context::AppContext;
use crate::datastore::SshSessionStatus;
use crate::http_api::ssh_gateway_v2::session::{
    ChannelReader, ChannelWriter, UserDataReceiver, UserDataSender,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::broadcast;

pub(crate) struct InternalRelay {
    context: AppContext,
    session_id: String,

    // SSH Channel read & write
    channel_reader: ChannelReader,
    channel_writer: ChannelWriter,

    // Intermediate channels read & write
    data_sender: UserDataSender,
    data_receiver: UserDataReceiver,

    terminate: broadcast::Receiver<()>,
}

impl InternalRelay {
    pub fn new(
        context: AppContext,
        session_id: String,
        channel_reader: ChannelReader,
        channel_writer: ChannelWriter,
        data_sender: UserDataSender,
        data_receiver: UserDataReceiver,
        terminate: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            context,
            session_id,
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
            self.session_id,
            self.channel_reader,
            self.channel_writer,
            self.data_sender,
            self.data_receiver,
            self.terminate,
        ));
    }
}

async fn internal_relay_impl(
    context: AppContext,
    session_id: String,
    channel_reader: ChannelReader,
    channel_writer: ChannelWriter,
    data_sender: UserDataSender,
    data_receiver: UserDataReceiver,
    mut terminate: broadcast::Receiver<()>,
) {
    tokio::select! {
        _ = from_users_to_channel(data_receiver, channel_writer) => {
            log::debug!("SSH Internal Relay: Channel to SSH relay finished");
        }
        _ = from_channel_to_users(channel_reader, data_sender) => {
            log::debug!("SSH Internal Relay: SSH to Channel relay finished");
        }
        _ = terminate.recv() => {
            log::debug!("SSH Internal Relay: TERM singal received");
        }
    }

    let _ = context.ssh_sessions_manager.remove(&session_id).await;

    let Ok(token) = context.sysdev_token_provider.get().await else {
        log::error!("SSH Internal Relay: failed to acquire token");
        return;
    };

    if context
        .datastore
        .update_ssh_session_status(&token.jwt, &session_id, SshSessionStatus::Terminated, false)
        .await
        .is_err()
    {
        log::error!("SSH Internal Relay: failed to update session status");
    }
}

async fn from_users_to_channel(
    mut data_receiver: UserDataReceiver,
    mut channel_writer: ChannelWriter,
) {
    while let Some(message) = data_receiver.recv().await {
        if channel_writer.write(message.as_slice()).await.is_err() {
            break;
        }
    }
}

async fn from_channel_to_users(mut channel_reader: ChannelReader, data_sender: UserDataSender) {
    let mut buffer = [0u8; 8 * 1024];

    loop {
        match channel_reader.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => {
                let message = buffer[..n].to_vec();
                if data_sender.send(message).is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}
