use actix::{Actor, AsyncContext, Handler, StreamHandler};
use actix_web_actors::ws::{self};

use std::sync::Arc;

use super::pty::Pty;
use super::pty_message::PtyMessage;

pub struct Session {
    pty: Arc<Pty>,
}

impl Session {
    pub fn new(pty: Arc<Pty>) -> Self {
        Self { pty }
    }
}

impl Actor for Session {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let address = ctx.address();
        let pty = self.pty.clone();

        tokio::spawn(async move {
            let current = pty.current_buffer().await;
            address.do_send(PtyMessage::from(current));

            let mut receiver = pty.subscribe().await;

            loop {
                match receiver.recv().await {
                    Ok(value) => address.do_send(PtyMessage::from(value)),
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        log::error!("Pty: Channel already closed");
                        return;
                    }
                }
            }
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Session {
    fn handle(&mut self, message: Result<ws::Message, ws::ProtocolError>, _: &mut Self::Context) {
        if let Ok(ws::Message::Text(text)) = message {
            let message = Vec::from(text.as_bytes().iter().as_slice());
            let pty = self.pty.clone();

            tokio::spawn(async move {
                let _ = pty.send(message).await;
            });
        }
    }
}

impl Handler<PtyMessage> for Session {
    type Result = ();

    fn handle(&mut self, message: PtyMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(message);
    }
}
