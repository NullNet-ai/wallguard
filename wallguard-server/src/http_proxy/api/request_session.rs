use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::web::Data;
use actix_web::web::Json;
use nullnet_liberror::Error;
use serde::Deserialize;
use serde_json::json;
use std::net::IpAddr;

use crate::app_context::AppContext;
use crate::datastore::RemoteAccessSession;
use crate::datastore::RemoteAccessType;
use crate::datastore::SSHKeypair;
use crate::http_proxy::utilities::authorization;
use crate::http_proxy::utilities::error_json::ErrorJson;

// We allow remote access to ANY HTTP/HTTPS web server that the client can relay traffic to.
// We trust the user to know where they want to tunnel into, which is why this piece of data is included.
#[derive(Deserialize)]
struct SessionData {
    local_addr: String,
    local_port: u32,
    protocol: String,
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

    if let Err(error) =
        handle_ssh_edgecase(context.clone(), &jwt, &body.device_id, session_type).await
    {
        return HttpResponse::InternalServerError().json(ErrorJson::from(format!(
            "Failed to handle SSH keys: {}",
            error.to_str()
        )));
    }

    let mut session = RemoteAccessSession::new(&body.device_id, &body.instance_id, session_type);

    if matches!(session_type, RemoteAccessType::Ui) {
        let Some(ex_data) = &body.data else {
            return HttpResponse::InternalServerError().json(ErrorJson::from(
                "Cannot create UI session: data block is missing",
            ));
        };

        if !validate_ip_address(&ex_data.local_addr) {
            return HttpResponse::InternalServerError().json(ErrorJson::from("Bad local_addr"));
        }

        if !validate_protocol(&ex_data.protocol) {
            return HttpResponse::InternalServerError()
                .json(ErrorJson::from("Unsupported protocol"));
        }

        session.set_ex_data(
            ex_data.local_addr.clone(),
            ex_data.local_port,
            ex_data.protocol.clone(),
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
        Ok(Some(_)) => {
            // Future enhancement: Validate SSH key expiry or other constraints.
            Ok(())
        }
        Ok(None) => {
            let data = SSHKeypair::generate(device_id).await?;
            context.datastore.create_ssh_keypair(token, &data).await
        }
        Err(err) => Err(err),
    }
}

fn validate_ip_address(addr: &str) -> bool {
    addr.parse::<IpAddr>().is_ok()
}

fn validate_protocol(proto: &str) -> bool {
    matches!(proto.to_ascii_lowercase().as_str(), "http" | "https")
}
