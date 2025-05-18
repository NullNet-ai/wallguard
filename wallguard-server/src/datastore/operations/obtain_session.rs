use crate::datastore::builders::{AdvanceFilterBuilder, GetByFilterRequestBuilder};
use crate::datastore::db_tables::DBTable;
use crate::datastore::{Datastore, RemoteAccessSession};
use crate::utilities::json;
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn obtain_session(
        &self,
        token: &str,
        device_id: &str,
    ) -> Result<RemoteAccessSession, Error> {
        let filter = AdvanceFilterBuilder::new()
            .field("device_id")
            .values(format!("[\"{device_id}\"]"))
            .r#type("criteria")
            .operator("equal")
            .entity(DBTable::RemoteAccessSessions)
            .build();

        let request = GetByFilterRequestBuilder::new()
            .table(DBTable::RemoteAccessSessions)
            .plucks(RemoteAccessSession::pluck())
            .limit(1)
            .advance_filter(filter)
            .order_by("timestamp")
            .order_direction("desc")
            .build();

        let response = self.inner.clone().get_by_filter(request, token).await?;
        let json_data = json::parse_string(&response.data)?;
        let session_data = json::first_element_from_array(&json_data)?;
        serde_json::from_value::<RemoteAccessSession>(session_data).handle_err(location!())
    }
}
