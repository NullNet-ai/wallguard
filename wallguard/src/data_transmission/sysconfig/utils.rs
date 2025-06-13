use nullnet_liberror::{location, Error, ErrorHandler, Location};
use std::{path::PathBuf, time::SystemTime};
use tokio::fs;

pub async fn get_mtime(path: &PathBuf) -> Result<u128, Error> {
    let value = fs::metadata(path)
        .await
        .handle_err(location!())?
        .modified()
        .handle_err(location!())?
        .duration_since(SystemTime::UNIX_EPOCH)
        .handle_err(location!())?;

    Ok(value.as_millis())
}
