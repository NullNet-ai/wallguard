use crate::client_data::Platform;
use crate::fireparse::Fireparse;
use crate::utilities::system;
use crate::{context::Context, control_channel::command::ExecutableCommand};
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use wallguard_common::protobuf::wallguard_models::Alias;
use xmltree::{Element, XMLNode};
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
        let alias = match self.context.client_data.platform {
            Platform::PfSense | Platform::OpnSense => {
                Fireparse::convert_alias(self.alias, self.context.client_data.platform)?
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

        let aliases_node = match self.context.client_data.platform {
            Platform::PfSense => document
                .get_mut_child("aliases")
                .ok_or("Malformed config.xml")
                .handle_err(location!())?,
            Platform::OpnSense => document
                .get_mut_child("OPNsense")
                .and_then(|el| el.get_mut_child("Firewall"))
                .and_then(|el| el.get_mut_child("Alias"))
                .and_then(|el| el.get_mut_child("aliases"))
                .ok_or("Malformed config.xml")
                .handle_err(location!())?,
            Platform::Generic | Platform::NfTables => {
                Err("Unexpected value").handle_err(location!())?
            }
        };

        aliases_node.children.push(XMLNode::Element(alias));

        let mut buffer = Vec::new();
        document.write(&mut buffer).handle_err(location!())?;
        tokio::fs::write("/conf/config.xml", buffer)
            .await
            .handle_err(location!())?;

        system::reload_configuraion().await
    }
}
