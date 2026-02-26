use actix_web::{
    HttpRequest, HttpResponse, Responder,
    web::{Data, Json},
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    app_context::AppContext,
    http_api::utilities::{authorization, error_json::ErrorJson},
};

#[derive(Deserialize)]
pub(in crate::http_api) struct RequestPayload {
    tunnel_id: String,
}

pub async fn delete_tunnel(
    request: HttpRequest,
    context: Data<AppContext>,
    body: Json<RequestPayload>,
) -> impl Responder {
    let Some(jwt) = authorization::extract_authorization_token(&request) else {
        return HttpResponse::Unauthorized().json(ErrorJson::from("Missing Authorization header"));
    };

    if context
        .datastore
        .obtain_tunnel(&jwt, &body.tunnel_id, false)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Failed to fetch session"));
    };

    let _ = context
        .tunnels_manager
        .on_tunnel_terminated(&body.tunnel_id)
        .await;

    HttpResponse::Ok().json(json!({}))
}
