use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::web::Data;
use actix_web::web::Json;
use serde::Deserialize;
use serde_json::json;

use crate::app_context::AppContext;
use crate::datastore::RemoteAccessSession;
use crate::datastore::RemoteAccessType;
use crate::http_api::utilities::authorization;
use crate::http_api::utilities::error_json::ErrorJson;

#[derive(Deserialize)]
struct SessionData {
    service_id: String,
}

#[derive(Deserialize)]
pub struct RequestPayload {
    device_id: String,
    instance_id: String,
    session_type: String,
    data: Option<SessionData>,
}

pub async fn request_session(
    request: HttpRequest,
    context: Data<AppContext>,
    body: Json<RequestPayload>,
) -> impl Responder {
    let Some(jwt) = authorization::extract_authorization_token(&request) else {
        return HttpResponse::Unauthorized().json(ErrorJson::from("Missing Authorization header"));
    };

    let session_type = match RemoteAccessType::try_from(body.session_type.as_str()) {
        Ok(value) => value,
        Err(err) => {
            return HttpResponse::BadRequest().json(ErrorJson::from(err));
        }
    };

    let mut session = RemoteAccessSession::new(&body.device_id, &body.instance_id, session_type);

    if matches!(session_type, RemoteAccessType::Ui) {
        let Some(ex_data) = &body.data else {
            return HttpResponse::InternalServerError().json(ErrorJson::from(
                "Cannot create UI session: data block is missing",
            ));
        };

        let Ok(Some(service)) = context
            .datastore
            .obtain_service(&jwt, &ex_data.service_id, false)
            .await
        else {
            return HttpResponse::InternalServerError().json(ErrorJson::from(
                "Cannot create UI session: cannot fetch service data",
            ));
        };

        session.set_ex_data(
            service.address.clone(),
            service.port as u32,
            service.protocol.clone(),
        );
    }

    if let Err(error) = context.datastore.create_session(&jwt, &session).await {
        return HttpResponse::InternalServerError().json(ErrorJson::from(format!(
            "Datastore operation failed: {}",
            error.to_str()
        )));
    }

    HttpResponse::Created().json(json!({"session_token": session.token}))
}

// TODO remove