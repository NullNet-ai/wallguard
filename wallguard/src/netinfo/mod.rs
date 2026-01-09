mod service;
mod sock;

pub async fn perform_service_discovery() -> Vec<service::ServiceInfo> {
    let sockets = sock::get_sockets_info().await;
    service::gather_info(&sockets).await
}
