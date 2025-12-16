use actix_web::{
    HttpRequest, HttpResponse, Responder,
    web::{Data, Json},
};
use serde::Deserialize;
use serde_json::json;
use wallguard_common::protobuf::wallguard_models::Alias;

use crate::{
    app_context::AppContext,
    http_proxy::utilities::{authorization, error_json::ErrorJson},
};

#[derive(Deserialize)]
pub struct RequestPayload {
    device_id: String,
    instance_id: String,
    alias: Alias,
}

pub async fn create_alias(
    request: HttpRequest,
    context: Data<AppContext>,
    body: Json<RequestPayload>,
) -> impl Responder {
    let Some(jwt) = authorization::extract_authorization_token(&request) else {
        return HttpResponse::Unauthorized().json(ErrorJson::from("Missing Authorization header"));
    };

    let Ok(device) = context
        .datastore
        .obtain_device_by_id(&jwt, &body.device_id, false)
        .await
    else {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Failed to fetch device record"));
    };

    if device.is_none() {
        return HttpResponse::NotFound().json(ErrorJson::from("Device not found"));
    }

    let device = device.unwrap();

    if !device.authorized {
        return HttpResponse::BadRequest().json(ErrorJson::from("Device is not authorized yet"));
    }

    let Some(client) = context
        .orchestractor
        .get_client(&device.id, &body.instance_id)
        .await
    else {
        return HttpResponse::NotFound().json(ErrorJson::from("Device is not online"));
    };

    if let Err(err) = client.lock().await.create_alias(body.alias.clone()).await {
        return HttpResponse::InternalServerError().json(ErrorJson::from(err));
    }

    HttpResponse::Ok().json(json!({}))
}
