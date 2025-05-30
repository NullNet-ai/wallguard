use crate::control_channel::command::ExecutableCommand;
use nullnet_liberror::Error;

pub struct HeartbeatCommand;

impl ExecutableCommand for HeartbeatCommand {
    async fn execute(self) -> Result<(), Error> {
        log::debug!("Received HeartbeatCommand");
        Ok(())
    }
}

impl HeartbeatCommand {
    pub fn new() -> Self {
        Self {}
    }
}
