use crate::{context::Context, control_channel::command::ExecutableCommand};

pub struct EnableTelemetryMonitoringCommand {
    value: bool,
    context: Context,
}

impl EnableTelemetryMonitoringCommand {
    pub fn new(context: Context, value: bool) -> Self {
        Self { value, context }
    }
}

impl ExecutableCommand for EnableTelemetryMonitoringCommand {
    async fn execute(mut self) -> Result<(), nullnet_liberror::Error> {
        log::debug!(
            "Executing EnableTelemetryMonitoringCommand command: {}",
            self.value
        );

        if self.value {
            self.context
                .transmission_manager
                .start_resource_monitoring();
        } else {
            self.context
                .transmission_manager
                .terminate_resource_monitoring();
        }

        Ok(())
    }
}
