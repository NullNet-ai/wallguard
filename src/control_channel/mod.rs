use crate::app_context::AppContext;
use command::ExecutableCommand;
use commands::{HeartbeatCommand, OpenSshSessionCommand, UpdateTokenCommand};
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use nullnet_libwallguard::{Command, SshSessionData};
use tokio::{io::copy_bidirectional, net::TcpStream};

mod command;
mod commands;
#[derive(Clone)]
pub struct ControlChannel {
    context: AppContext,
}

impl ControlChannel {
    pub fn new(context: AppContext) -> Self {
        Self { context }
    }

    pub async fn run(&self) -> Result<(), Error> {
        let mut stream = self
            .context
            .server
            .request_control_channel(
                &self.context.arguments.app_id,
                &self.context.arguments.app_secret,
            )
            .await
            .handle_err(location!())?;

        loop {
            while let Ok(command) = stream.message().await {
                let command = command.and_then(|cmd| cmd.command);

                if command.is_none() {
                    // Most likely the connection has been terminated
                    return Err("Control channel connection issue").handle_err(location!());
                }

                match command.unwrap() {
                    Command::UpdateTokenCommand(token) => {
                        if let Err(err) = UpdateTokenCommand::new(self.context.clone(), token)
                            .execute()
                            .await
                        {
                            log::error!("UpdateTokenCommand execution failed: {}", err.to_str());
                        }
                    }
                    Command::EnableNetworkMonitoringCommand(_) => todo!(),
                    Command::EnableConfigurationMonitoringCommand(_) => todo!(),
                    Command::EnableTelemetryMonitoringCommand(_) => todo!(),
                    Command::OpenSshSessionCommand(data) => {
                        let cmd = OpenSshSessionCommand::new(self.context.clone(), data);
                        if let Err(err) = cmd.execute().await {
                            log::error!("OpenSshSessionCommand execution failed: {}", err.to_str());
                        }
                    }
                    Command::OpenTtySessionCommand(_) => todo!(),
                    Command::OpenUiSessionCommand(_) => todo!(),
                    Command::HeartbeatCommand(_) => {
                        if let Err(err) = HeartbeatCommand::new().execute().await {
                            log::error!("HeartbeatCommand execution failed: {}", err.to_str());
                        }
                    }
                }
            }
        }
    }
}
