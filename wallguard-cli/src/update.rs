use anyhow::{Context, Result as AnyResult, bail};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use tonic::transport::Channel;
use wallguard_common::protobuf::wallguard_cli::wallguard_cli_client::WallguardCliClient;

const GITHUB_REPO: &str = "NullNet-ai/wallguard";

#[derive(serde::Deserialize)]
struct GithubRelease {
    tag_name: String,
}

pub async fn run(check_only: bool) -> AnyResult<()> {
    #[cfg(target_os = "macos")]
    {
        println!(
            "wallguard-cli update is not yet supported on macOS. \
             Please download the latest release manually: \
             https://github.com/{GITHUB_REPO}/releases/latest"
        );
        return Ok(());
    }

    #[cfg(target_os = "freebsd")]
    if let Some(platform) = pfsense_or_opnsense_marker() {
        println!(
            "wallguard-cli update does not support {platform} yet — no packaging exists \
             for it and its base system may conflict with an out-of-band binary swap. \
             Please update manually: https://github.com/{GITHUB_REPO}/releases/latest"
        );
        return Ok(());
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "macos",
        windows
    )))]
    bail!("wallguard-cli update is not supported on this platform");

    #[cfg(any(target_os = "linux", target_os = "freebsd", windows))]
    {
        let current = if crate::is_agent_running() {
            agent_reported_version().await
        } else {
            None
        };

        match current.as_deref() {
            Some(v) => println!("Current version: {v}"),
            None => println!("Current version: unknown (agent not running)"),
        }

        let latest_tag = fetch_latest_tag().await?;
        let latest = latest_tag.trim_start_matches('v').to_string();
        println!("Latest version : {latest}");

        let latest_semver = semver::Version::parse(&latest)
            .with_context(|| format!("Latest release tag {latest_tag:?} is not valid semver"))?;

        if let Some(cur) = current
            .as_deref()
            .and_then(|v| semver::Version::parse(v).ok())
            && latest_semver <= cur
        {
            println!("WallGuard is already up to date.");
            return Ok(());
        }

        if check_only {
            println!("A new version is available. Run `wallguard-cli update` to install it.");
            return Ok(());
        }

        apply_update(&latest).await
    }
}

#[cfg(target_os = "freebsd")]
fn pfsense_or_opnsense_marker() -> Option<&'static str> {
    if std::fs::read_to_string("/etc/platform")
        .map(|s| s.to_lowercase().contains("pfsense"))
        .unwrap_or(false)
    {
        return Some("pfSense");
    }
    if Path::new("/usr/local/opnsense").exists() {
        return Some("OPNsense");
    }
    None
}

