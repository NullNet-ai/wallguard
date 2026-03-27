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

    Ok(())
}

pub async fn disable_service(program: &str) -> io::Result<()> {
    remove_rcd_script(program).await?;
    Ok(())
}
