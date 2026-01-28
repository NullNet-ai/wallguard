use crate::datastore::db_tables::DBTable;
use crate::datastore::{Datastore, SshSessionModel};
use crate::utilities::json;
use nullnet_libdatastore::GetByIdRequestBuilder;
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn obtain_ssh_session(
        &self,
        token: &str,
        session_id: &str,
        performed_by_root: bool,
    ) -> Result<Option<SshSessionModel>, Error> {
        let request = GetByIdRequestBuilder::new()
            .table(DBTable::SshSessions)
            .durability("hard")
            .id(session_id)
            .pluck(SshSessionModel::pluck())
            .performed_by_root(performed_by_root)
            .build();

        let response = self.inner.clone().get_by_id(request, token).await?;

        if response.count == 0 {
            return Ok(None);
        }

        let json_data = json::parse_string(&response.data)?;
        let data = json::first_element_from_array(&json_data)?;

        let device = serde_json::from_value::<SshSessionModel>(data).handle_err(location!())?;
        Ok(Some(device))
    }
}
