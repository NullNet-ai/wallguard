use rmcp::schemars;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ExecuteCommandParameters {
    pub(super) command: String,
    pub(super) arguments: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ExecuteCommandError {
    pub(super) command: String,
    pub(super) arguments: Vec<String>,
    pub(super) error: String,
}
