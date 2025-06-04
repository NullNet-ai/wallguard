use crate::context::Context;
use command::ExecutableCommand;
use commands::{
    HeartbeatCommand, OpenSshSessionCommand, OpenTtySessionCommand, OpenUiSessionCommand,
    UpdateTokenCommand,
};
use nullnet_liberror::{location, Error, ErrorHandler, Location};
use nullnet_libwallguard::Command;

mod command;
mod commands;

#[derive(Clone)]
pub struct ControlChannel {
    context: Context,
}

impl ControlChannel {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn run(&self) -> Result<(), Error> {
        let mut stream = self
            .context
            .server
            .request_control_channel(todo!(), todo!())
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
                    Command::OpenTtySessionCommand(token) => {
                        let cmd = OpenTtySessionCommand::new(self.context.clone(), token);
                        if let Err(err) = cmd.execute().await {
                            log::error!("OpenTtySessionCommand execution failed: {}", err.to_str());
                        }
                    }
                    Command::OpenUiSessionCommand(data) => {
                        let cmd = OpenUiSessionCommand::new(self.context.clone(), data);

                        if let Err(err) = cmd.execute().await {
                            log::error!("OpenUiSessionCommand execution failed: {}", err.to_str());
                        }
                    }
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
