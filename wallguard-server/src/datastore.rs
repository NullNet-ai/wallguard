use crate::parser::parsed_message::ParsedMessage;
use libdatastore::{
    BatchCreateBody, BatchCreateRequest, CreateParams, DatastoreClient, DatastoreConfig,
    Error as DSError, ErrorKind as DSErrorKind, LoginBody, LoginData, LoginRequest, Params, Query,
    Response as DSResponse, UpdateRequest,
};
use serde_json::json;
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

    pub async fn device_setup(
        &self,
        token: String,
        device_id: String,
        device_version: String,
        device_uuid: String,
        device_hostname: String,
        device_address: String,
    ) -> Result<DSResponse, DSError> {
        let mut request = Request::new(UpdateRequest {
            params: Some(Params {
                table: String::from("devices"),
                id: device_id,
            }),
            query: Some(Query {
                pluck: String::from("id,code"),
            }),
            body: json!({
                "device_version": device_version,
                "hostname": device_hostname,
                "system_id": device_uuid,
                "ip_address": device_address,
                "is_connection_established": true
            })
            .to_string(),
        });

        Self::set_token_for_request(&mut request, token)?;

        let response = self.inner.update(request).await?;

        Ok(response)
    }
}
