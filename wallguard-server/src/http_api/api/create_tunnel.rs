use actix_web::{
    HttpRequest, HttpResponse, Responder,
    body::BoxBody,
    http::StatusCode,
    web::{Data, Json},
};
use serde::Deserialize;
use serde_json::json;

use crate::{app_context::AppContext, http_api::utilities::authorization};

#[derive(Deserialize)]
pub(in crate::http_api) struct RequestPayload {
    device_id: String,
    service_id: String,
}

pub async fn create_tunnel(
    request: HttpRequest,
    context: Data<AppContext>,
    body: Json<RequestPayload>,
) -> impl Responder {
    let Some(jwt) = authorization::extract_authorization_token(&request) else {
        return HttpResponse::Unauthorized().body("missing authorization header");
    };

    let id = match context
        .tunnels_manager
        .request(
            &jwt,
            &body.device_id,
            &body.service_id,
            context.clone().into_inner(),
        )
        .await
    {
        Ok(id) => id,
        Err(err) => {
            let status = StatusCode::from_u16(err.to_http_status())
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

            return HttpResponse::new(status).set_body(BoxBody::new(err.to_string()));
        }
    };

    HttpResponse::Ok().json(json!({"tunnel_id": id}))
}
