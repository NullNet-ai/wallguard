use std::sync::Arc;
use std::time::Duration;

use crate::app_context::AppContext;
use crate::datastore::SshSessionStatus;
use crate::http_api::ssh_gateway_v2::handler;
use crate::http_api::ssh_gateway_v2::internal_relay::InternalRelay;
use crate::reverse_tunnel::TunnelInstance;
use crate::{datastore::SshSessionModel};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use russh::ChannelStream;
use russh::client::{self, AuthResult, Msg};
use russh::keys::{PrivateKey, PrivateKeyWithHashAlg, decode_secret_key};
use tokio::io::{ReadHalf, WriteHalf};
use tokio::sync::{Mutex, broadcast, mpsc};

pub(in crate::http_api::ssh_gateway_v2) type ChannelReader = ReadHalf<ChannelStream<Msg>>;
pub(in crate::http_api::ssh_gateway_v2) type ChannelWriter = WriteHalf<ChannelStream<Msg>>;

pub(in crate::http_api::ssh_gateway_v2) type SessionDataSender = mpsc::Sender<Vec<u8>>;
pub(in crate::http_api::ssh_gateway_v2) type UserDataReceiver = mpsc::Receiver<Vec<u8>>;
pub(in crate::http_api::ssh_gateway_v2) type UserDataSender = broadcast::Sender<Vec<u8>>;
pub(in crate::http_api::ssh_gateway_v2) type SessionDataReceiver = broadcast::Receiver<Vec<u8>>;

const DEFAULT_TIMEOUT: Duration = Duration::from_mins(15);
const MEMORY_SIZE: usize = 16392;
type SessionMemory = Arc<Mutex<Vec<u8>>>;

#[derive(Debug)]
pub struct Session {
    data_sender: SessionDataSender,
    data_receiver: SessionDataReceiver,
    memory: SessionMemory,
}

impl Session {
    pub async fn new(
        context: AppContext,
        tunnel: TunnelInstance,
        data: &SshSessionModel,
    ) -> Result<Self, Error> {
        let private_key =
            decode_secret_key(&data.private_key, Some(&data.passphrase)).handle_err(location!())?;

        let (session_reader, session_writer) =
            Session::establish_ssh_session(tunnel, data.username.clone(), private_key).await?;

        let (from_users_sender, from_users_receiver) = mpsc::channel(128);
        let (to_users_sender, to_users_receiver) = broadcast::channel(128);

        InternalRelay::new(
            context.clone(),
            data.id.clone(),
            session_reader,
            session_writer,
            to_users_sender,
            from_users_receiver,
        )
        .spawn();

        let memory: SessionMemory = Default::default();

        tokio::spawn(session_timeout_impl(
            memory.clone(),
            to_users_receiver.resubscribe(),
            DEFAULT_TIMEOUT,
            context.clone(),
            data.id.clone(),
        ));

        Ok(Self {
            data_sender: from_users_sender,
            data_receiver: to_users_receiver,
            memory,
        })
    }

    pub fn get_data_send_channel(&self) -> SessionDataSender {
        self.data_sender.clone()
    }

    pub fn get_data_recv_channel(&self) -> SessionDataReceiver {
        self.data_receiver.resubscribe()
    }

    pub async fn get_memory_snaphot(&self) -> Vec<u8> {
        self.memory.lock().await.clone()
    }

    async fn establish_ssh_session(
        tunnel: TunnelInstance,
        username: String,
        private_key: PrivateKey,
    ) -> Result<(ChannelReader, ChannelWriter), Error> {
        let config = client::Config::default();

        let mut session = client::connect_stream(Arc::new(config), tunnel, handler::Handler)
            .await
            .handle_err(location!())?;

        let auth_response = session
            .authenticate_publickey(
                username,
                PrivateKeyWithHashAlg::new(Arc::new(private_key), None),
            )
            .await
            .handle_err(location!())?;

        if !matches!(auth_response, AuthResult::Success) {
            return Err("SSH authentication failed - check the keypair").handle_err(location!());
        }

        let channel = session
            .channel_open_session()
            .await
            .handle_err(location!())?;

        channel
            .request_pty(false, "xterm", 80, 24, 0, 0, &[])
            .await
            .handle_err(location!())?;

        channel.request_shell(false).await.handle_err(location!())?;

        let (reader, writer) = tokio::io::split(channel.into_stream());

        Ok((reader, writer))
    }
}

async fn session_timeout_impl(
    memory: SessionMemory,
    receiver: SessionDataReceiver,
    duration: Duration,
    context: AppContext,
    session_id: String,
) {
    // Session timeout is handled here to ensure proper cleanup:
    // When the timeout is reached, the receiver is dropped, which triggers
    // the internal relay to terminate. However, if a WebSocket connection
    // is still active, the session remains alive until that connection closes.
    // New connections are not allowed once the timeout has been hit.
    tokio::select! {
        _ = tokio::time::sleep(duration) => {
            let Ok(token) = context.sysdev_token_provider.get().await else {
                return;
            };

            let _ = context
                .datastore
                .update_ssh_session_status(&token.jwt, &session_id, SshSessionStatus::Expired, false)
                .await;

            let _ = context.ssh_sessions_manager.remove(&session_id).await;
        }
        _ = memory_monitor(memory, receiver) => {}
    }
}

async fn memory_monitor(memory: SessionMemory, mut receiver: SessionDataReceiver) {
    while let Ok(data) = receiver.recv().await {
        let mut mem = memory.lock().await;
        mem.extend_from_slice(&data);

        if mem.len() > MEMORY_SIZE {
            let excess = mem.len() - MEMORY_SIZE;
            mem.drain(0..excess);
        }
    }

    log::debug!("SSH session memory monitor terminated");
}
