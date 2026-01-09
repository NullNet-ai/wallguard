use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::web::{Data, Payload};

use crate::app_context::AppContext;
use crate::datastore::RemoteAccessType;
use crate::http_proxy::utilities::error_json::ErrorJson;
use crate::http_proxy::utilities::request_handling;
use crate::http_proxy::utilities::tunneling;
use crate::reverse_tunnel::TunnelAdapter;

mod request;

pub async fn proxy_http_request(
    request: HttpRequest,
    context: Data<AppContext>,
    body: Payload,
) -> impl Responder {
    log::info!("Proxy request: {request:?}");

    let session_token = match request_handling::extract_session_token(&request) {
        Ok(token) => token,
        Err(resp) => return resp,
    };

    let token = match request_handling::fetch_token(&context).await {
        Ok(t) => t,
        Err(resp) => return resp,
    };

    let session = match request_handling::fetch_session(&context, &token.jwt, &session_token).await
    {
        Ok(sess) => sess,
        Err(resp) => return resp,
    };

    if let Err(resp) = request_handling::ensure_session_type(&session, RemoteAccessType::Ui) {
        return resp;
    }

    let Ok(device) = context
        .datastore
        .obtain_device_by_id(&token.jwt, &session.device_id, false)
        .await
    else {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Unable to retrieve device from datastore"));
    };

    if device.is_none() {
        return HttpResponse::NotFound().json(ErrorJson::from("Associated device not found"));
    }

    let device = device.unwrap();

    if !device.authorized {
        return HttpResponse::NotFound().json(ErrorJson::from("Device is unauthorized"));
    }

    let (Some(local_addr), Some(local_port), Some(protocol)) =
        (session.local_addr, session.local_port, session.protocol)
    else {
        return HttpResponse::InternalServerError().json(ErrorJson::from(
            "Malformed session data: missing required info",
        ));
    };

    let Ok(tunnel) = tunneling::establish_tunneled_ui(
        &context,
        &device.id,
        &session.instance_id,
        &protocol,
        &local_addr,
        local_port,
    )
    .await
    else {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Failed to establish a tunnel"));
    };

    if !tunnel.is_authenticated() {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Tunnel is not authenticated"));
    }

    let Ok(tunnel_adapter) = TunnelAdapter::try_from(tunnel) else {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Failed to adapt tunnel transport"));
    };

    let (tunnel_id, tunnel_terminate) = context
        .orchestractor
        .on_tunnel_established(&session.id)
        .await;

    let proxy_fut = request::proxy_request(
        request,
        body,
        "domain.com", // TODO: real domain
        protocol.to_lowercase() == "https",
        tunnel_adapter,
    );

    tokio::pin!(proxy_fut);

    let response = tokio::select! {
        _ = tunnel_terminate => {
            HttpResponse::InternalServerError()
                .json(ErrorJson::from("Tunnel has been terminated unexpectedly"))
        }

        result = &mut proxy_fut => {
            result
        }
    };

    context
        .orchestractor
        .on_tunnel_terminated(&session.id, &tunnel_id)
        .await;

    response
}
