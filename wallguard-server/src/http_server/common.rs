use actix_web::HttpRequest;

pub fn extract_session_from_request(req: &HttpRequest) -> Option<String> {
    req.full_url()
        .domain()
        .and_then(extract_session_from_domain)
        .map(|v| v.to_owned())
}

fn extract_session_from_domain(domain: &str) -> Option<&str> {
    domain.split_once('.').map(|(session, _)| session)
}
