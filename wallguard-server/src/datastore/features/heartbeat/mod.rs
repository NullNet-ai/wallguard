mod latest_device_info;

use crate::datastore::DatastoreWrapper;
use chrono::Utc;
use latest_device_info::LatestDeviceInfo;
use nullnet_libdatastore::{
    CreateBody, CreateParams, CreateRequest, DatastoreClient, GetByIdRequest, Params, Query,
    ResponseData,
};
use nullnet_liberror::Error;
use serde_json::json;

impl DatastoreWrapper {
    pub async fn heartbeat(
        &self,
        token: &str,
        device_id: String,
    ) -> Result<LatestDeviceInfo, Error> {
        let (create_result, fetch_result) = tokio::join!(
            Self::internal_hb_create_hb_record(self.inner.clone(), device_id.clone(), token),
            Self::internal_hb_fetch_device_info(self.inner.clone(), device_id, token)
        );

        let _ = create_result?;

        fetch_result
    }

    async fn internal_hb_create_hb_record(
        mut client: DatastoreClient,
        device_id: String,
        token: &str,
    ) -> Result<ResponseData, Error> {
        let request = CreateRequest {
            params: Some(CreateParams {
                table: String::from("device_heartbeats"),
            }),
            query: Some(Query {
                pluck: String::new(),
                durability: String::from("soft"),
            }),
            body: Some(CreateBody {
                record: json!({
                    "device_id": device_id.clone(),
                    "timestamp": Utc::now().to_rfc3339(),
                })
                .to_string(),
                entity_prefix: String::from("HB"),
            }),
        };

        let retval = client.create(request, token).await?;

        Ok(retval)
    }

    async fn internal_hb_fetch_device_info(
        mut client: DatastoreClient,
        device_id: String,
        token: &str,
    ) -> Result<LatestDeviceInfo, Error> {
        let request = GetByIdRequest {
            params: Some(Params {
                id: device_id,
                table: String::from("devices"),
            }),
            query: Some(Query {
                pluck: String::from("status,is_monitoring_enabled,is_remote_access_enabled"),
                durability: String::from("soft"),
            }),
        };

        let response = client.get_by_id(request, token).await?;
        LatestDeviceInfo::from_response_data(&response)
    }
}
