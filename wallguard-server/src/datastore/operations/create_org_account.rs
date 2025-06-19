use crate::datastore::Datastore;
use crate::datastore::builders::CreateRequestBuilder;
use crate::datastore::db_tables::DBTable;
use crate::utilities;

use nullnet_liberror::{Error, ErrorHandler, Location, location};
use nullnet_libtoken::Token;
use serde_json::json;

impl Datastore {
    pub async fn create_org_account(
        &self,
        token: &str,
        device_id: &str,
        app_id: &str,
        app_secret: &str,
    ) -> Result<(), Error> {
        let token = Token::from_jwt(token).handle_err(location!())?;

        let record: serde_json::Value = json!({
            "account_id": app_id,
            "account_secret": utilities::hash::hash_secret(app_secret).unwrap(),
            "organization_id": &token.account.organization_id,
            "categories": vec!["Device"],
            "device_id": device_id,
        });

        let request = CreateRequestBuilder::new()
            .table(DBTable::OgranizationAccounts)
            .record(record.to_string())
            .build();

        let _ = self.inner.clone().create(request, &token.jwt).await?;

        Ok(())
    }
}
