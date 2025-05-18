use crate::app_context::AppContext;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::web::{Data, Payload};

mod ssh_session;

pub(super) async fn open_ssh_session(
    request: HttpRequest,
    context: Data<AppContext>,
    body: Payload,
) -> actix_web::Result<HttpResponse> {
    let (response, mut session, stream) = actix_ws::handle(&request, body)?;

    Ok(response)
}
