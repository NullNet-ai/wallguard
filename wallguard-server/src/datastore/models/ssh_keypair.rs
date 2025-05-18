use crate::datastore::db_tables::DBTable;
use crate::utilities;
use nullnet_liberror::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSHKeypair {
    pub device_id: String,
    pub public_key: String,
    pub private_key: String,
    pub passphrase: String,
}

impl SSHKeypair {
    pub fn new(
        device_id: impl Into<String>,
        public_key: impl Into<String>,
        private_key: impl Into<String>,
        passphrase: impl Into<String>,
    ) -> Self {
        Self {
            device_id: device_id.into(),
            public_key: public_key.into(),
            private_key: private_key.into(),
            passphrase: passphrase.into(),
        }
    }

    pub async fn generate(device_id: &str) -> Result<Self, Error> {
        let passphrase = utilities::random::generate_random_string(16);

        let (public_key, private_key) =
            utilities::ssh::generate_keypair(Some(passphrase.clone()), None).await?;

        Ok(Self::new(device_id, public_key, private_key, passphrase))
    }

    pub fn pluck() -> Vec<String> {
        vec![
            "device_id".into(),
            "public_key".into(),
            "private_key".into(),
            "passphrase".into(),
        ]
    }

    pub fn table() -> DBTable {
        DBTable::SSHKeys
    }

    pub fn entity_prefix() -> String {
        "SSH".into()
    }
}
