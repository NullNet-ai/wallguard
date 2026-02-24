cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        mod app_context;
        mod control_service;
        mod datastore;
        mod http_api;
        mod http_proxy_v2;
        mod mcp;
        mod orchestrator;
        mod reverse_tunnel;
        mod token_provider;
        mod traffic_handler;
        mod tunneling;
        mod utilities;
        mod linux_main;

        #[tokio::main]
        async fn main() {
            linux_main::linux_main().await
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    println!("wallguard-server is Linux-only");
}
