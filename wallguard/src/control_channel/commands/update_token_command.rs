use crate::{context::Context, control_channel::command::ExecutableCommand};

pub struct UpdateTokenCommand {
    context: Context,
    token: String,
}

impl UpdateTokenCommand {
    pub fn new(context: Context, token: String) -> Self {
        Self { context, token }
    }
}

impl ExecutableCommand for UpdateTokenCommand {
    async fn execute(self) -> Result<(), nullnet_liberror::Error> {
        log::debug!("Received UpdateTokenCommand");
        self.context.token_provider.update(self.token).await;
        Ok(())
    }
}
