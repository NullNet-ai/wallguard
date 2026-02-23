use std::sync::Arc;

use crate::app_context::AppContext;
use crate::datastore::{ServiceInfo, TunnelType};
use crate::http_proxy_v2::connector::Connector;
use crate::tunneling::tunnel_common::WallguardTunnel;

use pingora::prelude::*;
use pingora::upstreams::peer::HttpPeer;
use tonic::async_trait;

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

        let td = http_tunnel.lock().await;

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
}
