use std::io;
use tokio::process::Command;

pub async fn enable_service(program: &str, args: &[&str]) -> io::Result<()> {
    let flags = args.join(" ");

    run_sysrc(&format!("{}_enable=YES", program)).await?;

    if !flags.is_empty() {
        run_sysrc(&format!("{}_flags={}", program, flags)).await?;
    }

    Ok(())
}

pub async fn disable_service(program: &str) -> io::Result<()> {
    run_sysrc(&format!("{}_enable=NO", program)).await?;
    run_sysrc(&format!("-x {}_flags", program)).await?;

    Ok(())
}

async fn run_sysrc(arg: &str) -> io::Result<()> {
    let output = Command::new("/usr/sbin/sysrc").arg(arg).output().await?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "sysrc failed: {}\nstderr: {}",
                arg,
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }

    Ok(())
}
