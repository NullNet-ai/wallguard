use rmcp::schemars;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ExecuteCommandParameters {
    pub(super) command: String,
    pub(super) arguments: Vec<String>,
    pub(super) session: String,
}


#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ObtainDeviceParameters {
    pub(super) session: String,
}
