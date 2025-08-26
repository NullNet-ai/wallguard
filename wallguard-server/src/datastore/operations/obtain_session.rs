use crate::datastore::builders::{AdvanceFilterBuilder, GetByFilterRequestBuilder};
use crate::datastore::{Datastore, RemoteAccessSession};
use crate::utilities::json;
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn obtain_session(
        &self,
        token: &str,
        session_token: &str,
    ) -> Result<Option<RemoteAccessSession>, Error> {
        let filter = AdvanceFilterBuilder::new()
            .field("remote_access_session")
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
            .case_sensitive_sorting(true)
            .build();

        let response = self.inner.clone().get_by_filter(request, token).await?;
        if response.count == 0 {
            return Ok(None);
        }

        let json_data = json::parse_string(&response.data)?;
        let data = json::first_element_from_array(&json_data)?;

        let session =
            serde_json::from_value::<RemoteAccessSession>(data).handle_err(location!())?;
        Ok(Some(session))
    }
}
