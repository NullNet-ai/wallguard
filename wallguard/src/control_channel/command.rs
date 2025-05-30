use nullnet_liberror::Error;

pub trait ExecutableCommand {
    async fn execute(self) -> Result<(), Error>;
}
