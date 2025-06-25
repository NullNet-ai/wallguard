use crate::{context::Context, control_channel::command::ExecutableCommand};

pub struct EnableConfigurationMonitoringCommand {
    value: bool,
    context: Context,
}

impl EnableConfigurationMonitoringCommand {
    pub fn new(context: Context, value: bool) -> Self {
        Self { value, context }
    }
}

impl ExecutableCommand for EnableConfigurationMonitoringCommand {
    async fn execute(self) -> Result<(), nullnet_liberror::Error> {
        log::debug!(
            "Executing EnableConfigurationMonitoringCommand command: {}",
            self.value
        );

        if self.value {
            self.context
                .transmission_manager
                .lock()
                .await
                .start_sysconf_monitroing();
        } else {
            self.context
                .transmission_manager
                .lock()
                .await
                .terminate_sysconfig_monitoring();
        }

        Ok(())
    }
}
