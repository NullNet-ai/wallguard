use actix::{Actor, AsyncContext, Handler, StreamHandler};
use actix_web_actors::ws::{self, CloseCode, CloseReason};
use nullnet_libconfmon::Platform;
use nullnet_liberror::Error;
use std::io::{Read, Write};
use std::sync::Arc;

use super::pty::Pty;
use super::pty_message::PtyMessage;

pub struct Session {
    pty: Arc<Pty>,
}

impl Session {
    pub fn new(platform: Platform) -> Result<Self, Error> {
        let pty = Pty::new(&platform)?;
        Ok(Self { pty: Arc::new(pty) })
    }
}

impl Actor for Session {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let address = ctx.address();
        let reader = self.pty.reader.clone();

        tokio::task::spawn_blocking(move || {
            let mut buffer = [0; 1024];
            loop {
                let n = reader
                    .lock()
                    .map_or(0, |mut lock| lock.read(&mut buffer).unwrap_or(0));

                if n == 0 {
                    break;
                }

                let pty_message = PtyMessage::from_slice(&buffer[..n]);
                address.do_send(pty_message);
            }
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Session {
    fn handle(&mut self, message: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match message {
            Ok(ws::Message::Text(text)) => {
                if let Ok(mut lock) = self.pty.writer.lock() {
                    let _ = lock.write(text.as_bytes());
                    p
                } else {
                    let reason = CloseReason {
                        code: CloseCode::Error,
                        description: Some("Failed to write message to pty".to_string()),
                    };
                    ctx.close(Some(reason));
                }
            }
            _ => (),
        }
    }
}

impl Handler<PtyMessage> for Session {
    type Result = ();

    fn handle(&mut self, message: PtyMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(message);
    }
}
