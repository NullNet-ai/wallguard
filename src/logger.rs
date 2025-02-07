use log::{Level, LevelFilter};
use syslog::{BasicLogger, Facility, Formatter3164};

pub struct Logger;

impl Logger {
    pub fn init() -> Result<(), Box<dyn std::error::Error>> {
        let formatter = Formatter3164 {
            facility: Facility::LOG_USER,
            hostname: None,
            process: "wallguard".into(),
            pid: std::process::id(),
        };

        let logger = syslog::unix(formatter)?;

        log::set_boxed_logger(Box::new(BasicLogger::new(logger)))?;
        log::set_max_level(LevelFilter::Info);

        Ok(())
    }

    pub fn log<S: AsRef<str>>(level: Level, message: S) {
        log::log!(level, "{}", message.as_ref());

        #[cfg(debug_assertions)]
        println!("{} : {}", level, message.as_ref());
    }
}
