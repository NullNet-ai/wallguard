use nullnet_libdatastore::CreateRequestBuilder;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use serde_json::json;

use crate::{
    datastore::{Datastore, SshSessionModel},
    utilities,
};

impl Datastore {
    pub async fn create_ssh_session(
        &self,
        token: &str,
        session: &SshSessionModel,
    ) -> Result<String, Error> {
        let mut json = json!(session);

        json.as_object_mut().unwrap().remove("id");

        let request = CreateRequestBuilder::new()
            .pluck(SshSessionModel::pluck())
            .table(SshSessionModel::table())
            .record(json.to_string())
            .build();

        let response = self.inner.clone().create(request, token).await?;

        let json_data = utilities::json::parse_string(&response.data)?;
        let value = utilities::json::first_element_from_array(&json_data)?;
        let retval = serde_json::from_value::<SshSessionModel>(value).handle_err(location!())?;

        Ok(retval.id)
    }
}
