use std::path::PathBuf;
use tokio::fs::{self, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub async fn add_ssh_key_if_missing(public_key: &str) -> std::io::Result<()> {
    let mut auth_keys_path = PathBuf::from("/root");
    auth_keys_path.push(".ssh");
    fs::create_dir_all(&auth_keys_path).await?;

    auth_keys_path.push("authorized_keys");

    if fs::metadata(&auth_keys_path).await.is_ok() {
        let file = fs::File::open(&auth_keys_path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            if line.trim() == public_key.trim() {
                return Ok(());
            }
        }
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&auth_keys_path)
        .await?;

    file.write_all(public_key.trim().as_bytes()).await?;
    file.write_all(b"\n").await?;

    Ok(())
}
