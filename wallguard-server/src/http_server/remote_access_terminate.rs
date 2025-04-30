use crate::{app_context::AppContext, tunnel::RAType};
use actix_web::{HttpRequest, HttpResponse, Responder, http::header::AUTHORIZATION, web};
use serde::Deserialize;
use serde_json::json;
use std::str::FromStr;

#[derive(Deserialize)]
pub struct RequestPayload {
    device_id: String,
    ra_type: String,
}

pub async fn remote_access_terminate(
    req: HttpRequest,
    context: web::Data<AppContext>,
    body: web::Json<RequestPayload>,
) -> impl Responder {
    let Ok(ra_type) = RAType::from_str(&body.ra_type) else {
        return HttpResponse::BadRequest().body("Bad ra_type parameter value");
    };

    let Some(jwt_token) = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|hv| hv.to_str().ok())
    else {
        return HttpResponse::Unauthorized().body("Missing Authorization header");
    };

    // Checking the device settings also authorizes current request.
    // While fetching data, datastore validates if the token is valid.
    if context
        .datastore
        .device_check_if_remote_access_enabled(jwt_token, body.device_id.clone())
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError()
            .body("Failed to validate if remote access is enabled");
    }

    let body = body.into_inner();
    if let Err(err) = context
        .tunnel
        .lock()
        .await
        .remove_profile(&body.device_id, &ra_type)
        .await
    {
        return HttpResponse::NotFound().json(json!({"error": err.to_str()}));
    }

    // Ignore error if client is not yet connected
    let _ = context
        .clients_manager
        .lock()
        .await
        .force_heartbeat(&body.device_id)
        .await;

    if context
        .datastore
        .device_terminate_remote_session(jwt_token, body.device_id.clone(), ra_type)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().body("Failed to save session info");
    };

    HttpResponse::Ok().body("")
}
