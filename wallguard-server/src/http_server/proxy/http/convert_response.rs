use actix_web::error::ErrorInternalServerError as InternalServerError;
use actix_web::http::StatusCode as ActixStatus;
use actix_web::HttpResponse as ActixResponse;
use actix_web::Result as ActixResult;
use http_body_util::BodyExt;
use hyper::body::Incoming as HyperBody;
use hyper::Response as HyperResponse;

pub(super) async fn convert_response(
    mut response: HyperResponse<HyperBody>,
) -> ActixResult<ActixResponse> {
    let response_status =
        ActixStatus::from_u16(response.status().as_u16()).map_err(InternalServerError)?;

    let mut response_builder = actix_web::HttpResponse::build(response_status);

    for (name, value) in response.headers().iter() {
        response_builder.insert_header((name.as_str(), value.as_bytes()));
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
