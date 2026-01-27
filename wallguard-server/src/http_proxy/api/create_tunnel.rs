use actix_web::{
    HttpRequest, HttpResponse, Responder,
    web::{Data, Json},
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    app_context::AppContext,
    datastore::{TunnelModel, TunnelType},
    http_proxy::utilities::{authorization, error_json::ErrorJson},
};

#[derive(Deserialize)]
pub(in crate::http_proxy) struct RequestPayload {
    device_id: String,
    service_id: String,
}

pub async fn create_tunnel(
    request: HttpRequest,
    context: Data<AppContext>,
    body: Json<RequestPayload>,
) -> impl Responder {
    let Some(jwt) = authorization::extract_authorization_token(&request) else {
        return HttpResponse::Unauthorized().json(ErrorJson::from("Missing Authorization header"));
    };

    let Ok(Some(device)) = context
        .datastore
        .obtain_device_by_id(&jwt, &body.device_id, false)
        .await
    else {
        return HttpResponse::BadRequest().json(ErrorJson::from("Device not found"));
    };

    if !device.authorized {
        return HttpResponse::BadRequest().json(ErrorJson::from("Device is not authorized"));
    }

    let Ok(Some(service)) = context
        .datastore
        .obtain_service(&jwt, &body.service_id, false)
        .await
    else {
        return HttpResponse::BadRequest().json(ErrorJson::from("Service not found"));
    };

    if service.device_id != device.id {
        return HttpResponse::BadRequest().json(ErrorJson::from("Wrong service id"));
    }

    if let Ok(false) = context
        .datastore
        .does_tunnel_for_service_exist(&jwt, &service.id, false)
        .await
    {
        return HttpResponse::Conflict()
            .json(ErrorJson::from("Tunnel for the service already exists"));
    };

    let Ok(tunnel_type) = TunnelType::try_from(service.protocol.as_str()) else {
        return HttpResponse::BadRequest().json(ErrorJson::from("Unsupported tunnel type"));
    };

    let model = TunnelModel {
        device_id: device.id,
        service_id: service.id,
        tunnel_type,
        ..Default::default()
    };

    if context.datastore.create_tunnel(&jwt, &model).await.is_err() {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Unsupported tunnel type"));
    }

    HttpResponse::Ok().json(json!({}))
}
