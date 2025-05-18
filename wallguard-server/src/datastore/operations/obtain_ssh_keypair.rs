use crate::datastore::builders::{AdvanceFilterBuilder, GetByFilterRequestBuilder};
use crate::datastore::{Datastore, SSHKeypair};
use crate::utilities::json;
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn obtain_ssh_keypair(
        &self,
        token: &str,
        device_id: &str,
    ) -> Result<Option<SSHKeypair>, Error> {
        let filter = AdvanceFilterBuilder::new()
            .field("device_id")
            .values(format!("[\"{device_id}\"]"))
            .r#type("criteria")
            .operator("equal")
            .entity(SSHKeypair::table())
            .build();

        let request = GetByFilterRequestBuilder::new()
            .table(SSHKeypair::table())
            .plucks(SSHKeypair::pluck())
            .limit(1)
            .advance_filter(filter)
            .order_by("timestamp")
            .order_direction("desc")
            .build();

        let response = self.inner.clone().get_by_filter(request, token).await?;
        if response.count == 0 {
            return Ok(None);
        }

        let json_data = json::parse_string(&response.data)?;
        let data = json::first_element_from_array(&json_data)?;

        let keypair = serde_json::from_value::<SSHKeypair>(data).handle_err(location!())?;
        Ok(Some(keypair))
    }
}
