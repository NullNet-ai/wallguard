use crate::token::Token;
use crate::{control_service::service::WallGuardService, datastore::Device};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl WallGuardService {
    pub(crate) async fn ensure_device_exists_and_authrorized(
        &self,
        token: &Token,
    ) -> Result<Device, Error> {
        let device_id = token
            .account
            .device_id()
            .ok_or("Wrong token type")
            .handle_err(location!())?;

        let device = self
            .context
            .datastore
            .obtain_device_by_id(&token.jwt, device_id, false)
            .await?
            .ok_or("Device does not exists")
            .handle_err(location!())?;

        if !device.authorized {
            return Err("Device is not authrozied").handle_err(location!());
        }

        Ok(device)
    }
}
