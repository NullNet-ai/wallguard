use std::future::Future;
use std::pin::Pin;

use libwallguard::{Authentication, ConfigSnapshot, FileSnapshot, WallGuardGrpcInterface};
use nullnet_libconfmon::{Snapshot, Watcher};

use crate::authentication::AutoAuth;

static POLL_INTERVAL: u64 = 500;

async fn send_configuration_snapshot(addr: &str, port: u16, snapshot: Snapshot, token: String) {
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
                println!("Configuration uploaded successfully");
            } else {
                println!("Config upload failed: {}", response.message);
            }
        }
        Err(err) => println!("Failed to send configuration snapshot to the server: {err}"),
    }
}

pub async fn init_confmon(
    addr: String,
    port: u16,
    platform: &str,
    auth: AutoAuth,
) -> Watcher<
    impl Fn(Snapshot) -> Pin<Box<dyn Future<Output = ()> + Send>> + Clone,
    Pin<Box<dyn Future<Output = ()> + Send>>,
> {
    nullnet_libconfmon::make_watcher(platform, POLL_INTERVAL, move |snapshot| {
        let addr = addr.clone();
        let auth = auth.clone();
        Box::pin(async move {
            match auth.obtain_token_safe().await {
                Ok(token) => send_configuration_snapshot(&addr, port, snapshot, token).await,
                Err(message) => println!(
                    "Could not upload configuraiton, authentication failed: {}",
                    message
                ),
            }
        }) as Pin<Box<dyn Future<Output = ()> + Send>>
    })
    .await
    .expect("Failed to initialize configuration monitor")
}
