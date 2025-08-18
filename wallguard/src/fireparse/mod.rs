use nullnet_liberror::{location, Error, ErrorHandler, Location};
use wallguard_common::protobuf::wallguard_models::{Alias, Configuration, FilterRule, NatRule};
use xmltree::Element;

use crate::data_transmission::sysconfig::types::FileData;
use crate::fireparse::opnsense::OpnSenseParser;
use crate::{client_data::Platform, fireparse::pfsense::PfSenseParser};

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
            Platform::Generic => Err("Unsupported platform").handle_err(location!()),
        }
    }

    pub fn convert_filter_rule(rule: FilterRule, platform: Platform) -> Result<Element, Error> {
        match platform {
            Platform::PfSense => Ok(PfSenseParser::convert_filter_rule(rule)),
            Platform::OpnSense => Ok(OpnSenseParser::convert_filter_rule(rule)),
            Platform::Generic => Err("Not supported").handle_err(location!()),
        }
    }

    pub fn convert_nat_rules(rule: NatRule, platform: Platform) -> Result<Element, Error> {
        match platform {
            Platform::PfSense => Ok(PfSenseParser::convert_nat_rule(rule)),
            Platform::OpnSense => Ok(OpnSenseParser::convert_nat_rule(rule)),
            Platform::Generic => Err("Not supported").handle_err(location!()),
        }
    }

    pub fn convert_alias(alias: Alias, platform: Platform) -> Result<Element, Error> {
        match platform {
            Platform::PfSense => Ok(PfSenseParser::convert_alias(alias)),
            Platform::OpnSense => Ok(OpnSenseParser::convert_alias(alias)),
            Platform::Generic => Err("Not supported").handle_err(location!()),
        }
    }
}
