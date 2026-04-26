use anyhow::Context;
use hyper_util::rt::TokioIo;
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;

use crate::cmd::proto::cli::{agent_control_client::AgentControlClient, StatusRequest};

const SOCK: &str = "/run/wallguard/agent.sock";

pub async fn run() -> anyhow::Result<()> {
    let mut client = connect().await?;

    let resp = client
        .status(StatusRequest {})
        .await
        .context("Status RPC failed")?
        .into_inner();

    let state_str = match resp.state {
        0 => "provisioning",
        1 => "idle",
        2 => "connecting",
        3 => "connected",
        n => return Err(anyhow::anyhow!("unknown state {n}")),
    };

    let hours   = resp.uptime_secs / 3600;
    let minutes = (resp.uptime_secs % 3600) / 60;
    let secs    = resp.uptime_secs % 60;

    println!("WallGuard Agent");
    println!("  State   : {state_str}");
    println!("  Version : {}", resp.agent_version);
    println!("  Uptime  : {hours:02}:{minutes:02}:{secs:02}");

    if !resp.device_id.is_empty() {
        println!("  Device  : {}", resp.device_id);
    }
    if !resp.server_url.is_empty() {
        println!("  Server  : {}", resp.server_url);
    }

    Ok(())
}

pub(crate) async fn connect() -> anyhow::Result<AgentControlClient<tonic::transport::Channel>> {
    // Tonic requires an HTTP URI for the endpoint even for Unix sockets.
    let channel = Endpoint::try_from("http://[::]:0")?
        .connect_with_connector(service_fn(|_: Uri| async {
            let stream = tokio::net::UnixStream::connect(SOCK).await?;
            Ok::<_, std::io::Error>(TokioIo::new(stream))
        }))
        .await
        .with_context(|| format!("could not connect to agent socket {SOCK}"))?;

    Ok(AgentControlClient::new(channel))
}
