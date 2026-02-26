use std::io;
use std::path::Path;
use tokio::{fs, process::Command};

pub async fn create_rcd_script(program: &str, args: &str) -> io::Result<()> {
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
command_user="root"
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

pub async fn enable_service(program: &str, args: &[&str]) -> io::Result<()> {
    let flags = args.join(" ");
    create_rcd_script(program, &flags).await?;
    run_sysrc(&format!("{}_enable=YES", program)).await?;

    Ok(())
}

pub async fn disable_service(program: &str) -> io::Result<()> {
    run_sysrc(&format!("{}_enable=NO", program)).await?;

    Ok(())
}

async fn run_sysrc(arg: &str) -> io::Result<()> {
    let output = Command::new("/usr/sbin/sysrc").arg(arg).output().await?;

    if !output.status.success() {
        return Err(io::Error::other(format!(
            "sysrc failed for '{}', stderr: {}",
            arg,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}
