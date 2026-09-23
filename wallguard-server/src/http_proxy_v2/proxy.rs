use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;

use crate::app_context::AppContext;
use crate::datastore::{ServiceInfo, TunnelType};
use crate::http_proxy_v2::connector::Connector;
use crate::tunneling::tunnel_common::WallguardTunnel;

use pingora::prelude::*;
use pingora::upstreams::peer::HttpPeer;
use tonic::async_trait;

/// Pooled upstream connections idle for longer than this are closed. Each
/// one pins a tunnel socket plus a local socket on the agent, and Pingora's
/// default is to keep idle pooled connections until the upstream closes.
const POOL_IDLE_TIMEOUT: Duration = Duration::from_secs(60);

pub struct Proxy {
    context: AppContext,
}

impl Proxy {
    pub fn new(context: AppContext) -> Self {
        Self { context }
    }
}

#[derive(Default, Debug)]
pub struct RequestContext {
    pub service: Option<ServiceInfo>,
}

#[async_trait]
impl ProxyHttp for Proxy {
    type CTX = RequestContext;

    fn new_ctx(&self) -> Self::CTX {
        RequestContext::default()
    }

    async fn upstream_peer(
        &self,
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let Some(tunnel_id) = Proxy::parse_tunnel_id(session).await else {
            return Err(Error::new(ErrorType::HTTPStatus(400)));
        };

        let Some(tunnel) = self.context.tunnels_manager.get(&tunnel_id).await else {
            return Err(Error::new(ErrorType::HTTPStatus(404)));
        };

        let WallguardTunnel::Http(ref http_tunnel) = tunnel else {
            return Err(Error::new(ErrorType::HTTPStatus(400)));
        };

        let mut td = http_tunnel.lock().await;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let (date, time) = crate::utilities::time::timestamp_to_datetime(timestamp.cast_signed());
        td.data.tunnel_data.last_access_date = Some(date);
        td.data.tunnel_data.last_access_time = Some(time);

        self.update_tunnel_record(&td.data.tunnel_data.id, timestamp)
            .await;

        let address = format!(
            "{}:{}",
            td.data.service_data.address, td.data.service_data.port
        );

        let mut peer = HttpPeer::new(
            address,
            matches!(td.data.tunnel_data.tunnel_type, TunnelType::Https),
            td.data.service_data.address.clone(),
        );

        ctx.service = Some(td.data.service_data.clone());

        drop(td);

        peer.options.custom_l4 = Some(Arc::new(Connector::new(tunnel)));

        // Pingora's pool key covers the address, scheme, SNI and TLS
        // settings but not `custom_l4`, so two tunnels to the same service
        // address (every device's webgui on 127.0.0.1:443, say) would share
        // pooled connections: a request for one device could be sent down
        // another device's tunnel. Keying the pool on the tunnel prevents it.
        let mut hasher = DefaultHasher::new();
        tunnel_id.hash(&mut hasher);
        peer.group_key = hasher.finish();
        peer.options.idle_timeout = Some(POOL_IDLE_TIMEOUT);

        peer.options.verify_cert = false;
        peer.options.verify_hostname = false;

        Ok(Box::new(peer))
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        if let Some(service) = ctx.service.as_ref() {
            upstream_request.insert_header("host", service.address.as_str())?;
            upstream_request
                .insert_header("referer", format!("{}://localhost/", service.protocol))?;
        }

        Ok(())
    }
}

impl Proxy {
    async fn parse_tunnel_id(session: &mut Session) -> Option<String> {
        let request = session.req_header();

        if let Some(domain) = request.uri.host()
            && let Some((tunnel, _)) = domain.split_once('.')
        {
            return Some(tunnel.to_ascii_uppercase());
        }

        if let Some(host_val) = request.headers.get("host")
            && let Ok(host_str) = host_val.to_str()
        {
            let host_only = host_str.split(':').next().unwrap_or(host_str);

            if let Some((tunnel, _rest)) = host_only.split_once('.') {
                return Some(tunnel.to_ascii_uppercase());
            }
        }

        None
    }

    async fn update_tunnel_record(&self, tunnel_id: &str, timestamp: u64) {
        if let Ok(token) = self.context.sysdev_token_provider.get().await {
            let _ = self
                .context
                .datastore
                .update_tunnel_accessed(&token.jwt, tunnel_id, false, timestamp)
                .await;
        }
    }
}
