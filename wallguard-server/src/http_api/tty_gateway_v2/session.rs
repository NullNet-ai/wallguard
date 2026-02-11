use std::sync::Arc;
use std::time::Duration;

use crate::app_context::AppContext;
use crate::datastore::TtySessionModel;
use crate::datastore::TtySessionStatus;
use crate::http_api::tty_gateway_v2::internal_relay::InternalRelay;
use crate::reverse_tunnel::TunnelInstance;
use nullnet_liberror::Error;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::sync::{Mutex, broadcast, mpsc};

pub(in crate::http_api::tty_gateway_v2) type ChannelReader = ReadHalf<TunnelInstance>;
pub(in crate::http_api::tty_gateway_v2) type ChannelWriter = WriteHalf<TunnelInstance>;
pub(in crate::http_api::tty_gateway_v2) type SessionDataSender = mpsc::Sender<Vec<u8>>;
pub(in crate::http_api::tty_gateway_v2) type UserDataReceiver = mpsc::Receiver<Vec<u8>>;
pub(in crate::http_api::tty_gateway_v2) type UserDataSender = broadcast::Sender<Vec<u8>>;
pub(in crate::http_api::tty_gateway_v2) type SessionDataReceiver = broadcast::Receiver<Vec<u8>>;

const DEFAULT_TIMEOUT: Duration = Duration::from_mins(15);
const MEMORY_SIZE: usize = 16392;
type SessionMemory = Arc<Mutex<Vec<u8>>>;

#[derive(Debug)]
pub struct Session {
    data_sender: SessionDataSender,
    data_receiver: SessionDataReceiver,
    memory: SessionMemory,
    signal: broadcast::Sender<()>,
}

impl Session {
    pub async fn new(
        context: AppContext,
        tunnel: TunnelInstance,
        data: &TtySessionModel,
    ) -> Result<Self, Error> {
        let (session_reader, session_writer) = tokio::io::split(tunnel);

        let (from_users_sender, from_users_receiver) = mpsc::channel(128);
        let (to_users_sender, to_users_receiver) = broadcast::channel(128);

        let (terminate, _) = broadcast::channel(2);

        InternalRelay::new(
            context.clone(),
            data.id.clone(),
            session_reader,
            session_writer,
            to_users_sender,
            from_users_receiver,
            terminate.subscribe(),
        )
        .spawn();

        let memory: SessionMemory = Default::default();

        tokio::spawn(session_timeout_impl(
            memory.clone(),
            to_users_receiver.resubscribe(),
            DEFAULT_TIMEOUT,
            context.clone(),
            data.id.clone(),
            terminate.subscribe(),
        ));

        Ok(Self {
            data_sender: from_users_sender,
            data_receiver: to_users_receiver,
            memory,
            signal: terminate,
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

    pub async fn terminate(&self) {
        let _ = self.signal.send(());
    }
}

async fn session_timeout_impl(
    memory: SessionMemory,
    receiver: SessionDataReceiver,
    duration: Duration,
    context: AppContext,
    session_id: String,
    mut terminate: broadcast::Receiver<()>,
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
                .update_tty_session_status(&token.jwt, &session_id, TtySessionStatus::Expired, false)
                .await;

            let _ = context.tty_sessions_manager.remove(&session_id).await;
        }
        _ = memory_monitor(memory, receiver) => {}
        _ = terminate.recv() => {}
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

    log::debug!("TTY session memory monitor terminated");
}
