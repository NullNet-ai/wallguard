use crate::control_channel::OutboundStream;
use crate::storage::{Secret, Storage};
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use nullnet_libwallguard::{client_message, Authentication, ClientMessage};

pub async fn send_authenticate(outbound: OutboundStream) -> Result<(), Error> {
    let app_id = Storage::get_value(Secret::APP_ID)
        .await
        .ok_or("APP_ID not set")
        .handle_err(location!());

    let app_secret = Storage::get_value(Secret::APP_SECRET)
        .await
        .ok_or("APP_SECRET not set")
        .handle_err(location!());

    let message = ClientMessage {
        message: Some(client_message::Message::Authentication(Authentication {
            app_id: app_id.unwrap(),
            app_secret: app_secret.unwrap(),
        })),
    };

    outbound
        .lock()
        .await
        .send(message)
        .await
        .handle_err(location!())
}
