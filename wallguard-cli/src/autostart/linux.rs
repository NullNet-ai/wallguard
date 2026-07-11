use std::io;
use std::path::Path;
use tokio::fs;
use tokio::process::Command;

const SYSTEMD_DIR: &str = "/etc/systemd/system";

pub async fn enable_service(program: &str, args: &[&str]) -> io::Result<()> {
    let service_name = format!("{}.service", program);
    let service_path = format!("{}/{}", SYSTEMD_DIR, service_name);

    create_unit_file(program, args, &service_path).await?;

    run_systemctl(&["daemon-reload"]).await?;
    // `--now` both enables the unit for future boots and starts it right
    // away, so the currently running agent is supervised by systemd
    // (Restart=always) from this point on instead of running as a bare
    // orphan process until the next reboot.
    run_systemctl(&["enable", "--now", &service_name]).await?;

    Ok(())
}

pub async fn disable_service(program: &str) -> io::Result<()> {
    let service_name = format!("{}.service", program);
    let service_path = format!("{}/{}", SYSTEMD_DIR, service_name);

    let _ = run_systemctl(&["disable", &service_name]).await;

    if Path::new(&service_path).exists() {
        fs::remove_file(&service_path).await?;
    }

    run_systemctl(&["daemon-reload"]).await?;

    Ok(())
}

async fn create_unit_file(program: &str, args: &[&str], path: &str) -> io::Result<()> {
    let flags = args.join(" ");

    let content = format!(
        r#"[Unit]
Description={0} service
After=network.target

[Service]
Type=simple
User=root
ExecStart=/usr/local/bin/{0} {1}
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
"#,
        program, flags
    );

    fs::write(path, content).await?;
    Ok(())
}

/// Restarts the agent through systemd if it is registered as a supervised
/// unit (i.e. `wallguard-cli start` has run at least once), returning
/// `Ok(false)` if no unit is registered so the caller can fall back to a
/// bare spawn.
///
/// This exists to avoid racing systemd's own `Restart=always` timer: after a
/// graceful shutdown, systemd restarts the unit `RestartSec` later
/// regardless of exit code, so a separate unsupervised spawn done at the
/// same time can end up fighting it for the single-instance lock. Asking
/// systemd itself to restart makes it the only actor doing so.
pub async fn restart_via_service_manager(program: &str) -> io::Result<bool> {
    let service_name = format!("{program}.service");
    let service_path = format!("{SYSTEMD_DIR}/{service_name}");

    if !Path::new(&service_path).exists() {
        return Ok(false);
    }

    run_systemctl(&["restart", &service_name]).await?;
    Ok(true)
}

async fn run_systemctl(args: &[&str]) -> io::Result<()> {
    let output = Command::new("systemctl").args(args).output().await?;

    if !output.status.success() {
        return Err(io::Error::other(format!(
            "systemctl {:?} failed:\n{}",
            args,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}
