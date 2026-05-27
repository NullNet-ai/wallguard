use std::io;
use std::path::PathBuf;
use tokio::fs::{self, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

// tokio::process::Command is only needed on Unix (sshd -T).
#[cfg(unix)]
use tokio::process::Command;

// ── Authorized-keys path ──────────────────────────────────────────────────────

/// Returns the full path to the `authorized_keys` file for `username`.
///
/// Unix:
///   * `root`         → `/root/.ssh/authorized_keys`
///   * other users    → `/home/<username>/.ssh/authorized_keys`
///
/// Windows (OpenSSH for Windows):
///   * `Administrator` → `%PROGRAMDATA%\ssh\administrators_authorized_keys`
///     (Windows OpenSSH reads this shared file for all members of the
///     Administrators group — using per-user `authorized_keys` for admin
///     accounts is explicitly not supported.)
///   * other users    → `%SYSTEMDRIVE%\Users\<username>\.ssh\authorized_keys`
#[cfg(unix)]
fn authorized_keys_path(username: &str) -> PathBuf {
    if username == "root" {
        PathBuf::from("/root/.ssh/authorized_keys")
    } else {
        PathBuf::from(format!("/home/{}/.ssh/authorized_keys", username))
    }
}

#[cfg(windows)]
fn authorized_keys_path(username: &str) -> PathBuf {
    if username.eq_ignore_ascii_case("administrator") {
        let programdata = std::env::var("PROGRAMDATA")
            .unwrap_or_else(|_| r"C:\ProgramData".to_string());
        PathBuf::from(format!(r"{}\ssh\administrators_authorized_keys", programdata))
    } else {
        let system_drive = std::env::var("SYSTEMDRIVE").unwrap_or_else(|_| "C:".to_string());
        PathBuf::from(format!(r"{}\Users\{}\.ssh\authorized_keys", system_drive, username))
    }
}

// ── add_ssh_key_if_missing ────────────────────────────────────────────────────

pub async fn add_ssh_key_if_missing(public_key: &str, username: &str) -> std::io::Result<()> {
    let auth_keys_path = authorized_keys_path(username);

    // Create the parent directory (.ssh / ssh) if it doesn't exist yet.
    if let Some(dir) = auth_keys_path.parent() {
        fs::create_dir_all(dir).await?;
    }

    // Skip appending if the key is already present.
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

// ── get_sshd_ports_from_sshd_t ────────────────────────────────────────────────

/// Unix: ask the live daemon via `sshd -T` (authoritative; resolves Include
/// directives and runtime defaults).
#[cfg(unix)]
pub async fn get_sshd_ports_from_sshd_t() -> io::Result<Vec<u16>> {
    let output = Command::new("sshd").arg("-T").output().await?;

    if !output.status.success() {
        return Err(io::Error::other(format!(
            "sshd -T failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut ports = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if let Some(stripped) = line.strip_prefix("port ") {
            for value in stripped.split_whitespace() {
                if let Ok(port) = value.parse::<u16>() {
                    ports.push(port);
                }
            }
        }
    }

    Ok(ports)
}

/// Windows: `sshd -T` is not supported by Windows OpenSSH.  Parse
/// `%PROGRAMDATA%\ssh\sshd_config` directly instead.
/// Falls back to port 22 when the config file is absent or contains no
/// explicit `Port` directive (22 is the OpenSSH default).
#[cfg(windows)]
pub async fn get_sshd_ports_from_sshd_t() -> io::Result<Vec<u16>> {
    let programdata = std::env::var("PROGRAMDATA")
        .unwrap_or_else(|_| r"C:\ProgramData".to_string());
    let config_path = format!(r"{}\ssh\sshd_config", programdata);

    let content = match fs::read_to_string(&config_path).await {
        Ok(c) => c,
        // OpenSSH not installed or not yet configured — fall back to default.
        Err(_) => return Ok(vec![22]),
    };

    let mut ports = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines.
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // sshd_config keywords are case-insensitive.
        let lower = line.to_ascii_lowercase();
        if let Some(rest) = lower.strip_prefix("port ") {
            if let Ok(port) = rest.trim().parse::<u16>() {
                ports.push(port);
            }
        }
    }

    if ports.is_empty() {
        ports.push(22);
    }

    Ok(ports)
}
