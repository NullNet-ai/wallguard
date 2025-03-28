use super::convert_message;
use super::proxy_message::ProxyMessage;
use actix::AsyncContext;
use actix::StreamHandler;
use actix_web_actors::ws::Message as ActixWsMessage;
use actix_web_actors::ws::ProtocolError as ActixWsProtocolError;
use futures::stream::{SplitSink, SplitStream};
use futures::StreamExt;
use futures_util::SinkExt;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;
use tokio_tungstenite::WebSocketStream;

pub(super) struct ProxyWebsocket {
    reader: Arc<Mutex<SplitStream<WebSocketStream<TcpStream>>>>,
    writer: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, TungsteniteMessage>>>,
}

impl From<WebSocketStream<TcpStream>> for ProxyWebsocket {
    fn from(stream: WebSocketStream<TcpStream>) -> Self {
        let (w, r) = stream.split();
        Self {
            writer: Arc::new(Mutex::new(w)),
            reader: Arc::new(Mutex::new(r)),
        }
    }
}

impl actix::Actor for ProxyWebsocket {
    type Context = actix_web_actors::ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let address = ctx.address();
        let reader = self.reader.clone();

        tokio::spawn(async move {
            while let Some(message) = reader.lock().await.next().await {
                match message {
                    Ok(message) => {
                        address.do_send(ProxyMessage::from(message));
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        });
    }
}

impl StreamHandler<Result<ActixWsMessage, ActixWsProtocolError>> for ProxyWebsocket {
    fn handle(
        &mut self,
        msg: Result<ActixWsMessage, ActixWsProtocolError>,
        _ctx: &mut Self::Context,
    ) {
        let Ok(message) = msg else {
            log::error!("Recevied an error instead of message: {}", msg.unwrap_err());
            return;
        };

        let Ok(tungstenite_message) = convert_message::actix_to_tungstenite(message) else {
            log::error!("Failed to convert `actix` message");
            return;
        };

        let writer = self.writer.clone();
        tokio::spawn(async move {
            let _ = writer.lock().await.send(tungstenite_message).await;
        });
    }
}

impl actix::Handler<ProxyMessage> for ProxyWebsocket {
    type Result = ();

    fn handle(&mut self, msg: ProxyMessage, ctx: &mut Self::Context) -> Self::Result {
        let Ok(message) = convert_message::tungstenite_to_actix(msg.message) else {
            log::error!("Failed to convert `tungstenite` message");
            return;
        };

        match message {
            ActixWsMessage::Text(text) => ctx.text(text),
            ActixWsMessage::Binary(data) => ctx.binary(data),
            ActixWsMessage::Ping(data) => ctx.ping(&data),
            ActixWsMessage::Pong(data) => ctx.pong(&data),
            ActixWsMessage::Close(close_reason) => ctx.close(close_reason),
            ActixWsMessage::Continuation(_) | ActixWsMessage::Nop => unreachable!(),
        }
    }
}
