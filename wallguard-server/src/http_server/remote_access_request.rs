use crate::{app_context::AppContext, tunnel::ClientProfile};
use actix_web::{http::header::AUTHORIZATION, web, HttpRequest, HttpResponse, Responder};
use nullnet_libtunnel::Profile;
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
    // - Implement session timeout

    let Some(jwt_token) = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|hv| hv.to_str().ok())
    else {
        return HttpResponse::Unauthorized().body("Missing Authorization header");
    };

    // Checking the device settings also authorizes current request.
    // While fetching data, datastore validates if the token is valid.
    let Ok(feature_enabled) = context
        .datastore
        .device_check_if_remote_access_enabled(jwt_token, body.device_id.clone())
        .await
    else {
        return HttpResponse::InternalServerError()
            .body("Failed to validate if remote access is enabled");
    };

    if !feature_enabled {
        return HttpResponse::BadRequest().body("Remote access is disabled for this device");
    }

    if let Some(profile) = context
        .tunnel
        .lock()
        .await
        .get_profile_by_device_id(&body.device_id)
        .await
    {
        return HttpResponse::Ok().json(ResponsePayload {
            port: profile.get_visitor_addr().port(),
        });
    }

    let Ok(profile) = ClientProfile::new(&body.device_id, &body.ra_type).await else {
        return HttpResponse::InternalServerError().body("Failed to create client profile");
    };

    if context
        .datastore
        .device_new_remote_session(
            jwt_token,
            body.device_id.clone(),
            profile.remote_access_type(),
        )
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().body("Failed to save session info");
    };

    let port = profile.get_visitor_addr().port();

    if context
        .tunnel
        .lock()
        .await
        .insert_profile(profile)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().body("Failed to create client profile");
    }

    HttpResponse::Ok().json(ResponsePayload { port })
}
