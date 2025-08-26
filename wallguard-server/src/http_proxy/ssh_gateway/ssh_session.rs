use std::sync::Arc;

use crate::datastore::SSHKeypair;
use crate::http_proxy::ssh_gateway::ssh_handler::SSHHandler;
use crate::reverse_tunnel::TunnelAdapter;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use russh::ChannelStream;
use russh::client::{self, AuthResult, Msg};
use russh::keys::{PrivateKeyWithHashAlg, decode_secret_key};
use tokio::io::{ReadHalf, WriteHalf};
use tokio::sync::Mutex;

type Reader = ReadHalf<ChannelStream<Msg>>;
type Writer = WriteHalf<ChannelStream<Msg>>;

#[derive(Clone)]
pub(crate) struct SSHSession {
    pub(crate) reader: Arc<Mutex<Reader>>,
    pub(crate) writer: Arc<Mutex<Writer>>,
}

impl SSHSession {
    pub async fn new(tunnel: TunnelAdapter, key: &SSHKeypair) -> Result<Self, Error> {
        let handler = SSHHandler;
        let private_key =
            decode_secret_key(&key.private_key, Some(&key.passphrase)).handle_err(location!())?;

        let config = client::Config::default();

        let mut session = client::connect_stream(Arc::new(config), tunnel, handler)
            .await
            .handle_err(location!())?;

        let auth_response = session
            .authenticate_publickey(
                "root",
                PrivateKeyWithHashAlg::new(Arc::new(private_key), None),
            )
            .await
            .handle_err(location!())?;

        if !matches!(auth_response, AuthResult::Success) {
            return Err("SSH authentication failed - check the keypair").handle_err(location!());
        }

        let channel = session
            .channel_open_session()
            .await
            .handle_err(location!())?;

        channel
            .request_pty(false, "xterm", 80, 24, 0, 0, &[])
            .await
            .handle_err(location!())?;

        channel.request_shell(false).await.handle_err(location!())?;

        let (reader, writer) = tokio::io::split(channel.into_stream());

        Ok(Self {
            reader: Arc::new(Mutex::new(reader)),
            writer: Arc::new(Mutex::new(writer)),
        })
    }

    // pub async fn new(stream: TcpStream, key: &SSHKeypair) -> Result<Self, Error> {
    //     let mut session = AsyncSession::new(stream, None).handle_err(location!())?;

    //     session.handshake().await.handle_err(location!())?;

    //     session
    //         .userauth_pubkey_memory(
    //             "root",
    //             Some(&key.public_key),
    //             &key.private_key,
    //             Some(&key.passphrase),
    //         )
    //         .await
    //         .handle_err(location!())?;

    //     session
    //         .authenticated()
    //         .then_some(())
    //         .ok_or("SSH Session authentication failed")
    //         .handle_err(location!())?;

    //     let mut channel = session.channel_session().await.handle_err(location!())?;

    //     channel
    //         .request_pty("xterm", None, None)
    //         .await
    //         .handle_err(location!())?;

    //     channel.shell().await.handle_err(location!())?;

    //     let (reader, writer) = tokio::io::split(channel);

    //     Ok(Self {
    //         reader: Arc::new(Mutex::new(reader)),
    //         writer: Arc::new(Mutex::new(writer)),
    //     })
    // }
}
