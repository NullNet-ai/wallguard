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

type Client = WallguardCliClient<Channel>;

fn is_agent_running() -> bool {
    use std::ffi::OsStr;
    use sysinfo::{ProcessesToUpdate, System};

    let mut system = System::new_all();

    system.refresh_processes(ProcessesToUpdate::All, true);

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

#[tokio::main]
pub async fn main() -> AnyResult<()> {
    let arguments = Arguments::parse();

    if !nix::unistd::Uid::effective().is_root() {
        println!("This program must be run as root. Exiting ...");
        std::process::exit(-1);
    }

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
                State::Connected(_) => {
                    println!("  STATE    : CONNECTED");
                }
                State::Error(error) => {
                    println!("  STATE    : ERROR");
                    println!("  Message  : {}", error.message);
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
                true => println!("Successfully joined organization."),
                false => eprintln!("Failed to join organization: {}", response.message),
            }
        }
        arguments::Command::Leave => {
            let mut client = cli_connect().await;
            let response = client.leave_org(()).await?.into_inner();

            match response.success {
                true => println!("Successfully left the current organization."),
                false => eprintln!("Failed to leave organization: {}", response.message),
            }
        }

        arguments::Command::Start {
            control_channel_host,
            control_channel_port,
            platform,
        } => {
            if is_agent_running() {
                println!("Agent is already running");
                return Ok(());
            }

            const DEFAULT_SERVER_HOST: &str = "127.0.0.1";

            let control_channel_host = control_channel_host.unwrap_or(DEFAULT_SERVER_HOST.into());
            let control_channel_port = control_channel_port.unwrap_or(50051);

            if Command::new("wallguard")
                .arg("--control-channel-host")
                .arg(&control_channel_host)
                .arg("--control-channel-port")
                .arg(control_channel_port.to_string())
                .arg("--platform")
                .arg(platform.to_string())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .is_err()
            {
                eprintln!("Failed to spawn WallGuard agent.");
                std::process::exit(-1);
            } else {
                println!("Successfully spawned WallGuard agent.");
            }
        }
        arguments::Command::Stop => {
            use std::ffi::OsStr;
            use sysinfo::{ProcessesToUpdate, Signal, System};

            if !is_agent_running() {
                eprintln!("WallGuard agent is not running.");
                std::process::exit(-1);
            }

            let mut system = System::new();
            system.refresh_processes(ProcessesToUpdate::All, true);

            let target_name = OsStr::new("wallguard");

            for process in system.processes().values() {
                if process.name() == target_name {
                    if process.kill_with(Signal::Kill).is_none() {
                        eprintln!("Failed to send SIGKILL to WallGuard agent");
                        std::process::exit(-1);
                    } else {
                        println!("Successfully stopped WallGuard agent");
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
