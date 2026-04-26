use std::time::Duration;

use anyhow::Context;

use crate::cmd::proto::cli::{GracefulRestartRequest, StatusRequest};

const DRAIN_TIMEOUT_MS: u32     = 10_000;
const POLL_INTERVAL:    Duration = Duration::from_millis(500);
const MAX_WAIT:         Duration = Duration::from_secs(12);

pub async fn run() -> anyhow::Result<()> {
    let mut client = super::status::connect().await?;

    let resp = client
        .graceful_restart(GracefulRestartRequest {
            drain_timeout_ms: DRAIN_TIMEOUT_MS,
        })
        .await
        .context("GracefulRestart RPC failed")?
        .into_inner();

    if !resp.accepted {
        anyhow::bail!("agent rejected restart: {}", resp.message);
    }

    println!("Graceful restart initiated ({}). Waiting for agent to exit…", resp.message);

    // Poll until the socket becomes unreachable (agent exited) or we time out.
    let deadline = tokio::time::Instant::now() + MAX_WAIT;
    loop {
        tokio::time::sleep(POLL_INTERVAL).await;
        if tokio::time::Instant::now() >= deadline {
            anyhow::bail!("agent did not exit within {}s", MAX_WAIT.as_secs());
        }
        match super::status::connect().await {
            Err(_) => {
                println!("Agent has exited. Ready for upgrade.");
                return Ok(());
            }
            Ok(mut c) => {
                if c.status(StatusRequest {}).await.is_err() {
                    println!("Agent has exited. Ready for upgrade.");
                    return Ok(());
                }
            }
        }
    }
}
