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
    session_id: String,
}

pub async fn delete_ssh_session(
    request: HttpRequest,
    context: Data<AppContext>,
    body: Json<RequestPayload>,
) -> impl Responder {
    let Some(jwt) = authorization::extract_authorization_token(&request) else {
        return HttpResponse::Unauthorized().json(ErrorJson::from("Missing Authorization header"));
    };

    let session = match context
        .datastore
        .obtain_ssh_session(&jwt, &body.session_id, false)
        .await
    {
        Ok(Some(session)) => session,
        Ok(None) => return HttpResponse::NotFound().json(json!({})),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(ErrorJson::from("Failed to fetch session"));
        }
    };

    let Some(session) = context.ssh_sessions_manager.remove(&session.id).await else {
        return HttpResponse::NotFound().json(json!({}));
    };

    session.lock().await.terminate().await;

    HttpResponse::Ok().json(json!({}))
}
