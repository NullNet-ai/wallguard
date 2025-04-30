use crate::{datastore::DatastoreWrapper, tunnel::RAType};
use nullnet_libdatastore::{AdvanceFilter, BatchUpdateBody, BatchUpdateRequest, Params};
use nullnet_liberror::Error;
use serde_json::json;

impl DatastoreWrapper {
    pub async fn device_terminate_remote_session(
        &self,
        token: &str,
        device_id: String,
        remote_access_type: RAType,
    ) -> Result<(), Error> {
        let table = "device_remote_access_sessions";

        let updates = json!({
            "remote_access_status": "terminated"
        });

        let request = BatchUpdateRequest {
            params: Some(Params {
                id: String::new(),
                table: String::from(table),
            }),
            body: Some(BatchUpdateBody {
                advance_filters: vec![
                    AdvanceFilter {
                        r#type: "criteria".to_string(),
                        field: "remote_access_type".to_string(),
                        operator: "equal".to_string(),
                        entity: table.to_string(),
                        values: format!("[\"{}\"]", remote_access_type),
                    },
                    AdvanceFilter {
                        r#type: "operator".to_string(),
                        field: String::new(),
                        operator: "and".to_string(),
                        entity: String::new(),
                        values: String::new(),
                    },
                    AdvanceFilter {
                        r#type: "criteria".to_string(),
                        field: "device_id".to_string(),
                        operator: "equal".to_string(),
                        entity: table.to_string(),
                        values: format!("[\"{}\"]", device_id),
                    },
                ],
                updates: updates.to_string(),
            }),
        };

        let _ = self.inner.clone().batch_update(request, token).await?;

        Ok(())
    }

    pub async fn device_terminate_all_active_sessions(&self, token: &str) -> Result<(), Error> {
        let table = "device_remote_access_sessions";

        let updates = json!({
            "remote_access_status": "terminated"
        });

        let request = BatchUpdateRequest {
            params: Some(Params {
                id: String::new(),
                table: String::from(table),
            }),
            body: Some(BatchUpdateBody {
                advance_filters: vec![AdvanceFilter {
                    r#type: "criteria".to_string(),
                    field: "remote_access_status".to_string(),
                    operator: "equal".to_string(),
                    entity: table.to_string(),
                    values: "[\"active\"]".to_string(),
                }],
                updates: updates.to_string(),
            }),
        };

        let _ = self.inner.clone().batch_update(request, token).await?;

        Ok(())
    }
}
