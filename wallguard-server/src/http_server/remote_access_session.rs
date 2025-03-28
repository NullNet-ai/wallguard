use crate::app_context::AppContext;
use actix_web::{HttpRequest, HttpResponse, Responder, http::header::AUTHORIZATION, web};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    device_id: String,
}

#[derive(Serialize)]
pub struct ResponsePayload {
    port: u16,
    ra_type: String,
}

pub async fn remote_access_session(
    req: HttpRequest,
    context: web::Data<AppContext>,
    query: web::Query<QueryParams>,
) -> impl Responder {
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
        .device_check_if_remote_access_enabled(jwt_token, query.device_id.clone())
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError()
            .body("Failed to validate if remote access is enabled");
    }

    match context
        .tunnel
        .lock()
        .await
        .get_profile_if_online_by_device_id(&query.device_id)
        .await
    {
        Some(profile) => HttpResponse::Ok().json(json!({
            "session": profile.public_session_id(),
            "type": profile.remote_access_type().to_string()
        })),
        None => HttpResponse::NotFound().json(json!({})),
    }
}
