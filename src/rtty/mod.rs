mod session;

use nullnet_libconfmon::Platform;
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use session::Session;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::broadcast;
use tokio::sync::mpsc::{self, error::TryRecvError};
use tokio::sync::Mutex;
use uuid::Uuid;
use warp::{filters::ws::WebSocket, Filter};

pub struct RemoteTTYServer {
    /// The address the server is bound to.
    pub addr: SocketAddr,
    /// The platform on which the server is running.
    pub platform: Platform,
    /// Passed to session so that they could notify about session completion
    notifier: mpsc::Sender<Uuid>,
    /// Receives IDS of completed sessions
    receiver: mpsc::Receiver<Uuid>,
    /// Shutdown signal sender.
    shutdown: Option<broadcast::Sender<()>>,
    /// Currently open sessions
    sessions_map: HashMap<Uuid, Session>,
}

impl RemoteTTYServer {
    pub fn new(addr: SocketAddr, platform: Platform) -> Self {
        let (notifier, receiver) = mpsc::channel(64);
        Self {
            addr,
            platform,
            notifier,
            receiver,
            shutdown: None,
            sessions_map: HashMap::new(),
        }
    }

    /// Starts the server and spawns background tasks for handling WebSocket connections and session monitoring.
    pub async fn run(self) -> Arc<Mutex<Self>> {
        let server = Arc::new(Mutex::new(self));
        let (shutdown_signal, _) = broadcast::channel(64);

        let _ = tokio::spawn(Self::run_websocket_server(
            server.clone(),
            shutdown_signal.subscribe(),
        ));
        let _ = tokio::spawn(Self::run_sessions_listener(
            server.clone(),
            shutdown_signal.subscribe(),
        ));

        server.lock().await.shutdown = Some(shutdown_signal);

        server
    }

    /// Gracefully shuts down all active sessions.
    pub async fn shutdown(&mut self) {
        for (id, session) in self.sessions_map.drain() {
            log::debug!("Shutting down session {}", id);
            let _ = session.shutdown().await;
        }
    }

    /// Opens a new WebSocket-based TTY session.
    fn open_new_session(&mut self, websocket: WebSocket) {
        log::debug!("New WS RTTY session");
        let platform = self.platform.clone();
        let notifier = self.notifier.clone();
        let session = Session::spawn(websocket, platform, notifier);
        self.sessions_map.insert(session.id.clone(), session);
    }

    /// Runs the WebSocket server and listens for incoming connections.
    async fn run_websocket_server(this: Arc<Mutex<Self>>, mut shutdown: broadcast::Receiver<()>) {
        let server = this.clone();
        let route = warp::path("ws")
            .and(warp::ws())
            .map(move |ws: warp::ws::Ws| {
                let server = server.clone();
                ws.on_upgrade(|websocket| async move {
                    server.clone().lock().await.open_new_session(websocket);
                })
            });

        let addr = this.lock().await.addr;
        tokio::select! {
            _ = warp::serve(route).run(addr) => log::debug!("WebSocket server terminated"),
            _ = shutdown.recv() => log::debug!("Server shutdown signal received")
        }
    }

    /// Listens for session completion notifications and removes completed sessions.
    async fn run_sessions_listener(this: Arc<Mutex<Self>>, mut shutdown: broadcast::Receiver<()>) {
        tokio::select! {
            _ = tokio::spawn(async move {
                let server = this.clone();
                loop {
                    let recv_result = server.lock().await.receiver.try_recv();
                    match recv_result {
                        Ok(session_id) => {
                            log::info!("Session {} has completed", &session_id);
                            server.lock().await.sessions_map.remove(&session_id);
                        }
                        Err(TryRecvError::Empty) => tokio::task::yield_now().await,
                        Err(TryRecvError::Disconnected) => break,
                    }
                }
            }) => log::debug!("Session listener terminated"),
            _ = shutdown.recv() => log::debug!("Server shutdown signal received")
        }
    }
}
