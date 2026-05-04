use crate::datastore::{
    Datastore, InstallationCode,
    db_tables::DBTable,
    generated::{InstallationCodes, UpdateInstallationCodesRequest, UpdateParams, UpdateQuery},
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn redeem_installation_code(
        &self,
        code: &InstallationCode,
        token: &str,
    ) -> Result<(), Error> {
        let request = UpdateInstallationCodesRequest {
            installation_code: Some(InstallationCodes {
                redeemed: Some(true),
                ..Default::default()
            }),
            params: Some(UpdateParams {
                id: code.id.clone(),
                table: DBTable::InstallationCodes.into(),
                r#type: "root".to_string(),
            }),
            query: Some(UpdateQuery {
                pluck: String::new(),
            }),
        };

        let _ = self
            .inner
            .clone()
            .update_installation_codes(request)
            .await
            .handle_err(location!())?;

        Ok(())
    }
}
