use crate::datastore::{
    Datastore, Device,
    db_tables::DBTable,
    generated::{
        Accounts, RegisterDeviceParams, RegisterDeviceRequest, UpdateAccountsRequest, UpdateParams,
        UpdateQuery,
    },
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

        let register_response = self
            .inner
            .clone()
            .register_device(grpc_request)
            .await
            .handle_err(location!())?
            .into_inner();

        let data: serde_json::Value =
            serde_json::from_str(&register_response.data).handle_err(location!())?;

        let id = data["account_id"]
            .as_str()
            .map(str::to_string)
            .ok_or("Missing 'account_id' in register_device response")
            .handle_err(location!())?;

        let update_request = UpdateAccountsRequest {
            account: Some(Accounts {
                status: Some("Active".to_string()),
                account_status: Some("Active".to_string()),
                ..Default::default()
            }),
            params: Some(UpdateParams {
                id,
                table: DBTable::Accounts.into(),
                r#type: String::from("root"),
            }),
            query: Some(UpdateQuery {
                pluck: String::new(),
            }),
        };

        let mut grpc_update_request = tonic::Request::new(update_request);
        grpc_update_request.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", token)
                .parse()
                .handle_err(location!())?,
        );

        let _ = self
            .inner
            .clone()
            .update_accounts(grpc_update_request)
            .await
            .handle_err(location!())?;

        Ok(())
    }
}
