use crate::fireparse::Fireparse;
use crate::{context::Context, control_channel::command::ExecutableCommand};
use nullnet_liberror::Error;
use wallguard_common::protobuf::wallguard_models::Alias;

pub struct CreateAliasCommand {
    alias: Alias,
    context: Context,
}

impl CreateAliasCommand {
    pub fn new(alias: Alias, context: Context) -> Self {
        Self { alias, context }
    }
}

impl ExecutableCommand for CreateAliasCommand {
    async fn execute(self) -> Result<(), Error> {
        Fireparse::create_alias(self.alias, self.context.client_data.platform).await
    }
}
