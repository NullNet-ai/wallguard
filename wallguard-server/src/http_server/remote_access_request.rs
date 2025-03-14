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

    // Not sure if this is needed. Yes, we can validate that the token is somewhat valid,
    // but the final decision belongs to the datastore. Maybe we can save some CPU cycles here?
    let Ok(_) = nullnet_libtoken::Token::from_jwt(jwt_token) else {
        return HttpResponse::Unauthorized().body("Malformed token");
    };

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
        .get_profile_by_id(&body.device_id)
    {
        return HttpResponse::Ok().json(ResponsePayload {
            port: profile.visitor_port(),
        });
    }

    let Ok(profile) = ProfileEx::new(&body.device_id, &body.ra_type).await else {
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

    let port = profile.visitor_port();

    if context
        .tunnel
        .lock()
        .await
        .add_profile(profile)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().body("Failed to create client profile");
    }

    HttpResponse::Ok().json(ResponsePayload { port })
}
