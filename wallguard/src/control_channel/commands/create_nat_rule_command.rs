use crate::{
    client_data::Platform, context::Context, control_channel::command::ExecutableCommand,
    fireparse::Fireparse, utilities::system,
};
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use wallguard_common::protobuf::wallguard_models::NatRule;
use xmltree::Element;

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
        let rule = match self.context.client_data.platform {
            Platform::PfSense | Platform::OpnSense => {
                Fireparse::convert_nat_rules(self.rule, self.context.client_data.platform)?
            }
            _ => Err(format!(
                "Current platform {} does not support rules creation",
                self.context.client_data.platform
            ))
            .handle_err(location!())?,
        };

        let content = tokio::fs::read("/conf/config.xml")
            .await
            .handle_err(location!())?;

        let mut document = Element::parse(content.as_slice()).handle_err(location!())?;

        let rules_node = document
            .get_mut_child("nat")
            .ok_or("Malformed config.xml file")
            .handle_err(location!())?;

        rules_node.children.push(xmltree::XMLNode::Element(rule));

        let mut buffer = Vec::new();
        document.write(&mut buffer).handle_err(location!())?;
        tokio::fs::write("/conf/config.xml", buffer)
            .await
            .handle_err(location!())?;

        system::reload_configuraion().await
    }
}
