use std::sync::Arc;

use crate::app_context::AppContext;
use crate::datastore::{ServiceInfo, TunnelType};
use crate::http_proxy_v2::connector::Connector;
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
            return Err(Error::new(ErrorType::Custom("Failed to parse tunnel id")));
        };

        let service = self.get_service_info(&tunnel_id).await?;
        ctx.service = Some(service.clone());

        let Ok(tunnel_type) = TunnelType::try_from(service.protocol.as_str()) else {
            return Err(Error::new(ErrorType::InternalError));
        };

        if !matches!(tunnel_type, TunnelType::Http | TunnelType::Https) {
            return Err(Error::new(ErrorType::Custom("Wrong tunnel type")));
        }

        let address = format!("{}:{}", service.address, service.port);

        let mut peer = HttpPeer::new(
            address,
            matches!(tunnel_type, TunnelType::Https),
            service.address.clone(),
        );

        peer.options.custom_l4 = Some(Arc::new(Connector::new(self.context.clone(), service)));

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
        if let Some(host) = ctx.service.as_ref().map(|data| data.address.clone()) {
            upstream_request.insert_header("Host", host)?;
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

    async fn get_service_info(&self, tunnel_id: &str) -> Result<ServiceInfo, BError> {
        let token = self
            .context
            .sysdev_token_provider
            .get()
            .await
            .map_err(|_| Error::new(ErrorType::InternalError))?;

        let tunnel = self
            .context
            .datastore
            .obtain_tunnel(&token.jwt, tunnel_id, false)
            .await
            .map_err(|_| Error::new(ErrorType::InternalError))?
            .ok_or(Error::new(ErrorType::Custom("Tunnel Not Found")))?;

        self.context
            .datastore
            .obtain_service(&token.jwt, &tunnel.service_id, false)
            .await
            .map_err(|_| Error::new(ErrorType::InternalError))?
            .ok_or(Error::new(ErrorType::Custom("Service Not Found")))
    }
}
