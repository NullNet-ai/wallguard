use crate::datastore::{Datastore, TunnelModel, TunnelStatus};
use nullnet_libdatastore::{AdvanceFilterBuilder, BatchUpdateRequestBuilder};
use nullnet_liberror::Error;
use serde_json::json;

impl Datastore {
    pub async fn mark_all_tunnels_terminated(
        &self,
        token: &str,
        performed_by_root: bool,
    ) -> Result<(), Error> {
        let updates = json!({"tunnel_status": TunnelStatus::Terminated.to_string()});

        let filter = AdvanceFilterBuilder::new()
            .field("status")
            .values("[\"Active\"]")
            .r#type("criteria")
            .operator("equal")
            .entity(TunnelModel::table())
            .build();

        let request = BatchUpdateRequestBuilder::new()
            .advance_filter(filter)
            .performed_by_root(performed_by_root)
            .table(TunnelModel::table())
            .updates(updates.to_string())
            .build();

        let _ = self.inner.clone().batch_update(request, token).await?;

        Ok(())
    }
}
