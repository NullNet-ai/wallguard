use nullnet_libdatastore::UpdateRequestBuilder;
use nullnet_liberror::Error;
use serde_json::json;

use crate::datastore::{Datastore, TtySessionModel, TtySessionStatus};

impl Datastore {
    pub async fn update_tty_session_status(
        &self,
        token: &str,
        session_id: &str,
        status: TtySessionStatus,
        performed_by_root: bool,
    ) -> Result<(), Error> {
        let body = json!({
            "session_status": status.to_string()
        })
        .to_string();

        let request = UpdateRequestBuilder::new()
            .id(session_id)
            .table(TtySessionModel::table())
            .body(body)
            .performed_by_root(performed_by_root)
            .build();

        let _ = self.inner.clone().update(request, token).await?;

        Ok(())
    }
}
