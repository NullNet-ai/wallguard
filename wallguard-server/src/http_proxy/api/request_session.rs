use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::web::Data;
use actix_web::web::Json;
use nullnet_liberror::Error;
use serde::Deserialize;
use serde_json::json;

use crate::app_context::AppContext;
use crate::datastore::RemoteAccessSession;
use crate::datastore::RemoteAccessType;
use crate::datastore::SSHKeypair;
use crate::http_proxy::utilities::authorization;
use crate::http_proxy::utilities::error_json::ErrorJson;

#[derive(Deserialize)]
pub struct RequestPayload {
    device_id: String,
    session_type: String,
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

    if let Err(error) =
        handle_ssh_edgecase(context.clone(), &jwt, &body.device_id, session_type).await
    {
        return HttpResponse::InternalServerError().json(ErrorJson::from(format!(
            "Failed to handle SSH keys: {}",
            error.to_str()
        )));
    }

    let session = RemoteAccessSession::new(&body.device_id, session_type);

    if let Err(error) = context.datastore.create_session(&jwt, &session).await {
        return HttpResponse::InternalServerError().json(ErrorJson::from(format!(
            "Datastore operation failed: {}",
            error.to_str()
        )));
    }

    return HttpResponse::Created().json(json!({}));
}

async fn handle_ssh_edgecase(
    context: Data<AppContext>,
    token: &str,
    device_id: &str,
    session_type: RemoteAccessType,
) -> Result<(), Error> {
    if session_type != RemoteAccessType::Ssh {
        return Ok(());
    }

    match context.datastore.obtain_ssh_keypair(token, device_id).await {
        Ok(_) => {
            // Future enhancement: Validate SSH key expiry or other constraints.
            Ok(())
        }
        Err(_) => {
            let data = SSHKeypair::generate(device_id).await?;
            context.datastore.create_ssh_keypair(token, &data).await
        }
    }
}
