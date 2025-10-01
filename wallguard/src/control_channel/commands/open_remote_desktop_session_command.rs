use crate::context::Context;
use crate::control_channel::command::ExecutableCommand;

pub struct OpenRemoteDesktopSessionCommand {
    _context: Context,
    _token: String,
}

impl OpenRemoteDesktopSessionCommand {
    pub fn new(context: Context, token: String) -> Self {
        Self {
            _context: context,
            _token: token,
        }
    }
}

impl ExecutableCommand for OpenRemoteDesktopSessionCommand {
    async fn execute(self) -> Result<(), nullnet_liberror::Error> {
        log::debug!("Received OpenRemoteDesktopSessionCommand");

        todo!()
    }
}
