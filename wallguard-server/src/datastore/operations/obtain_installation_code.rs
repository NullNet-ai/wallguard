use crate::datastore::{Datastore, InstallationCode};
use crate::utilities::json;
use nullnet_libdatastore::{AdvanceFilterBuilder, GetByFilterRequestBuilder};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn obtain_installation_code(
        &self,
        code: &str,
        token: &str,
    ) -> Result<Option<InstallationCode>, Error> {
        let filter = AdvanceFilterBuilder::new()
            .field("code")
            .values(format!("[\"{code}\"]"))
            .r#type("criteria")
            .operator("equal")
            .entity(InstallationCode::table())
            .build();

        let request = GetByFilterRequestBuilder::new()
            .table(InstallationCode::table())
            .plucks(InstallationCode::pluck())
            .limit(1)
            .advance_filter(filter)
            .order_by("timestamp")
            .order_direction("desc")
            .case_sensitive_sorting(true)
            // We assume that at this point the server performs this request
            // using root credentials.
            .performed_by_root(true)
            .build();

        let response = self.inner.clone().get_by_filter(request, token).await?;

        if response.count == 0 {
            return Ok(None);
        }

        let json_data = json::parse_string(&response.data)?;
        let data = json::first_element_from_array(&json_data)?;

        let installation_code =
            serde_json::from_value::<InstallationCode>(data).handle_err(location!())?;
        Ok(Some(installation_code))
    }
}
