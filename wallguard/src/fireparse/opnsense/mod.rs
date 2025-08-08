use roxmltree::Document;

mod aliases_parser;
mod enpoint_parser;
mod interfaces_parser;
mod rules_parser;
mod ssh_parser;
mod webgui_parser;

use crate::{
    opnsense::{
        aliases_parser::OpnSenseAliasesParser, interfaces_parser::OpnSenseInterfacesParser,
        rules_parser::OpnSenseRulesParser, ssh_parser::OpnSenseSSHParser,
        webgui_parser::OpnSenseWebGuiParser,
    },
    utils::{self, find_in_snapshot},
    Configuration, FireparseError,
};

pub struct OpnSenseParser {}

impl OpnSenseParser {
    pub fn parse(snapshot: Snapshot) -> Result<Configuration, FireparseError> {
        let (document, encoded) = OpnSenseParser::parse_config_from_snapshot(&snapshot)?;
        let iterfaces = OpnSenseParser::parse_interfaces_info_from_snapshot(&snapshot)?;

        Ok(Configuration {
            rules: OpnSenseRulesParser::parse(&document),
            aliases: OpnSenseAliasesParser::parse(&document),
            interfaces: OpnSenseInterfacesParser::parse(&document, iterfaces),
            hostname: Default::default(),
            ssh: OpnSenseSSHParser::parse(&document),
            gui_protocol: OpnSenseWebGuiParser::parse(&document, "https"),
            raw_content: encoded,
        })
    }

    fn parse_config_from_snapshot(
        snapshot: &Snapshot,
    ) -> Result<(Document, String), FireparseError> {
        let pfsense_config =
            find_in_snapshot(snapshot, "config.xml").ok_or(FireparseError::ParserError(
                String::from("OpnSenseParser: 'config.xml' file is missing in the snapshot"),
            ))?;

        let config_content = std::str::from_utf8(&pfsense_config.content).map_err(|e| {
            FireparseError::ParserError(format!(
                "OpnSenseParser: Failed to parse 'config.xml' blob as UTF-8: {e}"
            ))
        })?;

        let xmltree = Document::parse(config_content)
            .map_err(|e| FireparseError::ParserError(e.to_string()))?;

        let document_encoded = utils::encode_base64(config_content.as_bytes());

        Ok((xmltree, document_encoded))
    }

    fn parse_interfaces_info_from_snapshot(
        snapshot: &Snapshot,
    ) -> Result<Vec<InterfaceSnapshot>, FireparseError> {
        let ifaces_data = find_in_snapshot(snapshot, "#NetworkInterfaces").ok_or(
            FireparseError::ParserError(String::from(
                "OpnSenseParser: '#NetworkInterfaces' file is missing in the snapshot",
            )),
        )?;

        InterfaceSnapshot::deserialize_snapshot(&ifaces_data.content)
            .map_err(|e| FireparseError::ParserError(format!("OpnSenseParser: Failed to deserialize network interfaces data from the snapshot. {e}")))
    }
}
