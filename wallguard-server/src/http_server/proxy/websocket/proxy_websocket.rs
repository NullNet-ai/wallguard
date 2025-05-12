use super::convert_message;
use super::proxy_message::ProxyMessage;
use super::stop_message::StopMessage;
use actix::AsyncContext;
use actix::StreamHandler;
use actix_web_actors::ws::Message as ActixWsMessage;
use actix_web_actors::ws::ProtocolError as ActixWsProtocolError;
use futures::StreamExt;
use futures::TryStreamExt;
use futures::stream::{SplitSink, SplitStream};
use futures_util::SinkExt;
use hyper::body::Bytes;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;

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
            loop {
                let result = reader.lock().await.try_next().await;

                if result.is_err() {
                    break;
                }

                let Some(message) = result.unwrap() else {
                    break;
                };

                address.do_send(ProxyMessage::from(message));
            }

            address.do_send(StopMessage {});
        });

        //
        // This block is a workaround for detecting unexpected resets of the underlying TCP stream.
        // The Tungstenite library does not reliably surface such resets, so we periodically send
        // WebSocket ping messages to force activity and reveal broken connections.
        // This is a temporary, duct-tape solution and should be revisited for a more robust approach.
        //
        // Since this proxy is solely intended for tunneling into TTY sessions,
        // we may want to replace the WebSocket server on the WallGuard client side
        // with a plain TCP stream to simplify the design and avoid unnecessary abstraction.
        //
        let address = ctx.address();
        let writer = self.writer.clone();
        tokio::spawn(async move {
            loop {
                let ping = TungsteniteMessage::Ping(Bytes::from(&b"ping"[..]));
                if writer.lock().await.send(ping).await.is_err() {
                    break;
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }

            address.do_send(StopMessage {});
        });
    }
}

impl StreamHandler<Result<ActixWsMessage, ActixWsProtocolError>> for ProxyWebsocket {
    fn handle(
        &mut self,
        msg: Result<ActixWsMessage, ActixWsProtocolError>,
        ctx: &mut Self::Context,
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
        let address = ctx.address();
        tokio::spawn(async move {
            if writer.lock().await.send(tungstenite_message).await.is_err() {
                address.do_send(StopMessage {});
            };
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

impl actix::Handler<StopMessage> for ProxyWebsocket {
    type Result = ();

    fn handle(&mut self, _msg: StopMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.close(None);
    }
}
