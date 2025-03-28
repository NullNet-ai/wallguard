//! ProxyMessage is a wrapper for messages coming from the server
//! that WebSocket is forwarding traffic to.

use actix::Message as ActixMessage;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;

pub(super) struct ProxyMessage {
    pub(super) message: TungsteniteMessage,
}

impl ActixMessage for ProxyMessage {
    type Result = ();
}

impl From<TungsteniteMessage> for ProxyMessage {
    fn from(value: TungsteniteMessage) -> Self {
        ProxyMessage { message: value }
    }
}