#[cfg(any(target_os = "linux", target_os = "freebsd", windows))]
async fn apply_update(version: &str) -> AnyResult<()> {
    let (artifact, checksum_file) = artifact_names(version)?;
    let http = reqwest::Client::builder()
        .user_agent("wallguard-cli")
        .build()?;
    let base = format!("https://github.com/{GITHUB_REPO}/releases/download/v{version}");

    println!("Downloading {artifact}...");
    let archive_bytes = http
        .get(format!("{base}/{artifact}"))
        .send()
        .await
        .context("Failed to download update artifact")?
        .error_for_status()
        .context("Update artifact not found for this platform/version")?
        .bytes()
        .await?;

    println!("Verifying checksum...");
    let sums = http
        .get(format!("{base}/{checksum_file}"))
        .send()
        .await
        .context("Failed to download checksum file")?
        .error_for_status()
        .context("Checksum file not found for this release")?
        .text()
        .await?;

    let expected = find_checksum(&sums, &artifact).ok_or_else(|| {
        anyhow::anyhow!("No checksum entry found for {artifact} in {checksum_file}")
    })?;
    let actual = sha256_hex(&archive_bytes);
    if !actual.eq_ignore_ascii_case(&expected) {
        bail!(
            "Checksum mismatch for {artifact}: expected {expected}, got {actual}. \
             Aborting — nothing was changed."
        );
    }

    let live_binary = live_binary_path()?;
    let parent = live_binary
        .parent()
        .context("Live binary path has no parent directory")?
        .to_path_buf();
    let tmp_path = parent.join(format!(".wallguard-update-{}", std::process::id()));

    extract_binary(&archive_bytes, &tmp_path)?;

    let was_running = crate::is_agent_running();
    let captured_args = crate::capture_agent_args();

    if was_running {
        println!("Stopping WallGuard agent for update...");
        let lock_path = wallguard_common::single_instance::agent_lock_path();

        if let Some(mut client) = try_connect().await {
            let _ = client.shutdown(()).await;
        }

        if !crate::wait_for_lock_free(&lock_path, 30, Duration::from_millis(500)).await {
            let _ = std::fs::remove_file(&tmp_path);
            bail!(
                "WallGuard agent did not shut down within 15s. Aborting update — no \
                 files were changed. Check /var/log/wallguard.log, or run \
                 `wallguard-cli stop` manually before retrying."
            );
        }
    }

    let backup_path = parent.join(format!(
        "{}.bak",
        live_binary
            .file_name()
            .context("Live binary path has no file name")?
            .to_string_lossy()
    ));

    if live_binary.exists() {
        println!("Backing up current binary to {}", backup_path.display());
        std::fs::rename(&live_binary, &backup_path)
            .context("Failed to back up the current wallguard binary")?;
    }

    println!("Installing new binary...");
    if let Err(err) = std::fs::rename(&tmp_path, &live_binary) {
        // Try to restore the backup so we don't leave the box with no binary at all.
        let _ = std::fs::rename(&backup_path, &live_binary);
        return Err(err).context("Failed to install the new wallguard binary");
    }

    if !was_running {
        println!(
            "WallGuard v{version} installed. The agent was not running before the update, \
             so it was not restarted — run `wallguard-cli start ...` when ready."
        );
        return Ok(());
    }

    println!("Restarting WallGuard agent...");
    restart_agent(&live_binary, captured_args.as_deref().unwrap_or(&[])).await?;

    // Polls the agent's gRPC server directly rather than probing the
    // single-instance lock first: taking that lock ourselves, even just to
    // peek and immediately release it, races the agent's own first
    // acquisition attempt and can make it think a duplicate instance is
    // already running. Reading the version over the wire never contends
    // for the lock at all.
    let new_version = poll_agent_version(20, Duration::from_millis(500)).await;

    if new_version.as_deref().is_some_and(|v| versions_match(v, version)) {
        let _ = std::fs::remove_file(&backup_path);
        println!("WallGuard successfully updated to v{version}.");
        return Ok(());
    }

    eprintln!("Update health check failed (agent did not come up as v{version}). Rolling back...");
    rollback(&live_binary, &backup_path, captured_args.as_deref())
        .await
        .context("Update failed AND automatic rollback failed")?;

    bail!(
        "Update to v{version} failed and was rolled back to the previous version. \
         Check /var/log/wallguard.log for details before retrying."
    );
}

#[cfg(any(target_os = "linux", target_os = "freebsd", windows))]
async fn rollback(
    live_binary: &Path,
    backup_path: &Path,
    captured_args: Option<&[String]>,
) -> AnyResult<()> {
    if crate::is_agent_running() {
        if let Some(mut client) = try_connect().await {
            let _ = client.shutdown(()).await;
        }
        let lock_path = wallguard_common::single_instance::agent_lock_path();
        if !crate::wait_for_lock_free(&lock_path, 10, Duration::from_millis(500)).await {
            crate::hard_kill_agent();
        }
    }

    if !backup_path.exists() {
        bail!(
            "No backup binary found at {}. Manual recovery required.",
            backup_path.display()
        );
    }

    std::fs::rename(backup_path, live_binary).with_context(|| {
        format!(
            "Failed to restore backup from {} to {}. Manual recovery required.",
            backup_path.display(),
            live_binary.display()
        )
    })?;

    restart_agent(live_binary, captured_args.unwrap_or(&[])).await?;

    // See the comment in `apply_update`: probe over gRPC, not the lock.
    if poll_agent_version(20, Duration::from_millis(500))
        .await
        .is_some()
    {
        println!("Rolled back to the previous version successfully.");
        Ok(())
    } else {
        bail!(
            "Rolled-back agent did not come back up either. The previous binary was \
             restored to {} but is not running — start it manually.",
            live_binary.display()
        )
    }
}

