// Shared proto module — visible to all modules in wg-server.
//
// wallguard.control.rs references super::models::*, so `models` must be a
// sibling of `control` inside the same parent module.

pub mod models {
    tonic::include_proto!("wallguard.models");
}
pub mod control {
    tonic::include_proto!("wallguard.control");
}
