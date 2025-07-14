use anyhow::Result as AnyResult;
use arguments::Arguments;
use clap::Parser;
use std::time::Duration;
use tonic::transport::{Channel, Error};
use wallguard_common::protobuf::wallguard_cli::{
    status::State, wallguard_cli_client::WallguardCliClient, JoinOrgReq,
};

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
            let response = client.get_capabilities(()).await?.into_inner();

            println!("WallGuard Capabilities:");
            println!("  Traffic  : {}", response.traffic);
            println!("  SysConf  : {}", response.sysconfig);
            println!("  Telemetry: {}", response.telemetry);
        }
        arguments::Command::Join { installation_code } => {
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
            let response = client.leave_org(()).await?.into_inner();

            match response.success {
                true => println!("Successfully left the current organization."),
                false => eprintln!("Failed to leave organization: {}", response.message),
            }
        }
    }

    Ok(())
}
