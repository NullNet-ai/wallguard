use actix_web::HttpResponse as ActixResponse;
use actix_web::Result as ActixResult;
use actix_web::error::ErrorInternalServerError as InternalServerError;
use actix_web::http::StatusCode as ActixStatus;
use http_body_util::BodyExt;
use hyper::Response as HyperResponse;
use hyper::body::Incoming as HyperBody;
use hyper::{HeaderMap, header::SET_COOKIE};

// @TODO: Review the cookie forwarding

fn remove_secure_flag_from_cookie(cookie_header: &str) -> String {
    let mut cookie = cookie_header.to_string();
    if let Some(secure_pos) = cookie.find("secure") {
        let secure_end_pos = cookie[secure_pos..].find(";").unwrap_or(cookie.len());
        cookie.replace_range(secure_pos..secure_pos + secure_end_pos, "");
    }
    cookie
}

fn forward_set_cookie_headers(response_headers: &HeaderMap) -> Vec<String> {
    let mut cookies: Vec<String> = Vec::new();
    let set_cookie_headers = response_headers.get_all(SET_COOKIE);

    for cookie_header in set_cookie_headers.iter() {
        let cookie_str = cookie_header.to_str().unwrap();
        let modified_cookie = remove_secure_flag_from_cookie(cookie_str);
        cookies.push(modified_cookie);
    }

    cookies
}

pub(super) async fn convert_response(
    mut response: HyperResponse<HyperBody>,
) -> ActixResult<ActixResponse> {
    let response_status =
        ActixStatus::from_u16(response.status().as_u16()).map_err(InternalServerError)?;

    let mut response_builder = actix_web::HttpResponse::build(response_status);

    let modified_cookies = forward_set_cookie_headers(response.headers());
    for cookie in modified_cookies {
        response_builder.insert_header((SET_COOKIE.as_str(), cookie.as_bytes()));
    }

    for (name, value) in response.headers().iter() {
        if name != SET_COOKIE {
            response_builder.insert_header((name.as_str(), value.as_bytes()));
        }
    }

    let mut data = vec![];

    while let Some(next) = response.body_mut().frame().await {
        let frame = next.map_err(InternalServerError)?;

        if let Some(chunk) = frame.data_ref() {
            data.extend_from_slice(chunk.iter().as_slice());
        }
    }

    Ok(response_builder.body(data))
}
