use nullnet_libdatastore::UpdateRequestBuilder;
use nullnet_liberror::Error;
use serde_json::json;

use crate::datastore::{Datastore, TunnelModel, TunnelStatus};

impl Datastore {
    pub async fn update_tunnel_status(
        &self,
        token: &str,
        tunnel_id: &str,
        status: TunnelStatus,
        performed_by_root: bool,
    ) -> Result<(), Error> {
        let body = json!({
            "tunnel_status": status.to_string()
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
