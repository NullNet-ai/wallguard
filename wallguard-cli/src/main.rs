use anyhow::Result as AnyResult;
use arguments::Arguments;
use clap::Parser;
use std::time::Duration;
use tonic::transport::{Channel, Error};
use wallguard_cli::{status::State, wallguard_cli_client::WallguardCliClient, Empty, JoinOrgReq};

#[rustfmt::skip]
mod wallguard_cli;
mod arguments;

type Client = WallguardCliClient<Channel>;

async fn cli_connect() -> Result<Client, Error> {
    const EXPECTED_ADDR: &str = "http://127.0.0.1:54056";
    const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

    let channel = Channel::from_shared(EXPECTED_ADDR)
        .unwrap()
        .timeout(DEFAULT_TIMEOUT)
        .connect()
        .await?;

    Ok(WallguardCliClient::new(channel))
}

#[tokio::main]
pub async fn main() -> AnyResult<()> {
    let arguments = Arguments::parse();

    let Ok(mut client) = cli_connect().await else {
        eprintln!(
            "Unable to connect to the WallGuard service. Make sure the service is running..."
        );
        std::process::exit(-1);
    };

    match arguments.command {
        arguments::Command::Status => {
            let response = client.get_status(Empty {}).await?.into_inner();

            if response.state.is_none() {
                eprintln!("wallguard returned empty status");
                std::process::exit(-1);
            }

            println!("WallGuard State:");

            match response.state.unwrap() {
                State::Idle(idle) => {
                    println!("  STATE    : IDLE");
                    println!("  Message  : {}", idle.message);
                }
                State::Connected(connected) => {
                    println!("  STATE    : CONNECTED");
                    println!("  Org ID   : {}", connected.org_id);
                    println!("  Traffic  : {}", connected.traffic);
                    println!("  SysConf  : {}", connected.sysconfig);
                    println!("  Telemetry: {}", connected.telemetry);
                }
                State::Error(error) => {
                    println!("  STATE    : ERROR");
                    println!("  Message  : {}", error.message);
                }
            }
        }
        arguments::Command::Capabilities => {
            let response = client.get_capabilities(Empty {}).await?.into_inner();

            println!("WallGuard Capabilities:");
            println!("  Traffic  : {}", response.traffic);
            println!("  SysConf  : {}", response.sysconfig);
            println!("  Telemetry: {}", response.telemetry);
        }
        arguments::Command::Join { org_id } => {
            let response = client.join_org(JoinOrgReq { org_id }).await?.into_inner();

            match response.success {
                true => println!("Successfully joined organization."),
                false => eprintln!("Failed to join organization: {}", response.message),
            }
        }
        arguments::Command::Leave => {
            let response = client.leave_org(Empty::default()).await?.into_inner();

            match response.success {
                true => println!("Successfully left the current organization."),
                false => eprintln!("Failed to leave organization: {}", response.message),
            }
        }
    }

    Ok(())
}
