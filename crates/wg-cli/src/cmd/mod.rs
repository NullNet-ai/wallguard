pub mod enroll;
pub mod status;
pub mod upgrade;
pub mod autostart;

pub(crate) mod proto {
    pub mod models {
        tonic::include_proto!("wallguard.models");
    }
    pub mod control {
        tonic::include_proto!("wallguard.control");
    }
    pub mod cli {
        tonic::include_proto!("wallguard.cli");
    }
}
