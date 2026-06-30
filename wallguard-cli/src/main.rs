use anyhow::{Ok, Result as AnyResult};
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
            if is_agent_running() {
                println!("Agent is already running");
                return Ok(());
            }

            const DEFAULT_SERVER_URL: &str = "localhost:50051";

            let control_channel_url = control_channel_url.unwrap_or(DEFAULT_SERVER_URL.into());
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
            use std::ffi::OsStr;
            use sysinfo::{ProcessesToUpdate, System};

            if !is_agent_running() {
                eprintln!("WallGuard agent is not running.");
                std::process::exit(-1);
            }

            if autostart::disable_service("wallguard").await.is_err() {
                eprintln!("WARNING: Error occured while trying to unregister wallguard service");
            }

            let mut system = System::new();
            system.refresh_processes(ProcessesToUpdate::All, true);

            // sysinfo includes the .exe extension in process names on Windows.
            #[cfg(windows)]
            let target_name = OsStr::new("wallguard.exe");
            #[cfg(not(windows))]
            let target_name = OsStr::new("wallguard");

            for process in system.processes().values() {
                if process.name() == target_name {
                    // process.kill() is cross-platform; kill_with(Signal::Kill)
                    // returns None on Windows because POSIX signals are unsupported.
                    if !process.kill() {
                        eprintln!("Failed to terminate WallGuard agent.");
                        std::process::exit(-1);
                    }
                    println!("WallGuard agent stopped successfully.");
                    break;
                }
            }
        }
    }

    Ok(())
}
