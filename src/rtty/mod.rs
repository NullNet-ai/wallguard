use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use nullnet_libconfmon::Platform;
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use session::Session;
use std::net::SocketAddr;
use tokio::task::JoinHandle;

mod pty;
mod pty_message;
mod session;

pub struct TTYServer {
    addr: SocketAddr,
    platform: Platform,
    handle: Option<JoinHandle<Result<(), std::io::Error>>>,
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
            let platform = self.platform.clone();
            let server = HttpServer::new(move || {
                App::new()
                    .app_data(web::Data::new(platform))
                    .route("/ws/", web::get().to(websocket_handler))
            })
            .bind(self.addr)
            .handle_err(location!())?;

            let server_task: tokio::task::JoinHandle<Result<(), std::io::Error>> =
                tokio::spawn(server.run());
            self.handle = Some(server_task);
        }

        Ok(())
    }

    pub async fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

async fn websocket_handler(
    request: HttpRequest,
    stream: web::Payload,
    platform: web::Data<Platform>,
) -> Result<HttpResponse, actix_web::Error> {
    let session = Session::new(platform.get_ref().clone())
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_str().to_string()))?;
    let response = ws::start(session, &request, stream)?;
    Ok(response)
}
