use crate::fireparse::{
    Configuration, FireparseError, nftables::token_stream::TokenStream, utils::find_in_snapshot,
};
use wallguard_common::protobuf::wallguard_service::FileSnapshot;

mod nft;
mod token_stream;

pub struct NfTablesParser {}

impl NfTablesParser {
    pub fn parse(snapshot: Vec<FileSnapshot>) -> Result<Configuration, FireparseError> {
        let ruleset =
            find_in_snapshot(&snapshot, "#Ruleset").ok_or(FireparseError::ParserError(
                String::from("NfTablesParser: '#Ruleset' file is missing in the snapshot"),
            ))?;

        let ruleset_content = std::str::from_utf8(&ruleset.contents).map_err(|e| {
            FireparseError::ParserError(format!(
                "PfSenseParser: Failed to parse 'config.xml' blob as UTF-8: {e}"
            ))
        })?;
        let _ = TokenStream::from(ruleset_content);

        todo!()
    }
}
