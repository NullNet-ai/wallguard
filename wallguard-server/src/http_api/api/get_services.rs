use actix_web::{HttpRequest, HttpResponse, Responder, web::{Data, Query}};
use serde::Deserialize;
use serde_json::json;

use crate::{app_context::AppContext, http_api::utilities::authorization};

#[derive(Deserialize)]
pub(in crate::http_api) struct QueryParams {
    device_id: String,
}

pub async fn get_services(
    request: HttpRequest,
    context: Data<AppContext>,
    query: Query<QueryParams>,
) -> impl Responder {
    let Some(jwt) = authorization::extract_authorization_token(&request) else {
        return HttpResponse::Unauthorized().body("missing authorization header");
    };

    match context
        .datastore
        .obtain_services(&jwt, &query.device_id, false)
        .await
    {
        Ok(Some(services)) => HttpResponse::Ok().json(services),
        Ok(None) => HttpResponse::Ok().json(json!([])),
        Err(_) => HttpResponse::InternalServerError().body("datastore error"),
    }
}
