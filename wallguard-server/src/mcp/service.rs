use std::time::Duration;

use super::schema::ExecuteCommandParameters;
use crate::app_context::AppContext;
use crate::datastore::Device;
use crate::mcp::config::SERVICE_INSTRUCTIONS;
use crate::utilities::random;
use axum::http::request::Parts;
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    AnnotateAble, CallToolResult, Content, ErrorCode, Implementation, ListResourcesResult,
    PaginatedRequestParam, ProtocolVersion, RawResource, ReadResourceRequestParam,
    ReadResourceResult, ResourceContents, ServerCapabilities, ServerInfo,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler, tool, tool_handler, tool_router};
use serde_json::json;
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

    async fn get_device_info(
        &self,
        context: &RequestContext<RoleServer>,
    ) -> Result<(Device, String), ErrorData> {
        let Some(mcp_session_id) = context
            .extensions
            .get::<Parts>()
            .cloned()
            .and_then(|parts| parts.headers.get("mcp-session-id").cloned())
            .and_then(|value| value.to_str().map(|v| v.to_string()).ok())
        else {
            return Err(ErrorData::internal_error(
                "MCP session id is undefined",
                None,
            ));
        };

        let session = self
            .context
            .mcp_sessions
            .lock()
            .await
            .get(&mcp_session_id)
            .cloned()
            .ok_or(ErrorData {
                code: ErrorCode::INVALID_REQUEST,
                message: "MCP session id is undefined".into(),
                data: None,
            })?;

        let token = self
            .context
            .sysdev_token_provider
            .get()
            .await
            .map_err(|_| {
                ErrorData::internal_error(
                    "Internal server error: failed to fetch datastore token",
                    None,
                )
            })?;

        let device = self
            .context
            .datastore
            .obtain_device_by_id(&token.jwt, &session.device_id, false)
            .await
            .map_err(|err| {
                ErrorData::internal_error(
                    format!("Failed to fetch client data: {}", err.to_str()),
                    None,
                )
            })?
            .ok_or(ErrorData::internal_error("Clint not found", None))?;

        Ok((device, session.instance_id))
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
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let (device, instance_id) = self.get_device_info(&context).await?;

        let Some(instance) = self
            .context
            .orchestractor
            .get_client(&device.uuid, &instance_id)
            .await
        else {
            return Err(ErrorData::internal_error(
                "Client is not connected, try again later",
                None,
            ));
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
            .map_err(|err| {
                ErrorData::internal_error(
                    format!("Failed to execute command: {}", err.to_str()),
                    None,
                )
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

    async fn list_resources(
        &self,
        _: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        Ok(ListResourcesResult {
            next_cursor: None,
            resources: vec![
                RawResource::new("json://client_details", "Client Details").no_annotation(),
            ],
        })
    }

    async fn read_resource(
        &self,
        ReadResourceRequestParam { uri }: ReadResourceRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, ErrorData> {
        let (device, _) = self.get_device_info(&context).await?;

        match uri.as_str() {
            "json://client_details" => Ok(ReadResourceResult {
                contents: vec![ResourceContents::text(json!(device).to_string(), uri)],
            }),
            _ => Err(ErrorData::resource_not_found(
                "Resource not found",
                Some(json!({ "uri": uri })),
            )),
        }
    }
}
