use std::sync::Arc;

use super::internal_relay::InternalRelay;
use crate::app_context::AppContext;
use crate::reverse_tunnel::TunnelInstance;
use nullnet_liberror::Error;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::sync::{Mutex, broadcast, mpsc};

pub type ChannelReader = ReadHalf<TunnelInstance>;
pub type ChannelWriter = WriteHalf<TunnelInstance>;
pub type SessionDataSender = mpsc::Sender<Vec<u8>>;
pub type UserDataReceiver = mpsc::Receiver<Vec<u8>>;
pub type UserDataSender = broadcast::Sender<Vec<u8>>;
pub type SessionDataReceiver = broadcast::Receiver<Vec<u8>>;

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
        context: Arc<AppContext>,
        tunnel: TunnelInstance,
        tunnel_id: String,
    ) -> Result<Self, Error> {
        let (session_reader, session_writer) = tokio::io::split(tunnel);

        let (from_users_sender, from_users_receiver) = mpsc::channel(128);
        let (to_users_sender, to_users_receiver) = broadcast::channel(128);

        let (terminate, _) = broadcast::channel(2);

        InternalRelay::new(
            context.clone(),
            tunnel_id,
            session_reader,
            session_writer,
            to_users_sender,
            from_users_receiver,
            terminate.subscribe(),
        )
        .spawn();

        let memory: SessionMemory = Default::default();

        tokio::spawn(memory_monitor_impl(
            memory.clone(),
            to_users_receiver.resubscribe(),
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

    pub async fn signal(&self) {
        let _ = self.signal.send(());
    }

    pub fn has_active_terminals(&self) -> bool {
        self.data_sender.strong_count() > 1
    }
}

async fn memory_monitor_impl(
    memory: SessionMemory,
    receiver: SessionDataReceiver,
    mut terminate: broadcast::Receiver<()>,
) {
    tokio::select! {
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
