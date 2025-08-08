use nullnet_liberror::{location, Error, ErrorHandler, Location};
use wallguard_common::protobuf::wallguard_models::Configuration;

use crate::data_transmission::sysconfig::types::FileData;
use crate::{client_data::Platform, fireparse::pfsense::PfSenseParser};

// mod opnsense;
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
            Platform::OpnSense => todo!(),
            Platform::Generic => Err("Unsupported platform").handle_err(location!()),
        }
    }
}
