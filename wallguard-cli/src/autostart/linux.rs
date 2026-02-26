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
    run_systemctl(&["enable", &service_name]).await?;

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
