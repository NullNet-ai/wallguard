use nullnet_liberror::{Error, ErrorHandler, Location, location};
use rand::{Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use tokio::{fs, process::Command};

const WALLGUARD_SYSTEM_EMAIL: &str = "wallguard-system@nullnet.ai";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSHKeypair {
    pub public_key: String,
    pub private_key: String,
    pub passphrase: String,
}

impl SSHKeypair {
    pub async fn generate() -> Result<Self, Error> {
        let passphrase = Self::random_passphrase(32);

        let suffix = Self::random_passphrase(16);

        let private_key_path = format!("/tmp/id_ed25519_{suffix}");
        let public_key_path = format!("/tmp/id_ed25519_{suffix}.pub");

        let status = Command::new("ssh-keygen")
            .args([
                "-t",
                "ed25519",
                "-f",
                &private_key_path,
                "-C",
                WALLGUARD_SYSTEM_EMAIL,
                "-N",
                &passphrase,
            ])
            .status()
            .await
            .handle_err(location!())?;

        if !status.success() {
            return Err("Failed to generate SSH keypair with ssh-keygen").handle_err(location!());
        }

        let private_key = fs::read_to_string(&private_key_path)
            .await
            .handle_err(location!())?;

        let public_key = fs::read_to_string(&public_key_path)
            .await
            .handle_err(location!())?;

        fs::remove_file(&private_key_path)
            .await
            .handle_err(location!())?;
        fs::remove_file(&public_key_path)
            .await
            .handle_err(location!())?;

        Ok(Self {
            public_key,
            private_key,
            passphrase,
        })
    }

    fn random_passphrase(length: usize) -> String {
        rand::rngs::ThreadRng::default()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_keypair() {
        let keypair = SSHKeypair::generate().await;

        assert!(keypair.is_ok());

        let keypair = keypair.unwrap();

        assert!(!keypair.public_key.is_empty());
        assert!(!keypair.private_key.is_empty());
        assert!(!keypair.passphrase.is_empty());
    }
}
