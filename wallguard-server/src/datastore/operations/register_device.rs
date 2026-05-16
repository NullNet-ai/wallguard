use crate::datastore::{
    Datastore, Device,
    generated::{RegisterDeviceParams, RegisterDeviceRequest},
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn register_device(
        &self,
        token: &str,
        account_id: &str,
        account_secret: &str,
        device: &Device,
    ) -> Result<(), Error> {
        let request = RegisterDeviceRequest {
            device: Some(RegisterDeviceParams {
                account_id: account_id.to_string(),
                account_secret: account_secret.to_string(),
                account_organization_status: "Active".to_string(),
                is_new_user: true,
                account_organization_categories: vec!["Device".to_string()],
                device_categories: vec!["Device".to_string()],
                organization_id: device.organization.clone(),
                device_id: device.id.clone(),
                ..Default::default()
            }),
        };

        let mut grpc_request = tonic::Request::new(request);
        grpc_request.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", token)
                .parse()
                .handle_err(location!())?,
        );

        let _ = self
            .inner
            .clone()
            .register_device(grpc_request)
            .await
            .handle_err(location!())?
            .into_inner();

        Ok(())
    }
}
