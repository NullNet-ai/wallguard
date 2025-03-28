use actix_web_actors::ws::CloseCode as ActixCloseCode;
use actix_web_actors::ws::CloseReason as ActixCloseReason;
use actix_web_actors::ws::Message as ActixMessage;
use std::error::Error;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode as TungsteniteCloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame as TungsteniteCloseFrame;
use tokio_tungstenite::tungstenite::{protocol::Message as TungsteniteMessage, Utf8Bytes};

pub(super) fn actix_to_tungstenite(
    message: ActixMessage,
) -> Result<TungsteniteMessage, Box<dyn Error>> {
    match message {
        ActixMessage::Text(text) => {
            let data = Utf8Bytes::try_from(text.as_bytes().clone())?;
            Ok(TungsteniteMessage::Text(data))
        }
        ActixMessage::Binary(data) => Ok(TungsteniteMessage::Binary(data)),
        ActixMessage::Ping(data) => Ok(TungsteniteMessage::Ping(data)),
        ActixMessage::Pong(data) => Ok(TungsteniteMessage::Pong(data)),
        ActixMessage::Close(close_reason) => {
            let close_frame = actix_close_reason_to_tungstenite_close_frame(close_reason);
            Ok(TungsteniteMessage::Close(close_frame))
        }
        ActixMessage::Continuation(_) | ActixMessage::Nop => {
            Err("Unexpected or unsupported message".into())
        }
    }
}

pub(super) fn tungstenite_to_actix(
    message: TungsteniteMessage,
) -> Result<ActixMessage, Box<dyn Error>> {
    match message {
        TungsteniteMessage::Text(text) => {
            let data = std::str::from_utf8(text.as_bytes())?;
            Ok(ActixMessage::Text(data.into()))
        }
        TungsteniteMessage::Binary(data) => Ok(ActixMessage::Binary(data)),
        TungsteniteMessage::Ping(data) => Ok(ActixMessage::Ping(data)),
        TungsteniteMessage::Pong(data) => Ok(ActixMessage::Pong(data)),
        TungsteniteMessage::Close(close_frame) => {
            let reason = tungstenite_close_frame_to_actix_close_reason(close_frame);
            Ok(ActixMessage::Close(reason))
        }
        // We only read tungstenite messages
        TungsteniteMessage::Frame(_) => unreachable!(),
    }
}

fn actix_close_reason_to_tungstenite_close_frame(
    reason: Option<ActixCloseReason>,
) -> Option<TungsteniteCloseFrame> {
    reason.map(|r| TungsteniteCloseFrame {
        code: match r.code {
            ActixCloseCode::Normal => TungsteniteCloseCode::Normal,
            ActixCloseCode::Away => TungsteniteCloseCode::Away,
            ActixCloseCode::Protocol => TungsteniteCloseCode::Protocol,
            ActixCloseCode::Unsupported => TungsteniteCloseCode::Unsupported,
            ActixCloseCode::Abnormal => TungsteniteCloseCode::Abnormal,
            ActixCloseCode::Invalid => TungsteniteCloseCode::Invalid,
            ActixCloseCode::Policy => TungsteniteCloseCode::Policy,
            ActixCloseCode::Size => TungsteniteCloseCode::Size,
            ActixCloseCode::Extension => TungsteniteCloseCode::Extension,
            ActixCloseCode::Restart => TungsteniteCloseCode::Restart,
            ActixCloseCode::Again => TungsteniteCloseCode::Again,
            ActixCloseCode::Error | _ => TungsteniteCloseCode::Error,
        },
        reason: r.description.map(|desc| desc.into()).unwrap_or_default(),
    })
}

fn tungstenite_close_frame_to_actix_close_reason(
    frame: Option<TungsteniteCloseFrame>,
) -> Option<ActixCloseReason> {
    frame.map(|reason| ActixCloseReason {
        code: match reason.code {
            TungsteniteCloseCode::Normal => ActixCloseCode::Normal,
            TungsteniteCloseCode::Away => ActixCloseCode::Away,
            TungsteniteCloseCode::Protocol => ActixCloseCode::Protocol,
            TungsteniteCloseCode::Unsupported => ActixCloseCode::Unsupported,
            TungsteniteCloseCode::Abnormal => ActixCloseCode::Abnormal,
            TungsteniteCloseCode::Invalid => ActixCloseCode::Invalid,
            TungsteniteCloseCode::Policy => ActixCloseCode::Policy,
            TungsteniteCloseCode::Size => ActixCloseCode::Size,
            TungsteniteCloseCode::Extension => ActixCloseCode::Extension,
            TungsteniteCloseCode::Restart => ActixCloseCode::Restart,
            TungsteniteCloseCode::Again => ActixCloseCode::Again,
            TungsteniteCloseCode::Error | _ => ActixCloseCode::Error,
        },
        description: Some(reason.reason.to_string()),
    })
}
