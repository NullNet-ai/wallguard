use actix_web::Result as ActixResult;
use actix_web::error::ErrorUnauthorized;
use nullnet_libtunnel::{Message, Payload, write_with_confirmation};
use tokio::net::TcpStream;

pub(crate) async fn authenticate(stream: &mut TcpStream, token: &str) -> ActixResult<()> {
    let token_hash = nullnet_libtunnel::str_hash(token);
    let message = Message::Authenticate(Payload::from(token_hash));

    write_with_confirmation(stream, message)
        .await
        .map_err(|err| ErrorUnauthorized(err.to_str().to_owned()))
}
