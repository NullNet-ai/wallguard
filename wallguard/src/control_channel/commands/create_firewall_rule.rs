use crate::{client_data::Platform, context::Context, control_channel::command::ExecutableCommand};
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use wallguard_common::protobuf::wallguard_models::FilterRule;
// use xmltree::{Element, XMLNode};

pub struct CreateFirewallRuleCommand {
    rule: FilterRule,
    context: Context,
}

impl CreateFirewallRuleCommand {
    pub fn new(rule: FilterRule, context: Context) -> Self {
        Self { rule, context }
    }

    // fn parse_protocol_field(&self) -> Result<(String, String), Error> {
    //     let values: Vec<&str> = self.rule.protocol.split("/").collect();

    //     if values.len() != 2 {
    //         Err(format!(
    //             "Expected value in the format of IP-PROTO/PROTO, can not parse string {}",
    //             self.rule.protocol
    //         ))
    //         .handle_err(location!())?;
    //     }

    //     let ipprotocol = match values[0].to_lowercase().as_str() {
    //         "*" => String::new(),
    //         "ipv4" => String::from("inet"),
    //         "ipv6" => String::from("inet6"),
    //         other => Err(format!("Unexpected iprotocol value {other}")).handle_err(location!())?,
    //     };

    //     let protocol = match values[1].to_ascii_lowercase().as_str() {
    //         "any" => String::new(),
    //         other => String::from(other),
    //     };

    //     Ok((ipprotocol, protocol))
    // }

    async fn create_opnsense_rule(&self) -> Result<(), Error> {
        // let mut element = Element::new("rule");

        // {
        //     let mut node = Element::new("type");
        //     node.children.push(XMLNode::Text(self.rule.policy.clone()));
        //     element.children.push(XMLNode::Element(node));
        // }

        // let (ipprotocol, protocol) = self.parse_protocol_field()?;

        // if !ipprotocol.is_empty() {
        //     let mut node = Element::new("ipprotocol");
        //     node.children.push(XMLNode::Text(ipprotocol));
        //     element.children.push(XMLNode::Element(node));
        // }

        // if !protocol.is_empty() {
        //     let mut node = Element::new("protocol");
        //     node.children.push(XMLNode::Text(protocol));
        //     element.children.push(XMLNode::Element(node));
        // }

        // {
        //     let mut node = Element::new("descr");
        //     node.children
        //         .push(XMLNode::CData(self.rule.description.clone()));
        //     element.children.push(XMLNode::Element(node));
        // }

        // {
        //     let mut node = Element::new("interface");
        //     node.children
        //         .push(XMLNode::CData(self.rule.interface.clone()));
        //     element.children.push(XMLNode::Element(node));
        // }

        // {
        //     // source element
        //     let mut src_node = Element::new("source");

        //     if self.rule.source_inversed {
        //         let node = Element::new("not");
        //         src_node.children.push(XMLNode::Element(node));
        //     }

        //     if self.rule.source_addr == "*" && self.rule.source_port == "*" {
        //         let node = Element::new("any");
        //         src_node.children.push(XMLNode::Element(node));
        //     } else {
        //         if self.rule.source_addr != "*" {
        //             let mut node = Element::new(&self.rule.source_type.to_lowercase());
        //             node.children
        //                 .push(XMLNode::Text(self.rule.source_addr.clone()));
        //         }

        //         if self.rule.source_port != "*" {
        //             let mut node = Element::new("port");
        //             node.children
        //                 .push(XMLNode::Text(self.rule.source_port.clone()));
        //         }
        //     }

        //     element.children.push(XMLNode::Element(src_node));
        // }

        // {
        //     // destination element
        //     let mut dest_node = Element::new("destination");

        //     if self.rule.destination_inversed {
        //         let node = Element::new("not");
        //         dest_node.children.push(XMLNode::Element(node));
        //     }

        //     if self.rule.destination_addr == "*" && self.rule.destination_port == "*" {
        //         let node = Element::new("any");
        //         dest_node.children.push(XMLNode::Element(node));
        //     } else {
        //         if self.rule.destination_addr != "*" {
        //             let mut node = Element::new(&self.rule.destination_type.to_lowercase());
        //             node.children
        //                 .push(XMLNode::Text(self.rule.destination_addr.clone()));
        //         }

        //         if self.rule.destination_port != "*" {
        //             let mut node = Element::new("port");
        //             node.children
        //                 .push(XMLNode::Text(self.rule.destination_port.clone()));
        //         }
        //     }

        //     element.children.push(XMLNode::Element(dest_node));
        // }

        Ok(())
    }

    async fn create_pfsense_rule(&self) -> Result<(), Error> {
        // self.rule.
        todo!()
    }
}

impl ExecutableCommand for CreateFirewallRuleCommand {
    async fn execute(self) -> Result<(), Error> {
        match self.context.client_data.platform {
            Platform::PfSense => self.create_pfsense_rule().await,
            Platform::OpnSense => self.create_opnsense_rule().await,
            _ => Err(format!(
                "Current platform {} does not support rules creation",
                self.context.client_data.platform
            ))
            .handle_err(location!()),
        }
    }
}
