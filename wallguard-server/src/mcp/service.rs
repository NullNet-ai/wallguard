use std::time::Duration;

use super::schema::ExecuteCommandParameters;
use crate::app_context::AppContext;
use crate::datastore::{Device, RemoteAccessSession, RemoteAccessType};
use crate::mcp::config::SERVICE_INSTRUCTIONS;
use crate::mcp::schema::ObtainDeviceParameters;
use crate::utilities::random;
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler, tool, tool_handler, tool_router};
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

    async fn get_session_details(&self, session: &str) -> Result<RemoteAccessSession, ErrorData> {
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

        let session = self
            .context
            .datastore
            .obtain_session(&token.jwt, session)
            .await
            .map_err(|_| {
                ErrorData::internal_error(
                    "Internal server error: failed to fetch session details",
                    None,
                )
            })?
            .ok_or_else(|| {
                ErrorData::internal_error("Internal server error: session not found", None)
            })?;

        if matches!(session.r#type, RemoteAccessType::Mcp) {
            Ok(session)
        } else {
            Err(ErrorData::invalid_params("Wrong session token type", None))
        }
    }

    async fn get_device_by_id(&self, device_id: &str) -> Result<Device, ErrorData> {
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

        self.context
            .datastore
            .obtain_device_by_id(&token.jwt, device_id, false)
            .await
            .map_err(|_| {
                ErrorData::internal_error(
                    "Internal server error: failed to fetch device details",
                    None,
                )
            })?
            .ok_or_else(|| {
                ErrorData::internal_error("Internal server error: device not found", None)
            })
    }
}

#[tool_router]
impl MCPService {
    #[tool(description = "Execute a CLI command on the remove device")]
    async fn execute_command(
        &self,
        Parameters(ExecuteCommandParameters {
            command,
            arguments,
            session,
        }): Parameters<ExecuteCommandParameters>,
        _: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let session = self.get_session_details(&session).await?;
        let device = self.get_device_by_id(&session.device_id).await?;

        let Some(instance) = self
            .context
            .orchestractor
            .get_client(&device.id, &session.instance_id)
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

    #[tool(description = "Obtain information about the device")]
    async fn obtain_device_info(
        &self,
        Parameters(ObtainDeviceParameters { session }): Parameters<ObtainDeviceParameters>,
    ) -> Result<CallToolResult, ErrorData> {
        let session = self.get_session_details(&session).await?;
        let device = self.get_device_by_id(&session.device_id).await?;

        let output = format!(
            "
        Device name: {}
        Device type: {}
        Device category: {}
        Device authorized: {}
        Online: {}
        Operating System: {}

        Configuration monitoring enabled: {}
        Telemetry data monitoring enabled: {}
        Traffic monitoring data enabled: {}
        ",
            device.name,
            device.r#type,
            device.category,
            device.authorized,
            device.online,
            device.os,
            device.sysconf_monitoring,
            device.telemetry_monitoring,
            device.traffic_monitoring
        );

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }
}

#[tool_handler]
impl ServerHandler for MCPService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(SERVICE_INSTRUCTIONS.into()),
        }
    }
}
