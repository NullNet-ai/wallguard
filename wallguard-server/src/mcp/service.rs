use super::schema::ExecuteCommandParameters;
use crate::app_context::AppContext;
use crate::mcp::config::SERVICE_INSTRUCTIONS;
use crate::utilities::random;
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, Content, ErrorCode, Implementation, ProtocolVersion, ServerCapabilities,
    ServerInfo,
};
use rmcp::{ErrorData, ServerHandler, tool, tool_handler, tool_router};
use std::time::Duration;
use wallguard_common::protobuf::wallguard_commands::ExecuteCliCommandRequest;

#[derive(Clone)]
pub struct MCPService {
    #[allow(dead_code)]
    context: AppContext,
    tool_router: ToolRouter<MCPService>,
}

impl MCPService {
    pub fn new(context: AppContext) -> Self {
        Self {
            context,
            tool_router: Self::tool_router(),
        }
    }

    fn get_device_info(&self) -> (String, String) {
        todo!()
    }
}

#[tool_router]
impl MCPService {
    #[tool(description = "Execute a command on the client machine")]
    async fn execute_command(
        &self,
        Parameters(ExecuteCommandParameters { command, arguments }): Parameters<
            ExecuteCommandParameters,
        >,
    ) -> Result<CallToolResult, ErrorData> {
        let (device_uuid, instance_id) = self.get_device_info();

        let Some(instance) = self
            .context
            .orchestractor
            .get_client(&device_uuid, &instance_id)
            .await
        else {
            return Err(ErrorData {
                code: ErrorCode::INVALID_REQUEST,
                message: "Client is not connected, try again later".into(),
                data: None,
            });
        };

        let request = ExecuteCliCommandRequest {
            command: command.clone(),
            arguments: arguments.clone(),
            request_unique_id: random::generate_random_string(32),
        };

        let timeout = Duration::from_secs(10);

        let response = instance
            .lock()
            .await
            .execute_cli_command(request, timeout)
            .await
            .map_err(|err| ErrorData {
                code: ErrorCode::INTERNAL_ERROR,
                message: format!("Failed to execute command: {}", err.to_str()).into(),
                data: None,
            })?;

        let output = format!(
            "Command `{}` executed with args {:?}\nStatus: {}\nstdout:\n{}\nstderr:\n{}",
            command, arguments, response.status, response.stdout, response.stderr
        );

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }
}

#[tool_handler]
impl ServerHandler for MCPService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(SERVICE_INSTRUCTIONS.into()),
        }
    }
}
