use std::sync::Arc;

use super::internal_relay::InternalRelay;
use crate::app_context::AppContext;
use crate::reverse_tunnel::TunnelInstance;
use nullnet_liberror::Error;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::sync::{broadcast, mpsc};

pub type ChannelReader = ReadHalf<TunnelInstance>;
pub type ChannelWriter = WriteHalf<TunnelInstance>;
pub type SessionDataSender = mpsc::Sender<Vec<u8>>;
pub type UserDataReceiver = mpsc::Receiver<Vec<u8>>;
pub type UserDataSender = broadcast::Sender<Vec<u8>>;
pub type SessionDataReceiver = broadcast::Receiver<Vec<u8>>;

#[derive(Debug)]
pub struct Session {
    data_sender: SessionDataSender,

    /// Cloneable broadcast sender — call `.subscribe()` to create a fresh
    /// receiver for each new viewer.  Viewers always start at the current
    /// stream head so they never see stale frames from before they connected.
    data_source: UserDataSender,

    /// Sentinel receiver that keeps the broadcast channel alive even when no
    /// viewers are connected.  Without it the InternalRelay's `send()` would
    /// return an error (no receivers) and stop relaying before any viewer
    /// has had a chance to connect.
    _sentinel: SessionDataReceiver,

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
        let (to_users_sender, sentinel) = broadcast::channel(128);

        let (terminate, _) = broadcast::channel(2);

        InternalRelay::new(
            context,
            tunnel_id,
            session_reader,
            session_writer,
            to_users_sender.clone(), // InternalRelay gets a clone; Session keeps the original
            from_users_receiver,
            terminate.subscribe(),
        )
        .spawn();

        Ok(Self {
            data_sender: from_users_sender,
            data_source: to_users_sender,
            _sentinel: sentinel,
            signal: terminate,
        })
    }

    pub fn get_data_send_channel(&self) -> SessionDataSender {
        self.data_sender.clone()
    }

    /// Creates a fresh broadcast receiver for a new viewer.
    ///
    /// `subscribe()` starts the receiver at the current stream head so the
    /// viewer only receives frames produced *after* it connects, avoiding
    /// both stale-frame replays and the `Lagged` error that would occur if
    /// `resubscribe()` were used on the long-idle sentinel receiver.
    pub fn get_data_recv_channel(&self) -> SessionDataReceiver {
        self.data_source.subscribe()
    }

    pub async fn signal(&self) {
        let _ = self.signal.send(());
    }

    pub fn has_active_viewers(&self) -> bool {
        self.data_sender.strong_count() > 1
    }
}
