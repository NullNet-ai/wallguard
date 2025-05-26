use crate::{
    app_context::AppContext, grpc_server::AuthHandler, proto::wallguard::HeartbeatResponse,
    tunnel::RAType,
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::time::Duration;
use tokio::sync::mpsc;

const HEARTBEAT_INTERVAL_SECS: u64 = 10;

pub struct AuthStream {
    // Used to forcibly send heartbeats to the client
    ntx: mpsc::Sender<()>,

    // Used to receive a notification of a heartbeat
    brx: mpsc::Receiver<()>,
}

impl AuthStream {
    pub fn new(
        auth_handler: AuthHandler,
        context: AppContext,
        tx: mpsc::Sender<Result<HeartbeatResponse, tonic::Status>>,
    ) -> Self {
        let (ntx, nrx) = mpsc::channel(32);
        let (btx, brx) = mpsc::channel(32);

        tokio::spawn(stream_impl(auth_handler, context, tx, nrx, btx));

        Self { ntx, brx }
    }

    pub async fn force_update(&mut self) -> Result<(), Error> {
        self.ntx.send(()).await.handle_err(location!())?;
        let _ = self.brx.recv().await;
        Ok(())
    }
}

async fn stream_impl(
    mut auth_handler: AuthHandler,
    context: AppContext,
    tx: mpsc::Sender<Result<HeartbeatResponse, tonic::Status>>,
    mut rx: mpsc::Receiver<()>,
    n_tx: mpsc::Sender<()>,
) {
    loop {
        if let Ok(token) = auth_handler.obtain_token_safe().await {
            if let Ok(token_info) = nullnet_libtoken::Token::from_jwt(&token) {
                let device_id = token_info.account.device.id;

                if let Ok(response) = context.datastore.heartbeat(&token, device_id.clone()).await {
                    let (remote_shell_enabled, remote_ui_enabled, remote_ssh_enabled) = {
                        let tunnel = context.tunnel.lock().await;

                        let remote_shell_enabled = tunnel
                            .get_profile_by_device_id(&device_id, &RAType::Shell)
                            .await
                            .is_some();

                        let remote_ui_enabled = tunnel
                            .get_profile_by_device_id(&device_id, &RAType::UI)
                            .await
                            .is_some();

                        let remote_ssh_enabled = tunnel
                            .get_profile_by_device_id(&device_id, &RAType::Ssh)
                            .await
                            .is_some();
                        (remote_shell_enabled, remote_ui_enabled, remote_ssh_enabled)
                    };

                    let response = HeartbeatResponse {
                        token,
                        status: response.status.into(),
                        remote_shell_enabled,
                        remote_ui_enabled,
                        remote_ssh_enabled,
                        is_monitoring_enabled: response.is_monitoring_enabled,
                        is_packet_capture_enabled: response.is_packet_capture_enabled,
                        is_resource_monitoring_enabled: response.is_resource_monitoring_enabled,
                    };

                    if tx.send(Ok(response)).await.is_err() {
                        context
                            .clients_manager
                            .lock()
                            .await
                            .on_client_disconnected(&device_id);

                        return;
                    } else {
                        let _ = n_tx.try_send(());
                    }
                }
            };
        }

        tokio::select! {
            _ = rx.recv() => {},
            _ = tokio::time::sleep(Duration::from_secs(HEARTBEAT_INTERVAL_SECS)) => {}
        }
    }
}
