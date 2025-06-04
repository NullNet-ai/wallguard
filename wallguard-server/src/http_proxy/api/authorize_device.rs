use crate::app_context::AppContext;
use crate::http_proxy::utilities::authorization;
use crate::http_proxy::utilities::error_json::ErrorJson;
use crate::utilities;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::web::Data;
use actix_web::web::Json;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct RequestPayload {
    device_id: String,
}

pub async fn authorize_device(
    request: HttpRequest,
    context: Data<AppContext>,
    body: Json<RequestPayload>,
) -> impl Responder {
    let Some(jwt) = authorization::extract_authorization_token(&request) else {
        return HttpResponse::Unauthorized().json(ErrorJson::from("Missing Authorization header"));
    };

    let Ok(value) = context
        .datastore
        .obtain_device_by_id(&jwt, &body.device_id)
        .await
    else {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Failed to fetch device record"));
    };

    let Some(mut device) = value else {
        return HttpResponse::BadRequest().json(ErrorJson::from("Device not found"));
    };

    device.authorized = true;

    if context
        .datastore
        .update_device(&jwt, &body.device_id, &device)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Failed to update device record"));
    };

    let (app_id, app_secret) = generate_credentials();

    if context
        .datastore
        .create_org_account(&jwt, &body.device_id, &app_id, &app_secret)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Failed to create credentials record"));
    }

    let Some(auth) = context.orchestractor.get_pending_auth(&device.uuid).await else {
        return HttpResponse::BadRequest().json(ErrorJson::from("Device not connected"));
    };

    if auth.send_approve(app_id, app_secret).await.is_err() {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Failed to send approval"));
    }

    HttpResponse::Ok().json(json!({}))
}

fn generate_credentials() -> (String, String) {
    let app_id = utilities::random::generate_random_string(12);
    let app_secret = utilities::random::generate_random_string(32);
    return (app_id, app_secret);
}
