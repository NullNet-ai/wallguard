use nullnet_liberror::{Error, ErrorHandler, Location, location};
use tokio::process::Command;

pub async fn reload_configuraion() -> Result<(), Error> {
    let status = Command::new("php")
        .arg("-f")
        .arg("/etc/rc.filter_configure")
        .status()
        .await
        .handle_err(location!())?;

    if !status.success() {
        Err(format!(
            "'php -f /etc/rc.filter_configure' failed with status: {status}"
        ))
        .handle_err(location!())
    } else {
        Ok(())
    }
}
