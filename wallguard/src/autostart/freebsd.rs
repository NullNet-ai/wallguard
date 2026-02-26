use std::io;
use std::path::Path;
use tokio::{fs, process::Command};

pub async fn create_rcd_script(program: &str) -> io::Result<()> {
    let script_path = format!("/usr/local/etc/rc.d/{}", program);
    if Path::new(&script_path).exists() {
        println!("rc.d script already exists: {}", script_path);
        return Ok(());
    }

    let content = format!(
        r#"#!/bin/sh
# PROVIDE: {0}
# REQUIRE: DAEMON
# KEYWORD: shutdown

. /etc/rc.subr

name="{0}"
rcvar="${{name}}_enable"

command="/usr/local/bin/{0}"
command_args="${{{0}_flags}}"

load_rc_config $name
run_rc_command "$1"
"#,
        program
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

pub async fn enable_service(program: &str, args: &[&str]) -> io::Result<()> {
    create_rcd_script(program).await?;

    let flags = args.join(" ");
    run_sysrc(&format!("{}_enable=YES", program)).await?;

    if !flags.is_empty() {
        run_sysrc(&format!("{}_flags={}", program, flags)).await?;
    }

    Ok(())
}

pub async fn disable_service(program: &str) -> io::Result<()> {
    run_sysrc(&format!("{}_enable=NO", program)).await?;
    run_sysrc(&format!("{}_flags", program)).await?;

    Ok(())
}

async fn run_sysrc(arg: &str) -> io::Result<()> {
    let output = Command::new("sudo")
        .arg("/usr/sbin/sysrc")
        .arg(arg)
        .output()
        .await?;

    if !output.status.success() {
        return Err(io::Error::other(format!(
            "sysrc failed for '{}', stderr: {}",
            arg,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}
