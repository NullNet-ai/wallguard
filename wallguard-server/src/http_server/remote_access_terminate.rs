use crate::app_context::AppContext;
use actix_web::{HttpRequest, HttpResponse, Responder, http::header::AUTHORIZATION, web};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct RequestPayload {
    device_id: String,
}

pub async fn remote_access_terminate(
    req: HttpRequest,
    context: web::Data<AppContext>,
    body: web::Json<RequestPayload>,
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
        .device_check_if_remote_access_enabled(jwt_token, body.device_id.clone())
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError()
            .body("Failed to validate if remote access is enabled");
    }

    let body = body.into_inner();
    match context
        .tunnel
        .lock()
        .await
        .remove_profile(&body.device_id)
        .await
    {
        Ok(_) => HttpResponse::Ok().body(""),
        Err(err) => HttpResponse::NotFound().json(json!({"error": err.to_str()})),
    }
}
