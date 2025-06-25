use crate::{context::Context, control_channel::command::ExecutableCommand};

pub struct EnableNetworkMonitoringCommand {
    value: bool,
    context: Context,
}

impl EnableNetworkMonitoringCommand {
    pub fn new(context: Context, value: bool) -> Self {
        Self { value, context }
    }
}

impl ExecutableCommand for EnableNetworkMonitoringCommand {
    async fn execute(self) -> Result<(), nullnet_liberror::Error> {
        log::debug!(
            "Executing EnableNetworkMonitoringCommand command: {}",
            self.value
        );

        if self.value {
            self.context
                .transmission_manager
                .lock()
                .await
                .start_packet_capture();
        } else {
            self.context
                .transmission_manager
                .lock()
                .await
                .terminate_packet_capture();
        }

        Ok(())
    }
}
