use russh::client::{self};
use russh::keys::ssh_key;

pub struct SSHHandler;

impl client::Handler for SSHHandler {
    type Error = russh::Error;

    #[allow(unused_variables)]
    async fn check_server_key(
        &mut self,
        server_public_key: &ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}
