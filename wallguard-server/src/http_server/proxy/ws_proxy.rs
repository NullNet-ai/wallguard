use actix::AsyncContext;
use futures::stream::{SplitSink, SplitStream};
use futures::StreamExt;
use futures_util::SinkExt;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

fn map_reason_to_close_frame(
    reason: Option<actix_web_actors::ws::CloseReason>,
) -> Option<tokio_tungstenite::tungstenite::protocol::CloseFrame> {
    reason.map(|r| tokio_tungstenite::tungstenite::protocol::CloseFrame {
        code: match r.code {
            actix_web_actors::ws::CloseCode::Normal => 1000.into(),
            actix_web_actors::ws::CloseCode::Away => 1001.into(),
            actix_web_actors::ws::CloseCode::Protocol => 1002.into(),
            actix_web_actors::ws::CloseCode::Unsupported => 1003.into(),
            actix_web_actors::ws::CloseCode::Abnormal => 1006.into(),
            actix_web_actors::ws::CloseCode::Invalid => 1007.into(),
            actix_web_actors::ws::CloseCode::Policy => 1008.into(),
            actix_web_actors::ws::CloseCode::Size => 1009.into(),
            actix_web_actors::ws::CloseCode::Extension => 1010.into(),
            actix_web_actors::ws::CloseCode::Restart => 1012.into(),
            actix_web_actors::ws::CloseCode::Again => 1013.into(),
            actix_web_actors::ws::CloseCode::Error | _ => 1011.into(),
        },
        reason: r.description.map(|desc| desc.into()).unwrap_or_default(),
    })
}

pub async fn proxy_websocket_request(
    request: actix_web::HttpRequest,
    body: actix_web::web::Payload,
    target: SocketAddr,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let tcp_stream = tokio::net::TcpStream::connect(target)
        .await
        .map_err(actix_web::error::ErrorServiceUnavailable)?;

    let (ws_stream, _) =
        tokio_tungstenite::client_async(format!("ws://{}{}", target, request.uri()), tcp_stream)
            .await
            .map_err(actix_web::error::ErrorServiceUnavailable)?;

    let (ws_writer, ws_reader) = ws_stream.split();

    actix_web_actors::ws::start(
        ProxyWS {
            // @TODO: ReffCell
            reader: Arc::new(Mutex::new(ws_reader)),
            writer: Arc::new(Mutex::new(ws_writer)),
        },
        &request,
        body,
    )
}

struct ProxyMessage {
    message: tokio_tungstenite::tungstenite::Message,
}

impl actix::Message for ProxyMessage {
    type Result = ();
}

impl From<tokio_tungstenite::tungstenite::protocol::Message> for ProxyMessage {
    fn from(value: tokio_tungstenite::tungstenite::protocol::Message) -> Self {
        ProxyMessage { message: value }
    }
}

struct ProxyWS {
    reader: Arc<Mutex<SplitStream<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>>>>,
    writer: Arc<
        Mutex<
            SplitSink<
                tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
                tokio_tungstenite::tungstenite::Message,
            >,
        >,
    >,
}

impl actix::Actor for ProxyWS {
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

impl
    actix::StreamHandler<Result<actix_web_actors::ws::Message, actix_web_actors::ws::ProtocolError>>
    for ProxyWS
{
    fn handle(
        &mut self,
        msg: Result<actix_web_actors::ws::Message, actix_web_actors::ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        let writer = self.writer.clone();

        tokio::spawn(async move {
            if let Ok(msg) = msg {
                println!("Received message from Browser");
                match msg {
                    actix_web_actors::ws::Message::Text(text) => {
                        // Unnessesary clone here ?
                        if let Ok(message) = tokio_tungstenite::tungstenite::Utf8Bytes::try_from(
                            text.as_bytes().clone(),
                        ) {
                            let _ = writer
                                .lock()
                                .await
                                .send(tokio_tungstenite::tungstenite::protocol::Message::Text(
                                    message,
                                ))
                                .await;
                        } else {
                            unreachable!();
                        }
                    }
                    actix_web_actors::ws::Message::Binary(bin) => {
                        let _ = writer
                            .lock()
                            .await
                            .send(tokio_tungstenite::tungstenite::protocol::Message::Binary(
                                bin,
                            ))
                            .await;
                    }
                    actix_web_actors::ws::Message::Close(reason) => {
                        let frame = map_reason_to_close_frame(reason.clone());
                        let _ = writer
                            .lock()
                            .await
                            .send(tokio_tungstenite::tungstenite::protocol::Message::Close(
                                frame,
                            ))
                            .await;
                        // @TODO
                        // ctx.close(reason);
                    }
                    actix_web_actors::ws::Message::Ping(bin) => {
                        let _ = writer
                            .lock()
                            .await
                            .send(tokio_tungstenite::tungstenite::protocol::Message::Ping(bin))
                            .await;
                    }
                    actix_web_actors::ws::Message::Pong(bin) => {
                        let _ = writer
                            .lock()
                            .await
                            .send(tokio_tungstenite::tungstenite::protocol::Message::Pong(bin))
                            .await;
                    }
                    _ => {
                        println!("Something bad happended");
                    }
                }
            }
        });
    }
}

impl actix::Handler<ProxyMessage> for ProxyWS {
    type Result = ();

    fn handle(&mut self, msg: ProxyMessage, ctx: &mut Self::Context) -> Self::Result {
        match msg.message {
            tokio_tungstenite::tungstenite::Message::Text(text) => {
                ctx.text(std::str::from_utf8(text.as_bytes()).unwrap()) //@TODO
            }
            tokio_tungstenite::tungstenite::Message::Binary(bytes) => ctx.binary(bytes),
            tokio_tungstenite::tungstenite::Message::Ping(bytes) => ctx.ping(&bytes),
            tokio_tungstenite::tungstenite::Message::Pong(bytes) => ctx.pong(&bytes),
            tokio_tungstenite::tungstenite::Message::Close(close_frame) => ctx.close(None), //@TODO
            tokio_tungstenite::tungstenite::Message::Frame(frame) => unreachable!(),
        }
    }
}
