use crate::data_transmission::sysconfig::data::FileToMonitor;
use crate::data_transmission::sysconfig::types::FileData;
use crate::utilities;
use nullnet_liberror::{location, Error, ErrorHandler, Location};

const DEFAULT_PROGRAM: Option<&str> = None;
const DEFAULT_PARAMETERS: &[&str] = &[];

#[derive(Debug, Default, Clone)]
pub struct NftablesRuleset {
    ruleset: String,
}

impl FileToMonitor for NftablesRuleset {
    fn take_snapshot(&self) -> crate::data_transmission::sysconfig::types::FileData {
        FileData {
            filename: "config.xml".into(),
            content: self.ruleset.as_bytes().into(),
        }
    }

    async fn update(&mut self) -> Result<bool, Error> {
        let prev = utilities::hash::sha256_digest_bytes(&self.ruleset);
        let ruleset = tokio::task::spawn_blocking(move || {
            nftables::helper::get_current_ruleset_raw(DEFAULT_PROGRAM, DEFAULT_PARAMETERS)
        })
        .await
        .handle_err(location!())?
        .handle_err(location!())?;

        self.ruleset = ruleset;
        let curr = utilities::hash::sha256_digest_bytes(&self.ruleset);

        Ok(prev != curr)
    }
}
