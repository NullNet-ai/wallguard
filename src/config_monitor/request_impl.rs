use libwallguard::{
    Authentication, ConfigSnapshot, ConfigStatus, FileSnapshot, WallGuardGrpcInterface,
};
use nullnet_libconfmon::{Snapshot, State};

pub async fn request_impl(
    addr: &str,
    port: u16,
    snapshot: Snapshot,
    token: String,
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
        auth: Some(Authentication { token }),
        status: state_to_status(state).into(),
    };

    match client.handle_config(data).await {
        Ok(response) => {
            if response.success {
                Ok(())
            } else {
                Err(response.message)
            }
        }
        Err(err) => Err(err),
    }
}

fn state_to_status(state: State) -> ConfigStatus {
    match state {
        State::Draft => ConfigStatus::CsDraft,
        State::Applied => ConfigStatus::CsApplied,
        State::Undefined => ConfigStatus::CsUndefined,
    }
}
