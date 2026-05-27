use nftables::schema::Nftables;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use wallguard_common::protobuf::wallguard_models::{Alias, Configuration, FilterRule, NatRule};

use crate::data_transmission::sysconfig::types::FileData;
use crate::fireparse::nft::NftablesParser;
use crate::fireparse::opnsense::OpnSenseParser;
use crate::{client_data::Platform, fireparse::pfsense::PfSenseParser};

mod nft;
mod opnsense;
mod pfsense;

pub struct Fireparse {}

impl Fireparse {
    pub fn parse(files: Vec<FileData>, platform: Platform) -> Result<Configuration, Error> {
        match platform {
            Platform::PfSense => {
                let config_file = files
                    .into_iter()
                    .find(|file| file.filename == "config.xml")
                    .ok_or("'Config.xml' not found")
                    .handle_err(location!())?;

                let data = String::from_utf8(config_file.content).handle_err(location!())?;

                PfSenseParser::parse(&data)
            }
            Platform::OpnSense => {
                let config_file = files
                    .into_iter()
                    .find(|file| file.filename == "config.xml")
                    .ok_or("'Config.xml' not found")
                    .handle_err(location!())?;

                let data = String::from_utf8(config_file.content).handle_err(location!())?;

                OpnSenseParser::parse(&data)
            }
            Platform::NfTables => {
                let ruleset = files
                    .into_iter()
                    .find(|file| file.filename == "#NFRuleset")
                    .ok_or("'#NFRuleset' not found")
                    .handle_err(location!())?;

                let tables: Nftables<'_> =
                    serde_json::from_slice(&ruleset.content).handle_err(location!())?;

                NftablesParser::parse(
                    tables,
                    format!("{:x}", md5::compute(ruleset.content.as_slice())),
                )
            }
            Platform::Generic => Err("Unsupported platform").handle_err(location!()),
        }
    }

    pub async fn create_filter_rule(rule: FilterRule, platform: Platform) -> Result<(), Error> {
        match platform {
            Platform::Generic => Err("Unsupported platform").handle_err(location!()),
            Platform::PfSense => PfSenseParser::create_filter_rule(rule).await,
            Platform::OpnSense => OpnSenseParser::create_filter_rule(rule).await,
            Platform::NfTables => NftablesParser::create_filter_rule(rule).await,
        }
    }

    pub async fn create_nat_rule(rule: NatRule, platform: Platform) -> Result<(), Error> {
        match platform {
            Platform::Generic => Err("Unsupported platform").handle_err(location!()),
            Platform::PfSense => PfSenseParser::create_nat_rule(rule).await,
            Platform::OpnSense => OpnSenseParser::create_nat_rule(rule).await,
            Platform::NfTables => NftablesParser::create_nat_rule(rule).await,
        }
    }

    pub async fn create_alias(alias: Alias, platform: Platform) -> Result<(), Error> {
        match platform {
            Platform::Generic => Err("Unsupported platform").handle_err(location!()),
            Platform::PfSense => PfSenseParser::create_alias(alias).await,
            Platform::OpnSense => OpnSenseParser::create_alias(alias).await,
            Platform::NfTables => NftablesParser::create_alias(alias).await,
        }
    }
}
