use crate::fireparse::{
    Configuration, FireparseError,
    utils::{self, find_in_snapshot},
};
use aliases_parser::AliasesParser;
use hostname_parser::PfSenseHostnameParser;
use interfaces_parser::PfSenseInterfacesParser;
use roxmltree::Document;
use rules_parser::PfSenseRulesParser;
use ssh_parser::PfSenseSSHParser;
use wallguard_common::{
    interface_snapshot::InterfaceSnapshot, protobuf::wallguard_service::FileSnapshot,
};
use webgui_parser::PfSenseWebGuiParser;

mod aliases_parser;
mod endpoint_parser;
mod hostname_parser;
mod interfaces_parser;
mod rules_parser;
mod ssh_parser;
mod webgui_parser;

/// A parser for extracting configuration details from a pfSense XML configuration.
pub struct PfSenseParser {}

impl PfSenseParser {
    /// Parses a pfSense configuration snapshot and extracts firewall settings.
    ///
    /// # Arguments
    /// * `snapshot` - A Snapshot containing pfSense configuration and network interface details.
    ///
    /// # Returns
    /// * `Ok(Configuration)` - A `Configuration` struct
    /// * `Err(FireparseError)` - If any part of the parsing process fails.
    pub fn parse(snapshot: Vec<FileSnapshot>) -> Result<Configuration, FireparseError> {
        let (xmltree, document_encoded) = PfSenseParser::parse_config_from_snapshot(&snapshot)?;
        let iterfaces = PfSenseParser::parse_interfaces_info_from_snapshot(&snapshot)?;

        Ok(Configuration {
            raw_content: document_encoded,
            aliases: AliasesParser::parse(&xmltree),
            rules: PfSenseRulesParser::parse(&xmltree),
            interfaces: PfSenseInterfacesParser::parse(&xmltree, iterfaces),
            hostname: PfSenseHostnameParser::parse(&xmltree),
            gui_protocol: PfSenseWebGuiParser::parse(&xmltree, "https"),
            ssh: PfSenseSSHParser::parse(&xmltree),
        })
    }

    /// Extracts and parses `config.xml` from the snapshot.
    fn parse_config_from_snapshot(
        snapshot: &[FileSnapshot],
    ) -> Result<(Document, String), FireparseError> {
        let pfsense_config =
            find_in_snapshot(snapshot, "config.xml").ok_or(FireparseError::ParserError(
                String::from("PfSenseParser: 'config.xml' file is missing in the snapshot"),
            ))?;

        let config_content = std::str::from_utf8(&pfsense_config.contents).map_err(|e| {
            FireparseError::ParserError(format!(
                "PfSenseParser: Failed to parse 'config.xml' blob as UTF-8: {e}"
            ))
        })?;

        let xmltree = Document::parse(config_content)
            .map_err(|e| FireparseError::ParserError(e.to_string()))?;

        let document_encoded = utils::encode_base64(config_content.as_bytes());

        Ok((xmltree, document_encoded))
    }

    /// Extracts and deserializes network interface information from the snapshot.
    fn parse_interfaces_info_from_snapshot(
        snapshot: &[FileSnapshot],
    ) -> Result<Vec<InterfaceSnapshot>, FireparseError> {
        let ifaces_data =
            find_in_snapshot(snapshot, "#NetworkInterfaces").ok_or(FireparseError::ParserError(
                String::from("PfSenseParser: '#NetworkInterfaces' file is missing in the snapshot"),
            ))?;

        InterfaceSnapshot::deserialize_snapshot(&ifaces_data.contents)
            .map_err(|e| FireparseError::ParserError(format!("PfSenseParser: Failed to deserialize network interfaces data from the snapshot. {e}")))
    }
}
