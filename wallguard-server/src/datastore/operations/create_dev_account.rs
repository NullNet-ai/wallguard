use crate::datastore::Datastore;
use crate::datastore::builders::CreateRequestBuilder;
use crate::datastore::db_tables::DBTable;
use crate::utilities;

use nullnet_liberror::{Error, ErrorHandler, Location, location};
use nullnet_libtoken::Token;
use serde_json::json;

impl Datastore {
    pub async fn create_dev_account(
        &self,
        token: &str,
        app_id: &str,
        app_secret: &str,
    ) -> Result<String, Error> {
        let token = Token::from_jwt(token).handle_err(location!())?;

        let record: serde_json::Value = json!({
            "account_id": app_id,
            "account_secret": utilities::hash::hash_secret(app_secret).unwrap(),
            "organization_id": &token.account.organization_id,
            "categories": vec!["Device"],
        });

        let request = CreateRequestBuilder::new()
            .table(DBTable::Accounts)
            .record(record.to_string())
            .pluck(vec!["id"])
            .build();

        let response = self.inner.clone().create(request, &token.jwt).await?;

        if response.count == 1 {
            let json_data = utilities::json::parse_string(&response.data)?;
            let data = utilities::json::first_element_from_array(&json_data)?;

            let id = data["id"]
                .as_str()
                .ok_or("Missing or invalid 'id' field")
                .handle_err(location!())?
                .to_string();

            Ok(id)
        } else {
            Err("Failed to create device account").handle_err(location!())
        }
    }
}
