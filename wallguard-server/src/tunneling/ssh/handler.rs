use russh::client::{self};
use russh::keys::ssh_key::PublicKey;

pub struct Handler;

impl client::Handler for Handler {
    type Error = russh::Error;

    async fn check_server_key(&mut self, _: &PublicKey) -> Result<bool, Self::Error> {
        Ok(true)
    }
}
