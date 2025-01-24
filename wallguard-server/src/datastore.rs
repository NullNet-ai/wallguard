use crate::parser::parsed_message::ParsedMessage;
use libdatastore::{
    BatchCreateBody, BatchCreateRequest, CreateParams, DatastoreClient, DatastoreConfig,
    Error as DSError, ErrorKind as DSErrorKind, LoginBody, LoginData, LoginRequest, Query,
    Response as DSResponse,
};
use std::str::FromStr;
use tonic::{metadata::MetadataValue, Request};

#[derive(Debug, Clone)]
pub struct DatastoreWrapper {
    inner: DatastoreClient,
}

impl DatastoreWrapper {
    pub fn new() -> Self {
        let config = DatastoreConfig::from_env();
        let inner = DatastoreClient::new(config);
        Self { inner }
    }

    pub fn set_token_for_request<T>(
        request: &mut Request<T>,
        token: String,
    ) -> Result<(), DSError> {
        let value = MetadataValue::from_str(token.as_str()).map_err(|e| DSError {
            kind: DSErrorKind::ErrorRequestFailed,
            message: e.to_string(),
        })?;

        request.metadata_mut().insert("authorization", value);

        Ok(())
    }

    pub async fn login(
        &self,
        account_id: String,
        account_secret: String,
    ) -> Result<String, DSError> {
        let request = Request::new(LoginRequest {
            body: Some(LoginBody {
                data: Some(LoginData {
                    account_id,
                    account_secret,
                }),
            }),
        });

        let response = self.inner.login(request).await?;

        Ok(response.token)
    }

    pub async fn packets_insert(
        &self,
        token: String,
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
            }),
            body: Some(BatchCreateBody { records }),
        });

        Self::set_token_for_request(&mut request, token)?;

        let response = self.inner.batch_create(request).await?;

        Ok(response)
    }
}
