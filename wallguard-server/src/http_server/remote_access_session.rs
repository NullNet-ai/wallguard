use std::str::FromStr;

use crate::{app_context::AppContext, tunnel::RAType};
use actix_web::{HttpRequest, HttpResponse, Responder, http::header::AUTHORIZATION, web};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, Debug)]
pub struct QueryParams {
    device_id: String,
    ra_type: String,
}

pub async fn remote_access_session(
    req: HttpRequest,
    context: web::Data<AppContext>,
    query: web::Query<QueryParams>,
) -> impl Responder {
    let Ok(ra_type) = RAType::from_str(&query.ra_type) else {
        return HttpResponse::BadRequest().body("Bad ra_type parameter value");
    };

    let Some(jwt_token) = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|hv| hv.to_str().ok())
    else {
        return HttpResponse::Unauthorized().body("Missing Authorization header");
    };

    // Checking the device settings also authorizes current request.
    // While fetching data, datastore validates if the token is valid.
    if context
        .datastore
        .device_check_if_remote_access_enabled(jwt_token, query.device_id.clone())
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError()
            .body("Failed to validate if remote access is enabled");
    }

    match context
        .tunnel
        .lock()
        .await
        .get_profile_if_online_by_device_id(&query.device_id, &ra_type)
        .await
    {
        Some(profile) => HttpResponse::Ok().json(json!({
            "session": profile.public_session_id(),
        })),
        None => HttpResponse::NotFound().json(json!({})),
    }
}
