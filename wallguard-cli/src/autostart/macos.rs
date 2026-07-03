use std::io;
use tokio::{fs, process::Command};

/// The launchd label used for the WallGuard daemon plist.
/// Must be stable across enable/disable calls.
const LAUNCHD_LABEL: &str = "ai.nullnet.wallguard";
const LAUNCHD_DIR: &str = "/Library/LaunchDaemons";

fn plist_path() -> String {
    format!("{}/{}.plist", LAUNCHD_DIR, LAUNCHD_LABEL)
}

/// Installs a launchd plist into /Library/LaunchDaemons and loads it.
///
/// The plist is owned by root:wheel with mode 644, as required by launchd
/// for daemons in that directory.  The service is set to start at boot
/// (RunAtLoad = true) and restart on exit (KeepAlive = true).
#[allow(dead_code)]
pub async fn enable_service(program: &str, args: &[&str]) -> io::Result<()> {
    let exe_path = format!("/usr/local/bin/{}", program);
    let path = plist_path();

    // Build the <array> of ProgramArguments entries.
    let mut prog_args = format!("        <string>{}</string>\n", exe_path);
    for arg in args {
        prog_args.push_str(&format!("        <string>{}</string>\n", arg));
    }

    let content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
    "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
{prog_args}    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/var/log/{program}.log</string>
    <key>StandardErrorPath</key>
    <string>/var/log/{program}.log</string>
</dict>
</plist>
"#,
        label = LAUNCHD_LABEL,
        prog_args = prog_args,
        program = program,
    );

    fs::write(&path, &content).await?;

    // launchd requires root:wheel ownership and mode 644.
    let _ = Command::new("chown")
        .args(["root:wheel", &path])
        .output()
        .await;
    let _ = Command::new("chmod").args(["644", &path]).output().await;

    // Load (and enable) the daemon.
    Command::new("launchctl")
        .args(["load", "-w", &path])
        .output()
        .await?;

    Ok(())
}

/// Unloads and removes the launchd plist installed by [`enable_service`].
pub async fn disable_service(_program: &str) -> io::Result<()> {
    let path = plist_path();

    // Unload first — best-effort, ignore errors if not currently loaded.
    let _ = Command::new("launchctl")
        .args(["unload", "-w", &path])
        .output()
        .await;

    if tokio::fs::try_exists(&path).await.unwrap_or(false) {
        fs::remove_file(&path).await?;
    }

    Ok(())
}
