use nullnet_libdatastore::{AdvanceFilterBuilder, GetByFilterRequestBuilder};
use crate::datastore::db_tables::DBTable;
use crate::datastore::{Datastore, DeviceConfiguration};
use crate::utilities;
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn obtain_config(
        &self,
        token: &str,
        device_id: &str,
    ) -> Result<Option<DeviceConfiguration>, Error> {
        let filter = AdvanceFilterBuilder::new()
            .field("device_id")
            .values(format!("[\"{device_id}\"]"))
            .r#type("criteria")
            .operator("equal")
            .entity(DBTable::DeviceConfigurations)
            .build();

        let request = GetByFilterRequestBuilder::new()
            .plucks(DeviceConfiguration::pluck())
            .order_by("timestamp")
            .order_direction("desc")
            .limit(1)
            .advance_filter(filter)
            .table(DBTable::DeviceConfigurations)
            .case_sensitive_sorting(true)
            .build();

        let response = self.inner.clone().get_by_filter(request, token).await?;

        if response.count == 1 {
            let json = utilities::json::parse_string(&response.data)?;
            let data = utilities::json::first_element_from_array(&json)?;

            let config =
                serde_json::from_value::<DeviceConfiguration>(data).handle_err(location!())?;

            Ok(Some(config))
        } else {
            Ok(None)
        }
    }
}
