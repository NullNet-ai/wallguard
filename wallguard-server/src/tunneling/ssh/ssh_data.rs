use nullnet_liberror::Error;

use crate::utilities;

pub struct SshData {
    pub public_key: String,
    pub private_key: String,
    pub passphrase: String,
    pub username: String,
}

impl SshData {
    pub async fn generate(username: String) -> Result<SshData, Error> {
        let passphrase = utilities::random::generate_random_string(16);

        let (public_key, private_key) =
            utilities::ssh::generate_keypair(Some(passphrase.clone()), None).await?;

        Ok(Self {
            public_key,
            private_key,
            passphrase,
            username,
        })
    }
}
