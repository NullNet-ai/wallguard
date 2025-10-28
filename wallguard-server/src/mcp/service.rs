use super::schema::{ExecuteCommandError, ExecuteCommandParameters};
use crate::app_context::AppContext;
use crate::mcp::config::SERVICE_INSTRUCTIONS;
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, Content, ErrorCode, Implementation, ProtocolVersion, ServerCapabilities,
    ServerInfo,
};
use rmcp::{ErrorData, ServerHandler, tool, tool_handler, tool_router};
use std::process::Stdio;
use tokio::{io::AsyncReadExt, process::Command};

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
        if command.trim().is_empty() {
            return Err(ErrorData {
                code: ErrorCode::INVALID_REQUEST,
                message: "Command was empty".into(),
                data: None,
            });
        }

        let mut child = Command::new(&command)
            .args(&arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| ErrorData {
                code: ErrorCode::INTERNAL_ERROR,
                message: format!("Failed to spawn command `{}`: {}", command, err).into(),
                data: None,
            })?;

        let mut stdout = String::new();
        if let Some(mut out_pipe) = child.stdout.take() {
            out_pipe
                .read_to_string(&mut stdout)
                .await
                .map_err(|e| ErrorData {
                    code: ErrorCode::INTERNAL_ERROR,
                    message: format!("Failed to read stdout: {}", e).into(),
                    data: None,
                })?;
        }

        let mut stderr = String::new();
        if let Some(mut err_pipe) = child.stderr.take() {
            err_pipe
                .read_to_string(&mut stderr)
                .await
                .map_err(|e| ErrorData {
                    code: ErrorCode::INTERNAL_ERROR,
                    message: format!("Failed to read stderr: {}", e).into(),
                    data: None,
                })?;
        }

        let status = child.wait().await.map_err(|e| ErrorData {
            code: ErrorCode::INTERNAL_ERROR,
            message: format!("Failed to wait on command: {}", e).into(),
            data: None,
        })?;

        if status.success() {
            let output = format!(
                "Command `{}` executed with args {:?}\nstdout:\n{}\nstderr:\n{}",
                command, arguments, stdout, stderr
            );

            Ok(CallToolResult::success(vec![Content::text(output)]))
        } else {
            let err_info = ExecuteCommandError {
                command: command.clone(),
                arguments: arguments.clone(),
                error: format!("Exit status: {}", status),
            };
            let err_json = serde_json::to_string_pretty(&err_info)
                .unwrap_or_else(|_| "<failed to serialize error>".to_string());

            Err(ErrorData {
                code: ErrorCode::INTERNAL_ERROR,
                message: format!("Command `{}` failed. See data for details.", command).into(),
                data: Some(err_json.into()),
            })
        }
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
