use crate::{app_context::AppContext, tunnel::ProfileEx};
use actix_web::{web, HttpResponse, Responder};
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
    context: web::Data<AppContext>,
    body: web::Json<RequestPayload>,
) -> impl Responder {
    // @TODO:
    // - Check if profile already exists
    // - Authorization
    // - Save session to the database
    // - Implement session timeout

    let Ok(profile) = ProfileEx::new(&body.device_id, &body.ra_type).await else {
        return HttpResponse::InternalServerError().body("Failed to create client profile");
    };

    let port = profile.visitor_port();

    if let Err(_) = context.tunnel.lock().await.add_profile(profile).await {
        return HttpResponse::InternalServerError().body("Failed to create client profile");
    }

    let response = ResponsePayload { port };
    HttpResponse::Ok().json(response)
}
