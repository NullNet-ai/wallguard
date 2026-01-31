use actix_web::{HttpRequest, HttpResponse, Responder, web};
use serde::Deserialize;

use crate::{
    app_context::AppContext,
    http_api::utilities::{authorization, error_json::ErrorJson, request_handling},
};

#[derive(Deserialize)]
pub(in crate::http_api) struct RequestPayload {
    session: String,
}

pub async fn remote_access_terminate(
    request: HttpRequest,
    context: web::Data<AppContext>,
    body: web::Json<RequestPayload>,
) -> impl Responder {
    let Some(jwt) = authorization::extract_authorization_token(&request) else {
        return HttpResponse::Unauthorized().json(ErrorJson::from("Missing Authorization header"));
    };

    let session = match request_handling::fetch_session(&context, &jwt, &body.session).await {
        Ok(sess) => sess,
        Err(resp) => return resp,
    };

    context
        .orchestractor
        .terminate_all_tunnels_for_session(&session.id)
        .await;

    if context
        .datastore
        .delete_remote_access_session(&jwt, &session.id)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError()
            .json(ErrorJson::from("Failed to delete session record"));
    }

    HttpResponse::Ok().body("")
}

// TODO remove