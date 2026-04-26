use std::time::Duration;

use anyhow::Context;
use clap::Args;

use crate::cmd::proto::cli::StatusRequest;

#[derive(Args, Debug)]
pub struct AutostartArgs {
    /// enable or disable autostart
    #[arg(value_enum)]
    pub action: AutostartAction,
}

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
pub enum AutostartAction {
    Enable,
    Disable,
}

pub async fn run(args: AutostartArgs) -> anyhow::Result<()> {
    #[cfg(target_os = "linux")]
    {
        match args.action {
            AutostartAction::Enable  => linux_enable().await,
            AutostartAction::Disable => linux_disable(),
        }
    }
    #[cfg(target_os = "freebsd")]
    {
        match args.action {
            AutostartAction::Enable  => freebsd_enable(),
            AutostartAction::Disable => freebsd_disable(),
        }
    }
    #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
    {
        let _ = args;
        anyhow::bail!("autostart is only supported on Linux and FreeBSD");
    }
}

// ---------------------------------------------------------------------------
// Linux — systemd
// ---------------------------------------------------------------------------

const SYSTEMD_UNIT_PATH: &str = "/etc/systemd/system/wallguard-agent.service";
const SYSTEMD_UNIT: &str = r#"[Unit]
Description=WallGuard Agent
After=network-online.target
Wants=network-online.target

[Service]
ExecStart=/usr/sbin/wg-agent --config /etc/wallguard/config.toml
Restart=on-failure
RestartSec=5
RuntimeDirectory=wallguard
RuntimeDirectoryMode=0700
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
"#;

#[cfg(target_os = "linux")]
async fn linux_enable() -> anyhow::Result<()> {
    std::fs::write(SYSTEMD_UNIT_PATH, SYSTEMD_UNIT)
        .with_context(|| format!("write {SYSTEMD_UNIT_PATH}"))?;

    systemctl(&["daemon-reload"])?;
    systemctl(&["enable", "--now", "wallguard-agent.service"])?;

    println!("wallguard-agent enabled and started.");
    println!("Waiting for agent to become ready…");
    wait_for_agent().await
}

#[cfg(target_os = "linux")]
fn linux_disable() -> anyhow::Result<()> {
    systemctl(&["disable", "--now", "wallguard-agent.service"])?;
    println!("wallguard-agent stopped and disabled.");
    Ok(())
}

#[cfg(target_os = "linux")]
fn systemctl(args: &[&str]) -> anyhow::Result<()> {
    let status = std::process::Command::new("systemctl")
        .args(args)
        .status()
        .context("systemctl not found")?;
    if !status.success() {
        anyhow::bail!("systemctl {:?} failed (exit {:?})", args, status.code());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// FreeBSD — rc.d
// ---------------------------------------------------------------------------

#[cfg(target_os = "freebsd")]
const RCD_SCRIPT_PATH: &str = "/usr/local/etc/rc.d/wallguard_agent";
#[cfg(target_os = "freebsd")]
const RCD_SCRIPT: &str = r#"#!/bin/sh
# PROVIDE: wallguard_agent
# REQUIRE: NETWORKING
# KEYWORD: shutdown

. /etc/rc.subr

name="wallguard_agent"
rcvar="wallguard_agent_enable"
command="/usr/local/sbin/wg-agent"
command_args="--config /usr/local/etc/wallguard/config.toml"
pidfile="/var/run/wallguard_agent.pid"
wallguard_agent_enable="${wallguard_agent_enable:-NO}"

load_rc_config $name
run_rc_command "$1"
"#;

#[cfg(target_os = "freebsd")]
const RC_CONF_PATH: &str = "/etc/rc.conf";

#[cfg(target_os = "freebsd")]
fn freebsd_enable() -> anyhow::Result<()> {
    std::fs::write(RCD_SCRIPT_PATH, RCD_SCRIPT)
        .with_context(|| format!("write {RCD_SCRIPT_PATH}"))?;
    std::process::Command::new("chmod")
        .args(["0755", RCD_SCRIPT_PATH])
        .status()
        .context("chmod failed")?;

    // Append enable flag to rc.conf if not already present.
    let rc_conf = std::fs::read_to_string(RC_CONF_PATH).unwrap_or_default();
    if !rc_conf.contains("wallguard_agent_enable") {
        use std::io::Write as _;
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(RC_CONF_PATH)
            .context("open /etc/rc.conf")?;
        writeln!(f, "\nwallguard_agent_enable=\"YES\"")?;
    }

    service_rc("start")?;
    println!("wallguard_agent enabled and started.");
    Ok(())
}

#[cfg(target_os = "freebsd")]
fn freebsd_disable() -> anyhow::Result<()> {
    let _ = service_rc("stop");

    let rc_conf = std::fs::read_to_string(RC_CONF_PATH).unwrap_or_default();
    let updated = rc_conf
        .lines()
        .filter(|l| !l.contains("wallguard_agent_enable"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(RC_CONF_PATH, updated).context("write /etc/rc.conf")?;

    println!("wallguard_agent stopped and disabled.");
    Ok(())
}

#[cfg(target_os = "freebsd")]
fn service_rc(action: &str) -> anyhow::Result<()> {
    let status = std::process::Command::new("service")
        .args(["wallguard_agent", action])
        .status()
        .context("service command failed")?;
    if !status.success() {
        anyhow::bail!("service wallguard_agent {action} failed");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Shared: poll until agent socket is alive (used after enable on Linux).
// ---------------------------------------------------------------------------

async fn wait_for_agent() -> anyhow::Result<()> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        if tokio::time::Instant::now() >= deadline {
            anyhow::bail!("agent did not become ready within 10s");
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
        if let Ok(mut c) = super::status::connect().await {
            if c.status(StatusRequest {}).await.is_ok() {
                println!("Agent is ready.");
                return Ok(());
            }
        }
    }
}
