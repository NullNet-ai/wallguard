use anyhow::Result as AnyResult;
use arguments::Arguments;
use clap::Parser;
use std::process::{Command, Stdio};
use std::time::Duration;
use tonic::transport::Channel;
use wallguard_common::protobuf::wallguard_cli::{
    JoinOrgReq, status::State, wallguard_cli_client::WallguardCliClient,
};

mod arguments;
mod autostart;
mod config;
mod update;

type Client = WallguardCliClient<Channel>;

fn is_agent_running() -> bool {
    use std::ffi::OsStr;
    use sysinfo::{ProcessesToUpdate, System};

    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);

    // sysinfo includes the .exe extension in process names on Windows.
    #[cfg(windows)]
    let target_name = OsStr::new("wallguard.exe");
    #[cfg(not(windows))]
    let target_name = OsStr::new("wallguard");

    system
        .processes()
        .values()
        .any(|proc| proc.name() == target_name)
}

/// Polls the single-instance lock up to `attempts` times, `delay` apart,
/// returning `true` as soon as it's found held by another process.
async fn agent_lock_held_within(
    lock_path: &std::path::Path,
    attempts: u32,
    delay: Duration,
) -> bool {
    use std::result::Result::Ok;

    for _ in 0..attempts {
        match wallguard_common::single_instance::InstanceLock::try_acquire(lock_path) {
            Ok(None) => return true,
            _ => tokio::time::sleep(delay).await,
        }
    }
    false
}

/// Polls the single-instance lock up to `attempts` times, `delay` apart,
/// returning `true` as soon as it's found free (i.e. the agent process has
/// actually exited, not just acknowledged a shutdown request).
pub(crate) async fn wait_for_lock_free(
    lock_path: &std::path::Path,
    attempts: u32,
    delay: Duration,
) -> bool {
    use std::result::Result::Ok;

    for _ in 0..attempts {
        match wallguard_common::single_instance::InstanceLock::try_acquire(lock_path) {
            Ok(Some(_lock)) => return true,
            _ => tokio::time::sleep(delay).await,
        }
    }
    false
}

/// Best-effort snapshot of the currently running agent's command-line
/// arguments (excluding argv[0], the executable path itself — callers pass
/// this straight to `Command::args()` after already setting the binary via
/// `Command::new()`, so including it would inject a stray leading argument
/// that the agent's clap parser would reject), so `update`/`restart` can
/// respawn it the same way it was running.
pub(crate) fn capture_agent_args() -> Option<Vec<String>> {
    use std::ffi::OsStr;
    use sysinfo::{ProcessesToUpdate, System};

    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);

    #[cfg(windows)]
    let target_name = OsStr::new("wallguard.exe");
    #[cfg(not(windows))]
    let target_name = OsStr::new("wallguard");

    system.processes().values().find_map(|proc| {
        if proc.name() == target_name {
            Some(
                proc.cmd()
                    .iter()
                    .skip(1)
                    .map(|s| s.to_string_lossy().into_owned())
                    .collect(),
            )
        } else {
            None
        }
    })
}

/// Pretty-prints a captured `--flag value` argv list (see
/// `capture_agent_args`). wallguard's own arguments are all `--long value`
/// pairs (no bare boolean flags), but a value is only paired up if it
/// doesn't itself look like a flag, so this degrades gracefully if that
/// ever changes.
fn print_agent_args(args: &[String]) {
    let mut i = 0;
    while i < args.len() {
        let flag = &args[i];
        match args.get(i + 1).filter(|v| !v.starts_with("--")) {
            Some(value) => {
                println!("  {flag:<22} : {value}");
                i += 2;
            }
            None => {
                println!("  {flag}");
                i += 1;
            }
        }
    }
}

