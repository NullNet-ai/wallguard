pub mod models {
    tonic::include_proto!("wallguard.models");
}

pub mod control {
    tonic::include_proto!("wallguard.control");
}

pub mod cli {
    tonic::include_proto!("wallguard.cli");
}

pub mod data {
    tonic::include_proto!("wallguard.data");
}
