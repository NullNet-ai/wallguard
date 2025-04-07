use nullnet_libconfmon::{Snapshot, State};
use nullnet_libwallguard::{ConfigSnapshot, ConfigStatus, FileSnapshot, WallGuardGrpcInterface};
use std::sync::Arc;
use tokio::sync::RwLock;

pub async fn request_impl(
    addr: &str,
    port: u16,
    snapshot: Snapshot,
    token: Arc<RwLock<String>>,
    state: State,
) -> Result<(), String> {
    let mut client = WallGuardGrpcInterface::new(addr, port).await;

    let data = ConfigSnapshot {
        files: snapshot
            .iter()
            .map(|fs| FileSnapshot {
                filename: fs.filename.clone(),
                contents: fs.content.clone(),
            })
            .collect(),
        token: token.read().await.clone(),
        status: state_to_status(&state).into(),
    };

    client.handle_config(data).await.map(|_| ())
}

fn state_to_status(state: &State) -> ConfigStatus {
    match state {
        State::Draft => ConfigStatus::CsDraft,
        State::Applied => ConfigStatus::CsApplied,
        State::Undefined => ConfigStatus::CsUndefined,
    }
}
