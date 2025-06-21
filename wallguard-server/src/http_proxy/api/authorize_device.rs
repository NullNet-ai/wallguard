use crate::app_context::AppContext;
use crate::http_proxy::utilities::authorization;
use crate::http_proxy::utilities::error_json::ErrorJson;
use crate::protocol::wallguard_commands::AuthenticationData;
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

    if device.authorized {
        return HttpResponse::Ok().json(json!({}));
    }

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

    if let Some(client) = context.orchestractor.get_client(&device.uuid).await {
        let Ok(systoken) = context.sysdev_token_provider.get().await else {
            return HttpResponse::InternalServerError()
                .json(ErrorJson::from("Failed to obtain sysdev token"));
        };

        let Ok(credentials) = context
            .datastore
            .obtain_device_credentials(&systoken.jwt, &device.uuid)
            .await
        else {
            return HttpResponse::InternalServerError()
                .json(ErrorJson::from("Failed to obtain device credentials"));
        };

        if let Some(credentials) = credentials {
            let mut lock = client.lock().await;

            if lock
                .authorize(AuthenticationData {
                    app_id: Some(credentials.account_id),
                    app_secret: Some(credentials.account_secret),
                })
                .await
                .is_err()
            {
                return HttpResponse::InternalServerError()
                    .json(ErrorJson::from("Failed to send approval"));
            }

            if context
                .datastore
                .delete_device_credentials(&systoken.jwt, &credentials.id)
                .await
                .is_err()
            {
                return HttpResponse::InternalServerError()
                    .json(ErrorJson::from("Failed to redeem credentials"));
            }
        }
    };

    HttpResponse::Ok().json(json!({}))
}
