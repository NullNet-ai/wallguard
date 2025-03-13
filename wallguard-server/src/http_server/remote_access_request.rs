use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::tunnel::ProfileEx;

use super::state::State;

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
    state: web::Data<State>,
    body: web::Json<RequestPayload>,
) -> impl Responder {
    // @TODO:
    // - Check if profile already exists
    // - Authorization
    // - Save session to the database 
    // - Implement session timeout
    
    let tunnel = state.tunnel.clone();

    let Ok(profile) = ProfileEx::new(&body.device_id, &body.ra_type).await else {
        return HttpResponse::InternalServerError().body("Failed to create client profile");
    };

    let port = profile.visitor_port();

    if let Err(_) = tunnel.lock().await.add_profile(profile).await {
        return HttpResponse::InternalServerError().body("Failed to create client profile");
    }

    let response = ResponsePayload { port };
    HttpResponse::Ok().json(response)
}
