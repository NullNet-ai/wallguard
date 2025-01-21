use super::config::DatastoreConfig;
use super::token::AuthToken;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::datastore::auth::authentication_request_impl;
use crate::parser::parsed_message::ParsedMessage;
use crate::proto::dna_store::store_service_client::StoreServiceClient;
use crate::proto::dna_store::{BatchCreateBody, BatchCreateRequest, CreateParams, Query};
use tonic::metadata::MetadataValue;
use tonic::transport::{Channel, ClientTlsConfig};

pub struct DatastoreClient {
    config: Arc<DatastoreConfig>,
    client: StoreServiceClient<Channel>,
    token: Arc<Mutex<Option<AuthToken>>>,
}

impl Clone for DatastoreClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            config: Arc::clone(&self.config),
            token: Arc::clone(&self.token),
        }
    }
}

impl DatastoreClient {
    pub async fn connect(config: DatastoreConfig) -> Result<Self, String> {
        let protocol = if config.tls { "https" } else { "http" };
        let host = config.host.as_str();
        let port = config.port;

        let mut endpoint = Channel::from_shared(format!("{protocol}://{host}:{port}"))
            .map_err(|e| e.to_string())?
            .connect_timeout(std::time::Duration::from_secs(10));

        if config.tls {
            endpoint = endpoint
                .tls_config(ClientTlsConfig::new().with_native_roots())
                .map_err(|e| e.to_string())?;
        }

        let channel: Channel = endpoint.connect().await.map_err(|e| e.to_string())?;

        Ok(DatastoreClient {
            client: StoreServiceClient::new(channel),
            config: Arc::new(config),
            token: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn handle_authentication(&mut self) -> Result<(), String> {
        let mut value = self.token.lock().await;

        if value.as_ref().map_or(true, AuthToken::is_expired) {
            let new_token = authentication_request_impl(
                self.client.clone(),
                self.config.email.clone(),
                self.config.password.clone(),
                self.config.token_leeway,
            )
            .await?;

            *value = Some(new_token);
        }

        Ok(())
    }

    pub async fn save_message(
        &mut self,
        message: ParsedMessage,
    ) -> Result<crate::proto::dna_store::Response, String> {
        self.handle_authentication().await?;

        let records = serde_json::to_string(&message).map_err(|e| e.to_string())?;

        let mut request = tonic::Request::new(BatchCreateRequest {
            params: Some(CreateParams {
                table: String::from("packets"),
            }),
            query: Some(Query {
                pluck: String::new(),
            }),
            body: Some(BatchCreateBody { records }),
        });

        let metadata = request.metadata_mut();

        let metadata_value = {
            let value = self.token.lock().await;
            MetadataValue::from_str(value.as_ref().unwrap().as_str()).map_err(|e| e.to_string())?
        };

        metadata.insert("authorization", metadata_value);

        match self.client.batch_create(request).await {
            Ok(response) => Ok(response.into_inner()),
            Err(err) => Err(err.to_string()),
        }
    }
}
