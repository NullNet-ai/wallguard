use crate::utilities::random::generate_random_string;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use tokio::{fs, process::Command};

const DEFAULT_EMAIL: &str = "wallgaurd@nullnet.ai";

/// Generates an SSH keypair using `ssh-keygen` and returns the public and private keys as strings.
/// Temporary files are written to `/tmp` and deleted after reading.
pub async fn generate_keypair(
    passphrase: Option<String>,
    email: Option<String>,
) -> Result<(String, String), Error> {
    let suffix = generate_random_string(8);
    let private_key_path = format!("/tmp/id_ed25519_{suffix}");
    let public_key_path = format!("{private_key_path}.pub");

    let email = email.unwrap_or_else(|| DEFAULT_EMAIL.to_string());
    let passphrase = passphrase.unwrap_or_default();

    let status = Command::new("ssh-keygen")
        .args([
            "-t",
            "ed25519",
            "-f",
            &private_key_path,
            "-C",
            &email,
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

    for path in [&private_key_path, &public_key_path] {
        fs::remove_file(path).await.handle_err(location!())?;
    }

    Ok((public_key, private_key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_keypair_with_no_args() {
        let keypair = generate_keypair(None, None).await;

        assert!(keypair.is_ok());

        let (public, private) = keypair.unwrap();

        assert!(!public.is_empty());
        assert!(!private.is_empty());
    }

    #[tokio::test]
    async fn test_generate_keypair_with_passcode() {
        let keypair = generate_keypair(Some("passcode".into()), None).await;

        assert!(keypair.is_ok());

        let (public, private) = keypair.unwrap();

        assert!(!public.is_empty());
        assert!(!private.is_empty());
    }

    #[tokio::test]
    async fn test_generate_keypair_with_email() {
        let keypair = generate_keypair(None, Some("example@example.com".into())).await;

        assert!(keypair.is_ok());

        let (public, private) = keypair.unwrap();

        assert!(!public.is_empty());
        assert!(!private.is_empty());
    }

    #[tokio::test]
    async fn test_generate_keypair_with_both() {
        let keypair =
            generate_keypair(Some("passcode".into()), Some("example@example.com".into())).await;

        assert!(keypair.is_ok());

        let (public, private) = keypair.unwrap();

        assert!(!public.is_empty());
        assert!(!private.is_empty());
    }
}