/// Restarts the agent, preferring the platform's service manager
/// (systemd/rc.d) over a bare spawn if the agent is registered as a
/// supervised service — see each platform's `restart_via_service_manager`
/// for why (in short: a bare spawn racing systemd's `Restart=always` can
/// lose that race under I/O contention and end up with the wrong process
/// holding the single-instance lock). Falls back to a bare spawn for
/// installs that were never brought up via `wallguard-cli start`, or on
/// platforms where the service manager can't race a bare spawn anyway.
///
/// Note: when restarting via the service manager, the args baked into its
/// unit file (from the last `start`) are used, not `args` — this can only
/// differ if the agent is currently running with args that were never
/// persisted via `start`, an unsupported/unusual setup.
#[cfg(any(target_os = "linux", target_os = "freebsd", windows))]
async fn restart_agent(binary: &Path, args: &[String]) -> AnyResult<()> {
    if crate::autostart::restart_via_service_manager("wallguard")
        .await
        .unwrap_or(false)
    {
        return Ok(());
    }
    spawn_agent(binary, args)
}

#[cfg(any(target_os = "linux", target_os = "freebsd", windows))]
fn spawn_agent(binary: &Path, args: &[String]) -> AnyResult<()> {
    Command::new(binary)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to spawn wallguard agent")?;
    Ok(())
}

#[cfg(any(target_os = "linux", target_os = "freebsd", windows))]
fn live_binary_path() -> AnyResult<PathBuf> {
    #[cfg(windows)]
    {
        let dir = std::env::current_exe()?
            .parent()
            .context("wallguard-cli.exe has no parent directory")?
            .to_path_buf();
        Ok(dir.join("wallguard.exe"))
    }
    #[cfg(not(windows))]
    {
        Ok(PathBuf::from("/usr/local/bin/wallguard"))
    }
}

#[cfg(any(target_os = "linux", target_os = "freebsd", windows))]
fn artifact_names(version: &str) -> AnyResult<(String, String)> {
    #[cfg(target_os = "linux")]
    {
        let arch = match std::env::consts::ARCH {
            "x86_64" => "x86_64",
            "aarch64" => "aarch64",
            other => bail!("Unsupported architecture for update: {other}"),
        };
        Ok((
            format!("wallguard-{version}-linux-{arch}.tar.gz"),
            "SHA256SUMS-deb".to_string(),
        ))
    }
    #[cfg(target_os = "freebsd")]
    {
        Ok((
            format!("wallguard-{version}-freebsd-x86_64.tar.gz"),
            "SHA256SUMS-freebsd".to_string(),
        ))
    }
    #[cfg(windows)]
    {
        Ok((
            format!("wallguard-{version}-windows-x86_64.zip"),
            "SHA256SUMS-windows".to_string(),
        ))
    }
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn extract_binary(archive_bytes: &[u8], dest: &Path) -> AnyResult<()> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let gz = GzDecoder::new(archive_bytes);
    let mut archive = Archive::new(gz);
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.into_owned();
        if path.file_name().is_some_and(|n| n == "wallguard") {
            let mut out = std::fs::File::create(dest)?;
            std::io::copy(&mut entry, &mut out)?;

            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(dest, std::fs::Permissions::from_mode(0o755))?;

            return Ok(());
        }
    }
    bail!("wallguard binary not found inside downloaded archive")
}

#[cfg(windows)]
fn extract_binary(archive_bytes: &[u8], dest: &Path) -> AnyResult<()> {
    use std::io::Cursor;

    let mut zip = zip::ZipArchive::new(Cursor::new(archive_bytes))?;
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        if file.name().ends_with("wallguard.exe") {
            let mut out = std::fs::File::create(dest)?;
            std::io::copy(&mut file, &mut out)?;
            return Ok(());
        }
    }
    bail!("wallguard.exe not found inside downloaded archive")
}

#[cfg(any(target_os = "linux", target_os = "freebsd", windows))]
fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

#[cfg(any(target_os = "linux", target_os = "freebsd", windows))]
fn find_checksum(sums: &str, filename: &str) -> Option<String> {
    sums.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let hash = parts.next()?;
        let name = parts.next()?.trim_start_matches('*');
        (name == filename).then(|| hash.to_string())
    })
}

