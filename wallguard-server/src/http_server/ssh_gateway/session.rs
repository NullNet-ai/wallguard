use super::{ssh_message::SSHMessage, stop_message::StopSession};
use crate::{http_server::proxy::auth::authenticate, ssh_keypair::SSHKeypair};
use actix::{AsyncContext, StreamHandler};
use async_ssh2_lite::{AsyncChannel, AsyncSession};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    net::TcpStream,
    sync::Mutex,
};

type Reader = ReadHalf<AsyncChannel<TcpStream>>;
type Writer = WriteHalf<AsyncChannel<TcpStream>>;

pub(super) struct Session {
    reader: Arc<Mutex<Reader>>,
    writer: Arc<Mutex<Writer>>,
}

impl Session {
    pub async fn new(
        addr: SocketAddr,
        visitor_token: Option<String>,
        key: &SSHKeypair,
    ) -> Result<Self, Error> {
        let mut stream = TcpStream::connect(addr).await.handle_err(location!())?;

        if let Some(token) = visitor_token {
            authenticate(&mut stream, &token)
                .await
                .handle_err(location!())?;
        }

        let mut session = AsyncSession::new(stream, None).handle_err(location!())?;

        session.handshake().await.handle_err(location!())?;

        session
            .userauth_pubkey_memory(
                "root",
                Some(&key.public_key),
                &key.private_key,
                Some(&key.passphrase),
            )
            .await
            .handle_err(location!())?;

        session
            .authenticated()
            .then_some(())
            .ok_or("SSH Session authentication failed")
            .handle_err(location!())?;

        let mut channel = session.channel_session().await.handle_err(location!())?;

        channel
            .request_pty("xterm", None, None)
            .await
            .handle_err(location!())?;

        channel.shell().await.handle_err(location!())?;

        let (reader, writer) = tokio::io::split(channel);

        Ok(Self {
            reader: Arc::new(Mutex::new(reader)),
            writer: Arc::new(Mutex::new(writer)),
        })
    }
}

impl actix::Actor for Session {
    type Context = actix_web_actors::ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let address = ctx.address();
        let reader = self.reader.clone();

        tokio::spawn(async move {
            loop {
                let mut buf = [0u8; 8196];
                match reader.lock().await.read(&mut buf).await {
                    Ok(0) => log::debug!("SSH Session: Read EOF"),
                    Ok(n) => address.do_send(SSHMessage::from(&buf[..n])),
                    Err(err) => {
                        log::error!("SSH Session: Failed to read bytes: {}", err);
                        address.do_send(StopSession {});
                        break;
                    }
                }
            }
        });
    }
}

impl StreamHandler<Result<actix_web_actors::ws::Message, actix_web_actors::ws::ProtocolError>>
    for Session
{
    fn handle(
        &mut self,
        msg: Result<actix_web_actors::ws::Message, actix_web_actors::ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        let Ok(message) = msg else {
            log::error!("Recevied an error instead of message: {}", msg.unwrap_err());
            return;
        };

        use actix_web_actors::ws::Message::*;

        match message {
            Ping(bytes) => {
                ctx.pong(&bytes);
            }
            Pong(_) | Nop => {
                // Do nothing
            }
            Close(reason) => {
                ctx.close(reason);
            }
            Continuation(_) => {
                log::error!("SSH Session: Received unsupported continuation frame");
                ctx.close(None);
            }
            Text(text) => {
                let writer = self.writer.clone();
                let bytes = text.into_bytes(); // safely consumes the string
                tokio::spawn(async move {
                    let _ = writer
                        .lock()
                        .await
                        .write(&bytes)
                        .await
                        .handle_err(location!());
                });
            }
            Binary(bytes) => {
                let writer = self.writer.clone();
                tokio::spawn(async move {
                    let _ = writer
                        .lock()
                        .await
                        .write(&bytes)
                        .await
                        .handle_err(location!());
                });
            }
        }
    }
}

impl actix::Handler<SSHMessage> for Session {
    type Result = ();

    fn handle(&mut self, msg: SSHMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.binary(msg.data);
    }
}

impl actix::Handler<StopSession> for Session {
    type Result = ();

    fn handle(&mut self, _msg: StopSession, ctx: &mut Self::Context) -> Self::Result {
        ctx.close(None);
    }
}
