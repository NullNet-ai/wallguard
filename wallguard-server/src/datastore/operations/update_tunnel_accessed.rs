use nullnet_libdatastore::UpdateRequestBuilder;
use nullnet_liberror::Error;
use serde_json::json;

use crate::datastore::{Datastore, TunnelModel};

impl Datastore {
    pub async fn update_tunnel_accessed(
        &self,
        token: &str,
        tunnel_id: &str,
        performed_by_root: bool,
        timestamp: u64,
    ) -> Result<(), Error> {
        let body = json!({
            "last_accessed": timestamp
        })
        .to_string();

        let request = UpdateRequestBuilder::new()
            .id(tunnel_id)
            .table(TunnelModel::table())
            .body(body)
            .performed_by_root(performed_by_root)
            .build();

        let _ = self.inner.clone().update(request, token).await?;

        Ok(())
    }
}
