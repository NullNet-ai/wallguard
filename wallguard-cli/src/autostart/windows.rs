use std::io;
use tokio::process::Command;

/// The Task Scheduler task name used for the WallGuard agent.
const TASK_NAME: &str = "WallGuard";

/// Registers `program` to run at boot via Task Scheduler.
///
/// wallguard.exe is a plain console app — it never calls
/// `StartServiceCtrlDispatcher` — so it cannot be registered as a real SCM
/// service (`sc create` would look right but SCM kills it ~30s after boot
/// with error 1053 once it fails to respond as a service). Task Scheduler
/// runs arbitrary executables as SYSTEM on a boot trigger, with no SCM
/// dispatcher requirement, and can restart it on failure — the Windows
/// equivalent of systemd's `Restart=always` / launchd's `KeepAlive`.
///
/// The binary is located by looking in the same directory as the current
/// executable (wallguard-cli.exe and wallguard.exe are installed side-by-side
/// by the MSI into `C:\Program Files\WallGuard\`).
pub async fn enable_service(program: &str, args: &[&str]) -> io::Result<()> {
    // Resolve the binary path relative to this executable's directory.
    let exe_dir = std::env::current_exe()
        .map_err(io::Error::other)?
        .parent()
        .ok_or_else(|| io::Error::other("could not determine executable directory"))?
        .to_path_buf();

    let bin_path = exe_dir.join(format!("{}.exe", program));
    let arguments = args.join(" ");

    let xml_path = std::env::temp_dir().join(format!("{program}-task.xml"));
    let xml = task_xml(&bin_path.display().to_string(), &arguments);
    tokio::fs::write(&xml_path, xml).await?;

    let create = Command::new("schtasks")
        .args(["/Create", "/TN", TASK_NAME, "/XML"])
        .arg(&xml_path)
        .arg("/F")
        .output()
        .await?;

    let _ = tokio::fs::remove_file(&xml_path).await;

    if !create.status.success() {
        return Err(io::Error::other(format!(
            "schtasks /Create failed:\n{}",
            String::from_utf8_lossy(&create.stderr)
        )));
    }

    // The boot trigger only fires on the *next* boot; run it now so the
    // agent is supervised by Task Scheduler immediately instead of the
    // caller falling back to a bare, unsupervised spawn.
    let _ = Command::new("schtasks")
        .args(["/Run", "/TN", TASK_NAME])
        .output()
        .await;

    Ok(())
}

/// Stops and removes the scheduled task registered by [`enable_service`].
pub async fn disable_service(_program: &str) -> io::Result<()> {
    // Best-effort stop — ignore errors if the task is not currently running.
    let _ = Command::new("schtasks")
        .args(["/End", "/TN", TASK_NAME])
        .output()
        .await;

    let output = Command::new("schtasks")
        .args(["/Delete", "/TN", TASK_NAME, "/F"])
        .output()
        .await?;

    if !output.status.success() {
        return Err(io::Error::other(format!(
            "schtasks /Delete failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Always returns `Ok(false)`: unlike systemd/launchd, Task Scheduler's
/// `RestartOnFailure` only fires on a nonzero exit code, and a graceful
/// shutdown exits 0 — so there's no restart-on-exit race to avoid here.
/// Kept only so callers can share one code path across platforms, always
/// falling back to a bare spawn on Windows.
pub async fn restart_via_service_manager(_program: &str) -> io::Result<bool> {
    Ok(false)
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Builds a Task Scheduler 2.0 task definition that starts `command` at
/// boot as SYSTEM, with highest privileges, and restarts it on failure.
fn task_xml(command: &str, arguments: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <Triggers>
    <BootTrigger>
      <Enabled>true</Enabled>
    </BootTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <UserId>S-1-5-18</UserId>
      <LogonType>ServiceAccount</LogonType>
      <RunLevel>HighestAvailable</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <StartWhenAvailable>true</StartWhenAvailable>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <RestartOnFailure>
      <Interval>PT1M</Interval>
      <Count>999</Count>
    </RestartOnFailure>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>{command}</Command>
      <Arguments>{arguments}</Arguments>
    </Exec>
  </Actions>
</Task>
"#,
        command = xml_escape(command),
        arguments = xml_escape(arguments),
    )
}
