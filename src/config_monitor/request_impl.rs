use libwallguard::{Authentication, ConfigSnapshot, FileSnapshot, WallGuardGrpcInterface};
use nullnet_libconfmon::Snapshot;

pub async fn request_impl(
    addr: &str,
    port: u16,
    snapshot: Snapshot,
    token: String,
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
