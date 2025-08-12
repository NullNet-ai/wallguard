use crate::{
    client_data::Platform, context::Context, control_channel::command::ExecutableCommand,
    fireparse::Fireparse,
};
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use tokio::process::Command;
use wallguard_common::protobuf::wallguard_models::FilterRule;
use xmltree::Element;
// use xmltree::{Element, XMLNode};

pub struct CreateFirewallRuleCommand {
    rule: FilterRule,
    context: Context,
}

impl CreateFirewallRuleCommand {
    pub fn new(rule: FilterRule, context: Context) -> Self {
        Self { rule, context }
    }

    async fn reload_configuraion() -> Result<(), Error> {
        let status = Command::new("configctl")
            .arg("system")
            .arg("reload")
            .status()
            .await
            .handle_err(location!())?;

        if !status.success() {
            Err(format!("configctl failed with status: {}", status)).handle_err(location!())
        } else {
            println!("pfSense config successfully reloaded.");
            Ok(())
        }
    }
}

impl ExecutableCommand for CreateFirewallRuleCommand {
    async fn execute(self) -> Result<(), Error> {
        let rule = match self.context.client_data.platform {
            Platform::PfSense | Platform::OpnSense => {
                Fireparse::convert_filter_rule(self.rule, self.context.client_data.platform)?
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
            .get_mut_child("rules")
            .ok_or("Malformed config.xml file")
            .handle_err(location!())?;

        rules_node.children.push(xmltree::XMLNode::Element(rule));

        let mut buffer = Vec::new();
        document.write(&mut buffer).handle_err(location!())?;
        tokio::fs::write("/conf/config.xml", buffer)
            .await
            .handle_err(location!())?;

        Self::reload_configuraion().await
    }
}
