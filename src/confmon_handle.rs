use std::future::Future;
use std::pin::Pin;

use libconfmon::{Snapshot, Watcher};
use wallguard_server::{Authentication, ConfigSnapshot, FileSnapshot, WallGuardGrpcInterface};

static POLL_INTERVAL: u64 = 500;

async fn send_configuration_snapshot(
    addr: String,
    port: u16,
    snapshot: Snapshot,
) {
    let mut client = WallGuardGrpcInterface::new(&addr, port).await;

    let data = ConfigSnapshot {
        files: snapshot
            .iter()
            .map(|fs| FileSnapshot {
                filename: fs.filename.clone(),
                contents: fs.content.clone(),
            })
            .collect(),
        auth: Some(Authentication {
            token: "@TODO".to_string(),
        }),
    };

    if let Err(err) = client.handle_config(data).await {
        println!(
            "Failed to send configuration snapshot to the server: {}",
            err
        );
    }
}

pub async fn init_confmon(
    addr: String,
    port: u16,
    platform: String,
) -> Watcher<
    impl Fn(Snapshot) -> Pin<Box<dyn Future<Output = ()> + Send>> + Clone,
    Pin<Box<dyn Future<Output = ()> + Send>>,
> {
    libconfmon::make_watcher(platform.clone(), POLL_INTERVAL, move |snapshot| {
        let addr = addr.clone();

        Box::pin(async move {
            send_configuration_snapshot(addr, port, snapshot).await;
        }) as Pin<Box<dyn Future<Output = ()> + Send>>
    })
    .await
    .expect("Failed to initialize configuration monitor")
}
