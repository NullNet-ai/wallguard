use crate::{datastore::DatastoreWrapper, parser::parsed_message::ParsedMessage};
use nullnet_libdatastore::{
    BatchCreateBody, BatchCreateRequest, CreateParams, Query, ResponseData,
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl DatastoreWrapper {
    pub async fn connections_insert(
        &self,
        token: &str,
        parsed_message: ParsedMessage,
    ) -> Result<ResponseData, Error> {
        let records = serde_json::to_string(&parsed_message).handle_err(location!())?;

        let request = BatchCreateRequest {
            params: Some(CreateParams {
                table: String::from("connections"),
            }),
            query: Some(Query {
                pluck: String::new(),
                durability: String::from("soft"),
            }),
            body: Some(BatchCreateBody {
                records,
                entity_prefix: String::from("PK"),
            }),
        };

        let response = self.inner.clone().batch_create(request, token).await?;

        Ok(response)
    }
}
