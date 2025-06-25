use crate::datastore::Datastore;
use crate::datastore::builders::{AdvanceFilterBuilder, BatchUpdateRequestBuilder};
use nullnet_liberror::Error;
use serde_json::json;

// THIS CODE IS TO BE REMOVED
// Temporary solution to allow device authentication
// The platform team is currently working on the new device-specific registration RPC
// which will make this step redundant

impl Datastore {
    pub async fn activate_account_organization(
        &self,
        token: &str,
        account_id: &str,
    ) -> Result<(), Error> {
        let updates = json!({"status": "Active"}).to_string();

        let filter = AdvanceFilterBuilder::new()
            .field("email")
            .values(format!("[\"{account_id}\"]"))
            .r#type("criteria")
            .operator("equal")
            .entity("account_organizations")
            .build();

        let request = BatchUpdateRequestBuilder::new()
            .table("account_organizations")
            .advance_filter(filter)
            .updates(updates)
            .performed_by_root(true)
            .build();

        log::warn!("{:?}", request);

        let _ = self.inner.clone().batch_update(request, token).await?;
        Ok(())
    }
}
