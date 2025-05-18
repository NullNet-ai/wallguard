use crate::datastore::builders::{AdvanceFilterBuilder, GetByFilterRequestBuilder};
use crate::datastore::{Datastore, RemoteAccessSession};
use crate::utilities::json;
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn obtain_session(
        &self,
        token: &str,
        session_token: &str,
    ) -> Result<RemoteAccessSession, Error> {
        let filter = AdvanceFilterBuilder::new()
            .field("remote_access_type")
            .values(format!("[\"{session_token}\"]"))
            .r#type("criteria")
            .operator("equal")
            .entity(RemoteAccessSession::table())
            .build();

        let request = GetByFilterRequestBuilder::new()
            .table(RemoteAccessSession::table())
            .plucks(RemoteAccessSession::pluck())
            .limit(1)
            .advance_filter(filter)
            .order_by("timestamp")
            .order_direction("desc")
            .build();

        let response = self.inner.clone().get_by_filter(request, token).await?;
        if response.count == 0 {
            return Err("Query returned no results").handle_err(location!());
        }

        let json_data = json::parse_string(&response.data)?;
        let keypair_data = json::first_element_from_array(&json_data)?;
        serde_json::from_value::<RemoteAccessSession>(keypair_data).handle_err(location!())
    }
}
