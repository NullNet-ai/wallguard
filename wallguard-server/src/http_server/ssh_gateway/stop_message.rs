use actix::Message;

#[derive(Message)]
#[rtype(result = "()")]
pub(super) struct StopSession;
