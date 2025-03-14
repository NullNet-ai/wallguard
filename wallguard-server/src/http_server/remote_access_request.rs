use crate::{app_context::AppContext, tunnel::ProfileEx};
use actix_web::{http::header::AUTHORIZATION, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RequestPayload {
    device_id: String,
    ra_type: String,
}

#[derive(Serialize)]
pub struct ResponsePayload {
    port: u16,
}

pub async fn remote_access_request(
    req: HttpRequest,
    context: web::Data<AppContext>,
    body: web::Json<RequestPayload>,
) -> impl Responder {
    // @TODO:
    // - Check if profile already exists
    // - Implement session timeout

    let Some(jwt_token) = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|hv| hv.to_str().ok())
    else {
        return HttpResponse::Unauthorized().body("Missing Authorization header");
    };

    let Ok(token) = nullnet_libtoken::Token::from_jwt(jwt_token) else {
        return HttpResponse::Unauthorized().body("Bad token");
    };

    let Ok(profile) = ProfileEx::new(&body.device_id, &body.ra_type).await else {
        return HttpResponse::InternalServerError().body("Failed to create client profile");
    };

    let Ok(_) = context
        .datastore
        .device_new_remote_session(
            jwt_token,
            token.account.device.id,
            profile.remote_access_type(),
        )
        .await
    else {
        return HttpResponse::InternalServerError().body("Failed to save session info");
    };

    let port = profile.visitor_port();

    if let Err(_) = context.tunnel.lock().await.add_profile(profile).await {
        return HttpResponse::InternalServerError().body("Failed to create client profile");
    }

    let response = ResponsePayload { port };
    HttpResponse::Ok().json(response)
}