#[cfg(any(target_os = "linux", target_os = "freebsd", windows))]
async fn fetch_latest_tag() -> AnyResult<String> {
    let url = format!("https://api.github.com/repos/{GITHUB_REPO}/releases/latest");
    let client = reqwest::Client::builder()
        .user_agent("wallguard-cli")
        .build()?;

    let release: GithubRelease = client
        .get(&url)
        .send()
        .await
        .context("Failed to reach GitHub to check for updates")?
        .error_for_status()
        .context("GitHub API returned an error while checking for updates")?
        .json()
        .await
        .context("Failed to parse GitHub release response")?;

    Ok(release.tag_name)
}

/// Same connection this CLI otherwise uses (`cli_connect` in `main.rs`), but
/// returns `None` on failure instead of exiting the process — needed here
/// since a connection failure mid-update is a signal to roll back, not to
/// abort the whole CLI process outright.
#[cfg(any(target_os = "linux", target_os = "freebsd", windows))]
async fn try_connect() -> Option<crate::Client> {
    const EXPECTED_ADDR: &str = "http://127.0.0.1:54056";

    let channel = Channel::from_shared(EXPECTED_ADDR)
        .ok()?
        .timeout(Duration::from_secs(5))
        .connect()
        .await
        .ok()?;

    Some(WallguardCliClient::new(channel))
}

#[cfg(any(target_os = "linux", target_os = "freebsd", windows))]
async fn agent_reported_version() -> Option<String> {
    let mut client = try_connect().await?;
    client
        .get_version(())
        .await
        .ok()
        .map(|r| r.into_inner().value)
}

/// Retries `agent_reported_version` instead of asking once. The
/// single-instance lock is acquired at the very top of the agent's
/// `main()`, well before it resolves `control_channel_url` (a blocking DNS
/// lookup for hostnames), joins its org, and finally binds the CLI gRPC
/// server — so a lock-is-held signal does not mean the server is ready to
/// answer `GetVersion` yet. Polling for a while absorbs that startup
/// latency instead of misreading it as "the new version failed to come up".
/// Compares a version reported by the agent's `GetVersion` RPC against the
/// expected release version by semver value rather than exact string
/// equality, so a leading `v`, incidental whitespace, or a difference in
/// zero-padding doesn't fail the health check for what is actually the same
/// version. Falls back to a trimmed string comparison if either side isn't
/// valid semver, rather than silently treating an unparsable version as a
/// match.
#[cfg(any(target_os = "linux", target_os = "freebsd", windows))]
fn versions_match(reported: &str, expected: &str) -> bool {
    let normalize = |s: &str| s.trim().trim_start_matches('v').to_string();
    let (reported, expected) = (normalize(reported), normalize(expected));

    match (
        semver::Version::parse(&reported),
        semver::Version::parse(&expected),
    ) {
        (Ok(r), Ok(e)) => r == e,
        _ => reported == expected,
    }
}

#[cfg(any(target_os = "linux", target_os = "freebsd", windows))]
async fn poll_agent_version(attempts: u32, delay: Duration) -> Option<String> {
    for _ in 0..attempts {
        if let Some(version) = agent_reported_version().await {
            return Some(version);
        }
        tokio::time::sleep(delay).await;
    }
    None
}

#[cfg(all(test, any(target_os = "linux", target_os = "freebsd")))]
mod tests {
    use super::*;

    #[test]
    fn sha256_matches_known_vectors() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn versions_match_ignores_v_prefix_and_whitespace() {
        assert!(versions_match("1.3.8", "1.3.8"));
        assert!(versions_match("v1.3.8", "1.3.8"));
        assert!(versions_match(" 1.3.8 \n", "1.3.8"));
    }

    #[test]
    fn versions_match_rejects_different_versions() {
        assert!(!versions_match("1.3.5", "1.3.8"));
    }

    #[test]
    fn versions_match_falls_back_to_string_equality_for_invalid_semver() {
        assert!(versions_match("not-a-version", "not-a-version"));
        assert!(!versions_match("not-a-version", "1.3.8"));
    }

    #[test]
    fn find_checksum_parses_gnu_sha256sum_format() {
        let sums =
            "aaaa111  wallguard-1.4.0-linux-x86_64.tar.gz\nbbbb222  wallguard_1.4.0_amd64.deb\n";
        assert_eq!(
            find_checksum(sums, "wallguard-1.4.0-linux-x86_64.tar.gz"),
            Some("aaaa111".to_string())
        );
        assert_eq!(
            find_checksum(sums, "wallguard_1.4.0_amd64.deb"),
            Some("bbbb222".to_string())
        );
        assert_eq!(find_checksum(sums, "does-not-exist.tar.gz"), None);
    }

