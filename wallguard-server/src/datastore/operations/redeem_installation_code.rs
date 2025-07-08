use crate::datastore::builders::UpdateRequestBuilder;
use crate::datastore::{Datastore, InstallationCode};
use nullnet_liberror::Error;
use serde_json::json;

impl Datastore {
    pub async fn redeem_installation_code(
        &self,
        code: &InstallationCode,
        token: &str,
    ) -> Result<(), Error> {
        let update = json!({
            "reedemed": true
        });

        let request = UpdateRequestBuilder::new()
            .performed_by_root(true)
            .id(&code.id)
            .body(update.to_string())
            .table(InstallationCode::table())
            .build();

        let _ = self.inner.clone().update(request, token).await;

        Ok(())
    }
}
