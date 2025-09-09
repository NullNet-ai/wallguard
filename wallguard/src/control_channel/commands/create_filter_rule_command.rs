use crate::{context::Context, control_channel::command::ExecutableCommand, fireparse::Fireparse};
use nullnet_liberror::Error;
use wallguard_common::protobuf::wallguard_models::FilterRule;

pub struct CreateFilterRuleCommand {
    rule: FilterRule,
    context: Context,
}

impl CreateFilterRuleCommand {
    pub fn new(rule: FilterRule, context: Context) -> Self {
        Self { rule, context }
    }
}

impl ExecutableCommand for CreateFilterRuleCommand {
    async fn execute(self) -> Result<(), Error> {
        Fireparse::create_filter_rule(self.rule, self.context.client_data.platform).await
    }
}
