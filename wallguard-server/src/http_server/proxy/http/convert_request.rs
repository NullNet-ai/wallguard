use actix_web::error::ErrorInternalServerError as InternalServerError;
use actix_web::http::Method as ActixMethod;
use actix_web::web::Payload as ActixBody;
use actix_web::HttpRequest as ActixRequest;
use actix_web::Result as ActixResult;
use http_body_util::Full as BodyWrapper;
use hyper::body::Bytes as HyperBody;
use hyper::header::HeaderName as HyperHeaderName;
use hyper::header::HeaderValue as HyperHeaderValue;
use hyper::Method as HyperMethod;
use hyper::Request as HyperRequest;
use std::net::SocketAddr;

pub(super) async fn convert_request(
    request: ActixRequest,
    body: ActixBody,
    target: SocketAddr,
) -> ActixResult<HyperRequest<BodyWrapper<HyperBody>>> {
    let uri: hyper::Uri = request
        .uri()
        .to_string()
        .parse()
        .map_err(InternalServerError)?;

    let method = convert_method(request.method());

    let mut request_builder = hyper::Request::builder().method(method).uri(uri);

    for (header_name, header_value) in request.headers() {
        if header_name.as_str() == "host" {
            continue;
        }

        if let (Ok(name), Ok(value)) = (
            HyperHeaderName::from_bytes(header_name.as_str().as_bytes()),
            HyperHeaderValue::from_bytes(header_value.as_bytes()),
        ) {
            request_builder = request_builder.header(name, value);
        }
    }

    request_builder = request_builder.header(hyper::header::HOST, target.to_string());
    // @TODO: change origin ? Add X-Forwarded-For ?

    let bbbb = HyperBody::from(body.to_bytes().await?);
    let body = BodyWrapper::new(bbbb);

    let request = request_builder.body(body).map_err(InternalServerError)?;

    Ok(request)
}

fn convert_method(method: &ActixMethod) -> HyperMethod {
    match *method {
        ActixMethod::CONNECT => HyperMethod::CONNECT,
        ActixMethod::DELETE => HyperMethod::DELETE,
        ActixMethod::GET => HyperMethod::GET,
        ActixMethod::HEAD => HyperMethod::HEAD,
        ActixMethod::OPTIONS => HyperMethod::OPTIONS,
        ActixMethod::PATCH => HyperMethod::PATCH,
        ActixMethod::POST => HyperMethod::POST,
        ActixMethod::PUT => HyperMethod::PUT,
        ActixMethod::TRACE => HyperMethod::TRACE,
        _ => HyperMethod::GET,
    }
}
