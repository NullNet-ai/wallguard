use russh::client::{self};
use russh::keys::ssh_key;

pub struct SSHHandler;

impl client::Handler for SSHHandler {
    type Error = russh::Error;

    #[allow(unused_variables)]
    fn check_server_key(
        &mut self,
        server_public_key: &ssh_key::PublicKey,
    ) -> impl Future<Output = Result<bool, Self::Error>> + Send {
        // @TODO: Have clients send server's public key and verify it.
        async { Ok(true) }
    }
}