    #[test]
    fn find_checksum_parses_bsd_sha256_dash_r_format() {
        // FreeBSD's `sha256 -r` uses a single space and no `*` marker,
        // unlike GNU sha256sum's two-space/binary-mode format.
        let sums = "cccc333 wallguard-1.4.0-freebsd-x86_64.tar.gz\n";
        assert_eq!(
            find_checksum(sums, "wallguard-1.4.0-freebsd-x86_64.tar.gz"),
            Some("cccc333".to_string())
        );
    }

    #[test]
    fn find_checksum_strips_binary_mode_marker() {
        // GNU sha256sum without --text prefixes the filename with `*`.
        let sums = "dddd444  *wallguard-1.4.0-linux-x86_64.tar.gz\n";
        assert_eq!(
            find_checksum(sums, "wallguard-1.4.0-linux-x86_64.tar.gz"),
            Some("dddd444".to_string())
        );
    }

    /// Builds an in-memory .tar.gz with a `wallguard` and a `wallguard-cli`
    /// entry, matching `packbuild.sh`'s `tarball()` layout exactly (both
    /// binaries at the archive root, no directory prefix).
    fn build_test_archive() -> Vec<u8> {
        use flate2::Compression;
        use flate2::write::GzEncoder;

        let mut tar_bytes = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut tar_bytes);

            let mut agent_header = tar::Header::new_gnu();
            agent_header.set_size(b"AGENT-BINARY-CONTENTS".len() as u64);
            agent_header.set_mode(0o755);
            agent_header.set_cksum();
            builder
                .append_data(
                    &mut agent_header,
                    "wallguard",
                    &b"AGENT-BINARY-CONTENTS"[..],
                )
                .unwrap();

            let mut cli_header = tar::Header::new_gnu();
            cli_header.set_size(b"CLI-BINARY-CONTENTS".len() as u64);
            cli_header.set_mode(0o755);
            cli_header.set_cksum();
            builder
                .append_data(
                    &mut cli_header,
                    "wallguard-cli",
                    &b"CLI-BINARY-CONTENTS"[..],
                )
                .unwrap();

            builder.finish().unwrap();
        }

        let mut gz_bytes = Vec::new();
        {
            let mut encoder = GzEncoder::new(&mut gz_bytes, Compression::default());
            std::io::Write::write_all(&mut encoder, &tar_bytes).unwrap();
            encoder.finish().unwrap();
        }
        gz_bytes
    }

    #[test]
    fn extract_binary_picks_wallguard_not_wallguard_cli() {
        let archive = build_test_archive();
        let dest = std::env::temp_dir().join(format!(
            "wallguard-update-test-{}-{}",
            std::process::id(),
            "extract-picks-agent"
        ));

        extract_binary(&archive, &dest).expect("extraction should succeed");
        let extracted = std::fs::read(&dest).expect("extracted file should exist");
        std::fs::remove_file(&dest).ok();

        assert_eq!(extracted, b"AGENT-BINARY-CONTENTS");
    }

    #[test]
    fn extract_binary_fails_clearly_when_entry_missing() {
        // An archive containing only the CLI binary, no agent binary.
        let mut tar_bytes = Vec::new();
        {
            let mut builder = tar::Builder::new(&mut tar_bytes);
            let mut header = tar::Header::new_gnu();
            header.set_size(b"CLI-ONLY".len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            builder
                .append_data(&mut header, "wallguard-cli", &b"CLI-ONLY"[..])
                .unwrap();
            builder.finish().unwrap();
        }
        let mut gz_bytes = Vec::new();
        {
            use flate2::Compression;
            use flate2::write::GzEncoder;
            let mut encoder = GzEncoder::new(&mut gz_bytes, Compression::default());
            std::io::Write::write_all(&mut encoder, &tar_bytes).unwrap();
            encoder.finish().unwrap();
        }

        let dest = std::env::temp_dir().join(format!(
            "wallguard-update-test-{}-{}",
            std::process::id(),
            "extract-missing"
        ));
        let result = extract_binary(&gz_bytes, &dest);
        assert!(result.is_err());
        assert!(!dest.exists());
    }
}
