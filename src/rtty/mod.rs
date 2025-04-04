use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use nullnet_libconfmon::Platform;
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use pty::Pty;
use session::Session;
use std::{net::SocketAddr, sync::Arc};
use tokio::{sync::oneshot, task::JoinHandle};

mod pty;
mod pty_message;
mod session;

struct Handle {
    inner: JoinHandle<Result<(), Error>>,
    sh_tx: oneshot::Sender<()>,
}

pub struct TTYServer {
    addr: SocketAddr,
    handle: Option<Handle>,
    platform: Platform,
}

impl Handle {
    pub fn new(addr: SocketAddr, platform: Platform) -> Result<Self, Error> {
        let pty = Pty::new(platform)?;
        let (sh_tx, sh_rx) = oneshot::channel();
        let inner = tokio::spawn(main_loop(addr, pty, sh_rx));
        Ok(Self { inner, sh_tx })
    }

    pub async fn shutdown(self) {
        if self.sh_tx.send(()).is_ok() {
            let _ = self.inner.await;
        } else {
            self.inner.abort();
        }
    }
}

impl TTYServer {
    pub fn new(addr: SocketAddr, platform: Platform) -> Self {
        Self {
            addr,
            platform,
            handle: None,
        }
    }

    pub async fn start(&mut self) -> Result<(), Error> {
        if self.handle.is_none() {
            let handle = Handle::new(self.addr, self.platform)?;
            self.handle = Some(handle);
        }

        Ok(())
    }

    pub async fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.shutdown().await;
        }
    }
}

async fn main_loop(
    addr: SocketAddr,
    pty: Pty,
    shutdown_rx: oneshot::Receiver<()>,
) -> Result<(), Error> {
    let data = web::Data::new(pty);

    let factory = data.clone();
    let server = HttpServer::new(move || {
        App::new()
            .app_data(factory.clone())
            .route("/ws/", web::get().to(websocket_handler))
    })
    .bind(addr)
    .handle_err(location!())?
    .run();

    let handle = server.handle();

    let server_task = tokio::spawn(server);
    let shutdown_task = tokio::spawn(async move {
        let _ = shutdown_rx.await;
        log::debug!("TTY Server received shutdown signal");
        handle.stop(true).await;
    });

    let _ = tokio::try_join!(server_task, shutdown_task).expect("Unable to join tasks");

    Arc::try_unwrap(data.into_inner()).expect("Someone still holds a reference to the PTY even after the TTY server has been shutdown.").shutdown().await;

    log::debug!("TTY Server terminated");

    Ok(())
}

async fn websocket_handler(
    request: HttpRequest,
    stream: web::Payload,
    data: web::Data<Pty>,
) -> Result<HttpResponse, actix_web::Error> {
    let session = Session::new(data.into_inner());
    let response = ws::start(session, &request, stream)?;
    Ok(response)
}