/// Falls back to the last configuration passed to `start` when the agent's
/// live argv can't be read (e.g. it's not currently running). This may not
/// reflect the actual running process if it was started some other way.
fn print_cached_start_args() {
    let cached = config::StartConfig::load();

    if cached.control_channel_url.is_none() && cached.platform.is_none() {
        println!("  (unavailable)");
        return;
    }

    println!("  (last known `start` configuration — agent not running or its argv is unreadable)");
    if let Some(url) = &cached.control_channel_url {
        println!("  --control-channel-url : {url}");
    }
    if let Some(platform) = &cached.platform {
        println!("  --platform             : {platform}");
    }
    if let Some(batch_size) = cached.batch_size {
        println!("  --batch-size           : {batch_size}");
    }
}

/// Hard-kills the running agent process by name. This is the same
/// unconditional kill `Command::Stop` uses; it does not touch service
/// registration (see `autostart::disable_service` for that) and does not
/// attempt a graceful RPC-based shutdown first (see `update` for that).
pub(crate) fn hard_kill_agent() -> bool {
    use std::ffi::OsStr;
    use sysinfo::{ProcessesToUpdate, System};

    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All, true);

    #[cfg(windows)]
    let target_name = OsStr::new("wallguard.exe");
    #[cfg(not(windows))]
    let target_name = OsStr::new("wallguard");

    for process in system.processes().values() {
        if process.name() == target_name {
            return process.kill();
        }
    }
    true // nothing to kill counts as success
}

/// Same connection `cli_connect` uses, but returns `None` on failure instead
/// of exiting the process — used by `restart`, where an unreachable RPC
/// server should fall back to a hard kill rather than abort the command.
async fn try_connect() -> Option<Client> {
    const EXPECTED_ADDR: &str = "http://127.0.0.1:54056";

    let channel = Channel::from_shared(EXPECTED_ADDR)
        .ok()?
        .timeout(Duration::from_secs(5))
        .connect()
        .await
        .ok()?;

    Some(WallguardCliClient::new(channel))
}

async fn cli_connect() -> Client {
    use std::result::Result::Ok;

    const EXPECTED_ADDR: &str = "http://127.0.0.1:54056";
    const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

    let Ok(channel) = Channel::from_shared(EXPECTED_ADDR)
        .unwrap()
        .timeout(DEFAULT_TIMEOUT)
        .connect()
        .await
    else {
        eprintln!("Unable to connect to the WallGuard agent. Make sure the service is running...");
        std::process::exit(-1);
    };

    WallguardCliClient::new(channel)
}

fn check_privileges() {
    #[cfg(windows)]
    {
        if !is_elevated::is_elevated() {
            println!("This program must be run as Administrator. Exiting …");
            std::process::exit(-1);
        }
    }

    #[cfg(unix)]
    {
        if !nix::unistd::Uid::effective().is_root() {
            println!("This program must be run as root. Exiting ...");
            std::process::exit(-1);
        }
    }
}

