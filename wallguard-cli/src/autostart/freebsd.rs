use std::io;
use std::path::PathBuf;
use tokio::{fs, process::Command};

pub async fn create_rcd_script(program: &str, args: &str) -> io::Result<()> {
    let script_path = format!("/usr/local/etc/rc.d/{}.sh", program);

    let content = format!(
        r#"#!/bin/sh
# PROVIDE: {0}
# REQUIRE: NETWORKING
# KEYWORD: shutdown

. /etc/rc.subr

name="{0}"
rcvar="${{name}}_enable"

command="/usr/local/bin/{0}"
: ${{{0}_enable:="YES"}}

command_args="{1} &"

load_rc_config $name
run_rc_command "$1"
"#,
        program, args
    );

    fs::write(&script_path, content).await?;

    Command::new("chmod")
        .arg("+x")
        .arg(&script_path)
        .output()
        .await?;

    println!("Created rc.d script at {}", script_path);

    Ok(())
}

pub async fn remove_rcd_script(program: &str) -> io::Result<()> {
    let script_filename = format!("{}.sh", program);
    let script_path = PathBuf::from("/usr/local/etc/rc.d").join(script_filename);

    if script_path.exists() {
        fs::remove_file(&script_path).await?;
    }

    Ok(())
}

pub async fn enable_service(program: &str, args: &[&str]) -> io::Result<()> {
    let flags = args.join(" ");
    create_rcd_script(program, &flags).await?;

    // Explicitly persist `<program>_enable="YES"` to /etc/rc.conf (rather
    // than relying on the rc.d script's own default), then start it now so
    // the agent is supervised by rc.d immediately instead of running as a
    // bare orphan process until the next reboot.
    run_service(program, "enable").await?;
    run_service(program, "start").await?;

    Ok(())
}

pub async fn disable_service(program: &str) -> io::Result<()> {
    remove_rcd_script(program).await?;
    Ok(())
}

/// Restarts the agent through rc.d if it is registered as a supervised
/// service, returning `Ok(false)` if no rc.d script is registered so the
/// caller can fall back to a bare spawn.
///
/// FreeBSD's rc.d has no restart-on-exit supervision (unlike systemd's
/// `Restart=always` or launchd's `KeepAlive`), so there's no race to avoid
/// here — this exists so the update/restart flow can go through `service`
/// rather than bypass it, for consistency with the other platforms.
pub async fn restart_via_service_manager(program: &str) -> io::Result<bool> {
    let script_path = PathBuf::from("/usr/local/etc/rc.d").join(format!("{program}.sh"));

    if !script_path.exists() {
        return Ok(false);
    }

    run_service(program, "restart").await?;
    Ok(true)
}

async fn run_service(program: &str, action: &str) -> io::Result<()> {
    let output = Command::new("service")
        .args([program, action])
        .output()
        .await?;

    if !output.status.success() {
        return Err(io::Error::other(format!(
            "service {program} {action} failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}
