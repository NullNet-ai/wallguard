use std::sync::Arc;

use crate::datastore::SSHKeypair;
use async_ssh2_lite::{AsyncChannel, AsyncSession};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

type Reader = ReadHalf<AsyncChannel<TcpStream>>;
type Writer = WriteHalf<AsyncChannel<TcpStream>>;

#[derive(Clone)]
pub(crate) struct SSHSession {
    pub(crate) reader: Arc<Mutex<Reader>>,
    pub(crate) writer: Arc<Mutex<Writer>>,
}

impl SSHSession {
    pub async fn new(stream: TcpStream, key: &SSHKeypair) -> Result<Self, Error> {
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