#[tokio::main]
pub async fn main() -> AnyResult<()> {
    use std::result::Result::Ok;

    check_privileges();
    let arguments = Arguments::parse();

    match arguments.command {
        arguments::Command::Status => {
            let mut client = cli_connect().await;
            let response = client.get_status(()).await?.into_inner();

            if response.state.is_none() {
                eprintln!("wallguard returned empty status");
                std::process::exit(-1);
            }

            println!("WallGuard State:");

            match response.state.unwrap() {
                State::Idle(_) => {
                    println!("  STATE    : IDLE");
                }
                State::Connected(data) => {
                    println!("  STATE    : CONNECTED");
                    println!(
                        "  DEVICE ID: {}",
                        data.device_id.as_deref().unwrap_or("unknown")
                    );
                    println!(
                        "  UUID     : {}",
                        data.device_uuid.as_deref().unwrap_or("unknown")
                    );
                }
                State::Error(error) => {
                    println!("  STATE    : ERROR");
                    println!("  Message  : {}", error.message);
                }
                State::Connecting(_) => {
                    println!("  STATE    : CONNECTING");
                }
            }

            println!();
            println!("Arguments:");
            match capture_agent_args() {
                Some(args) if !args.is_empty() => print_agent_args(&args),
                _ => print_cached_start_args(),
            }
        }
        arguments::Command::Capabilities => {
            let mut client = cli_connect().await;
            let response = client.get_capabilities(()).await?.into_inner();

            println!("WallGuard Capabilities:");
            println!("  Traffic  : {}", response.traffic);
            println!("  SysConf  : {}", response.sysconfig);
            println!("  Telemetry: {}", response.telemetry);
        }
        arguments::Command::Join { installation_code } => {
            let mut client = cli_connect().await;
            let response = client
                .join_org(JoinOrgReq { installation_code })
                .await?
                .into_inner();

            match response.success {
                true => {
                    println!("Connecting to organization.");
                    println!("Run `wallguard-cli status` to check progress.");
                }
                false => eprintln!("Failed to join organization: {}", response.message),
            }
        }
        arguments::Command::Leave => {
            let mut client = cli_connect().await;
            let response = client.leave_org(()).await?.into_inner();

            match response.success {
                true => println!("Left organization successfully."),
                false => eprintln!("Failed to leave organization: {}", response.message),
            }
        }
        arguments::Command::Reconnect => {
            let mut client = cli_connect().await;
            let response = client.reconnect(()).await?.into_inner();

            match response.success {
                true => {
                    println!("Reconnecting.");
                    println!("Run `wallguard-cli status` to check progress.");
                }
                false => eprintln!("Failed to reconnect: {}", response.message),
            }
        }
        arguments::Command::Version => {
            let mut client = cli_connect().await;

            let response = client.get_version(()).await?.into_inner();

            println!("Version: {}", response.value);
        }
        arguments::Command::Start {
            control_channel_url,
            platform,
            batch_size,
        } => {
            let lock_path = wallguard_common::single_instance::agent_lock_path();

            // Consult the same lock the agent itself acquires on startup:
            // this is atomic (unlike a process-name scan) and catches an
            // agent started by any means (service manager, manual run, ...).
            match wallguard_common::single_instance::InstanceLock::try_acquire(&lock_path) {
                // Not currently running. `_lock` is scoped to this arm, so it
                // is dropped (and the flock released) right here — well
                // before we get to `enable_service`/spawning the agent below.
                Ok(Some(_lock)) => {}
                Ok(None) => {
                    println!("Agent is already running");
                    return Ok(());
                }
                Err(err) => {
                    eprintln!("Failed to check WallGuard agent state: {err}");
                    std::process::exit(-1);
                }
            }

            let mut cached = config::StartConfig::load();

            let Some(control_channel_url) =
                control_channel_url.or_else(|| cached.control_channel_url.clone())
            else {
                eprintln!(
                    "No server URL configured yet. Provide one with --control-channel-url on the first `start`."
                );
                std::process::exit(-1);
            };

            let platform = platform.unwrap_or_else(|| {
                cached
                    .platform
                    .clone()
                    .unwrap_or(arguments::Platform::Generic)
            });

            let batch_size = batch_size.or(cached.batch_size);

            // Persist the effective values (whatever was just passed, or
            // whatever was already cached) so the next `start` can omit them.
            cached.control_channel_url = Some(control_channel_url.clone());
            cached.platform = Some(platform.clone());
            cached.batch_size = batch_size;
            if let Err(err) = cached.save() {
                eprintln!("WARNING: Failed to persist start configuration: {err}");
            }

            let batch_size_str = batch_size.map(|n| n.to_string());
            let platform_str = platform.to_string();

            let mut service_args = vec![
                "--control-channel-url",
                control_channel_url.as_str(),
                "--platform",
                platform_str.as_str(),
            ];
            if let Some(ref s) = batch_size_str {
                service_args.push("--batch-size");
                service_args.push(s.as_str());
            }

            if autostart::enable_service("wallguard", &service_args)
                .await
                .is_err()
            {
                eprintln!("WARNING: Failed to register wallguard as a service");
            }

            // `enable_service` also starts the agent immediately on every
            // platform (systemd `enable --now`, FreeBSD `service start`,
            // macOS launchd with RunAtLoad, Windows `schtasks /Run`). Give
            // it a moment to acquire the single-instance lock before
            // falling back to spawning it ourselves — otherwise every
            // `start` would launch a second, doomed copy that immediately
            // loses the lock race.
            if agent_lock_held_within(&lock_path, 5, Duration::from_millis(100)).await {
                println!("WallGuard agent started successfully.");
                println!("Logs are written to /var/log/wallguard.log.");
                println!("Check its status with `wallguard-cli status`.");
                return Ok(());
            }

            let mut cmd = Command::new("wallguard");
            cmd.arg("--control-channel-url")
                .arg(&control_channel_url)
                .arg("--platform")
                .arg(platform.to_string());
            if let Some(n) = batch_size {
                cmd.arg("--batch-size").arg(n.to_string());
            }
            cmd.stdout(Stdio::null()).stderr(Stdio::null());

            if let Err(err) = cmd.spawn() {
                eprintln!("Failed to spawn WallGuard agent: {err}");
                eprintln!(
                    "Make sure the 'wallguard' binary is installed at /usr/local/bin/wallguard"
                );
                std::process::exit(-1);
            } else {
                println!("WallGuard agent started successfully.");
                println!("Logs are written to /var/log/wallguard.log.");
                println!("Check its status with `wallguard-cli status`.");
            }
        }
        arguments::Command::Stop => {
            if !is_agent_running() {
                eprintln!("WallGuard agent is not running.");
                std::process::exit(-1);
            }

            if autostart::disable_service("wallguard").await.is_err() {
                eprintln!("WARNING: Error occured while trying to unregister wallguard service");
            }

            if !hard_kill_agent() {
                eprintln!("Failed to terminate WallGuard agent.");
                std::process::exit(-1);
            }
            println!("WallGuard agent stopped successfully.");
        }
        arguments::Command::Restart => {
            if !is_agent_running() {
                eprintln!("WallGuard agent is not running. Use `wallguard-cli start` instead.");
                std::process::exit(-1);
            }

            let captured_args = capture_agent_args();
            let lock_path = wallguard_common::single_instance::agent_lock_path();

            println!("Stopping WallGuard agent...");
            if let Some(mut client) = try_connect().await {
                let _ = client.shutdown(()).await;
            }

            if !wait_for_lock_free(&lock_path, 20, Duration::from_millis(500)).await {
                println!("Agent did not shut down gracefully in time, forcing termination...");
                if !hard_kill_agent() {
                    eprintln!("Failed to terminate WallGuard agent.");
                    std::process::exit(-1);
                }
            }

            println!("Starting WallGuard agent...");
            let mut cmd = Command::new("wallguard");
            cmd.args(captured_args.as_deref().unwrap_or(&[]))
                .stdout(Stdio::null())
                .stderr(Stdio::null());

            if let Err(err) = cmd.spawn() {
                eprintln!("Failed to spawn WallGuard agent: {err}");
                eprintln!(
                    "Make sure the 'wallguard' binary is installed at /usr/local/bin/wallguard"
                );
                std::process::exit(-1);
            }

            if agent_lock_held_within(&lock_path, 20, Duration::from_millis(500)).await {
                println!("WallGuard agent restarted successfully.");
            } else {
                eprintln!("WallGuard agent did not come back up. Check /var/log/wallguard.log.");
                std::process::exit(-1);
            }
        }
        arguments::Command::Update { check } => {
            if let Err(err) = update::run(check).await {
                eprintln!("Update failed: {err:#}");
                std::process::exit(-1);
            }
        }
    }

    Ok(())
}
