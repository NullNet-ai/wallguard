use std::io;
use tokio::process::Command;

/// Registers `program` as a Windows service via `sc.exe`.
///
/// The binary is located by looking in the same directory as the current
/// executable (wallguard-cli.exe and wallguard.exe are installed side-by-side
/// by the MSI into `C:\Program Files\WallGuard\`).
///
/// The service is registered with `start= demand` so it does not run at boot
/// unless explicitly enabled.  Use `wallguard-cli start` to launch it.
pub async fn enable_service(program: &str, args: &[&str]) -> io::Result<()> {
    // Resolve the binary path relative to this executable's directory.
    let exe_dir = std::env::current_exe()
        .map_err(io::Error::other)?
        .parent()
        .ok_or_else(|| io::Error::other("could not determine executable directory"))?
        .to_path_buf();

    let bin_path = exe_dir.join(format!("{}.exe", program));

    // Build the full binPath value: "C:\...\wallguard.exe" [--arg value …]
    let mut bin_path_value = format!("\"{}\"", bin_path.display());
    if !args.is_empty() {
        bin_path_value.push(' ');
        bin_path_value.push_str(&args.join(" "));
    }

    let output = Command::new("sc")
        .arg("create")
        .arg(program)
        .arg("binPath=")
        .arg(&bin_path_value)
        .arg("displayname=")
        .arg("WallGuard Agent")
        .arg("start=")
        .arg("demand")
        .output()
        .await?;

    if !output.status.success() {
        return Err(io::Error::other(format!(
            "sc create failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Stops and removes the Windows service registered by [`enable_service`].
pub async fn disable_service(program: &str) -> io::Result<()> {
    // Best-effort stop — ignore errors if the service is not running.
    let _ = Command::new("sc").args(["stop", program]).output().await;

    let output = Command::new("sc")
        .args(["delete", program])
        .output()
        .await?;

    if !output.status.success() {
        return Err(io::Error::other(format!(
            "sc delete failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}
