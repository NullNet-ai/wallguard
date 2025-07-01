use crate::{control_service::service::WallGuardService, datastore::Device};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use nullnet_libtoken::Token;

impl WallGuardService {
    pub(crate) async fn ensure_device_exists_and_authrorized(
        &self,
        token: &Token,
    ) -> Result<Device, Error> {
        let device = token.account.device.as_ref().ok_or("Wrong token type").handle_err(location!())?;
        
        let device = self
            .context
            .datastore
            .obtain_device_by_id(&token.jwt, &device.id)
            .await?
            .ok_or("Device does not exists")
            .handle_err(location!())?;

        if !device.authorized {
            return Err("Device is not authrozied").handle_err(location!());
        }

        Ok(device)
    }
}
