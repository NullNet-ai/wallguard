use crate::{context::Context, control_channel::command::ExecutableCommand, fireparse::Fireparse};
use nullnet_liberror::Error;
use wallguard_common::protobuf::wallguard_models::NatRule;

pub struct CreateNatRuleCommand {
    rule: NatRule,
    context: Context,
}

impl CreateNatRuleCommand {
    pub fn new(rule: NatRule, context: Context) -> Self {
        Self { rule, context }
    }
}

impl ExecutableCommand for CreateNatRuleCommand {
    async fn execute(self) -> Result<(), Error> {
        Fireparse::create_nat_rule(self.rule, self.context.client_data.platform).await
    }
}
