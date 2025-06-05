//! TODO:
//! - Fetch session and related device in 1 Datastore query
//! - Use actual device HTTP protocol (http vs https)

use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::web::{Data, Payload};

use crate::app_context::AppContext;
use crate::datastore::RemoteAccessType;
use crate::http_proxy::utilities::error_json::ErrorJson;
use crate::http_proxy::utilities::request_handling;
use crate::http_proxy::utilities::tunneling;

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
        .obtain_device_by_id(&token.jwt, &session.device_id)
        .await
    else {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Unable to retrieve device from datastore"));
    };

    if device.is_none() {
        return HttpResponse::NotFound().json(ErrorJson::from("Associated device not found"));
    }

    let device = device.unwrap();
    let protocol = "http";

    let Ok(stream) = tunneling::establish_tunneled_ui(&context, &device.uuid, protocol).await
    else {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Failed to establish a tunnel"));
    };

    request::proxy_request(request, body, "domain.com", false, stream).await
}
