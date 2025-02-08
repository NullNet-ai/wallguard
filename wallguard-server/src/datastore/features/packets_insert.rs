use crate::{datastore::DatastoreWrapper, parser::parsed_message::ParsedMessage};
use nullnet_libdatastore::{
    BatchCreateBody, BatchCreateRequest, CreateParams, Error as DSError, ErrorKind as DSErrorKind,
    Query, Response as DSResponse,
};
use tonic::Request;

impl DatastoreWrapper {
    pub async fn packets_insert(
        &self,
        token: &str,
        parsed_message: ParsedMessage,
    ) -> Result<DSResponse, DSError> {
        let records = serde_json::to_string(&parsed_message).map_err(|e| DSError {
            kind: DSErrorKind::ErrorRequestFailed,
            message: e.to_string(),
        })?;

        let mut request = Request::new(BatchCreateRequest {
            params: Some(CreateParams {
                table: String::from("packets"),
            }),
            query: Some(Query {
                pluck: String::new(),
                durability: String::from("soft"),
            }),
            body: Some(BatchCreateBody {
                records,
                entity_prefix: String::from("PK"),
            }),
        });

        Self::set_token_for_request(&mut request, token)?;

        let response = self.inner.batch_create(request).await?;

        Ok(response)
    }
}
