use crate::datastore::{Datastore, TtySessionModel};
use nullnet_libdatastore::{AdvanceFilterBuilder, BatchUpdateRequestBuilder};
use nullnet_liberror::Error;
use serde_json::json;

impl Datastore {
    pub async fn terminate_all_active_tty_sessions(
        &self,
        token: &str,
        performed_by_root: bool,
    ) -> Result<(), Error> {
        let update = json!({"session_status": "terminated"});

        let filter = AdvanceFilterBuilder::new()
            .field("session_status")
            .values("[\"active\"]")
            .r#type("criteria")
            .operator("equal")
            .entity(TtySessionModel::table())
            .build();

        let request = BatchUpdateRequestBuilder::new()
            .advance_filter(filter)
            .performed_by_root(performed_by_root)
            .table(TtySessionModel::table())
            .updates(update.to_string())
            .build();

        let _ = self.inner.clone().batch_update(request, token).await?;

        Ok(())
    }
}
