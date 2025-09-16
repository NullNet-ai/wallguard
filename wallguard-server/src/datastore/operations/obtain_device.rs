use crate::datastore::db_tables::DBTable;
use crate::datastore::{Datastore, Device};
use crate::utilities::json;
use nullnet_libdatastore::AdvanceFilterBuilder;
use nullnet_libdatastore::GetByFilterRequestBuilder;
use nullnet_libdatastore::GetByIdRequestBuilder;
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn obtain_device_by_uuid(
        &self,
        token: &str,
        device_uuid: &str,
    ) -> Result<Option<Device>, Error> {
        let filter = AdvanceFilterBuilder::new()
            .field("device_uuid")
            .values(format!("[\"{device_uuid}\"]"))
            .r#type("criteria")
            .operator("equal")
            .entity(Device::table())
            .build();

        let request = GetByFilterRequestBuilder::new()
            .table(Device::table())
            .plucks(Device::pluck())
            .limit(1)
            .advance_filter(filter)
            .order_by("timestamp")
            .order_direction("desc")
            .case_sensitive_sorting(true)
            // `obtain_device_by_uuid` is done by `AuthReqHandler`
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

        let device = serde_json::from_value::<Device>(data).handle_err(location!())?;
        Ok(Some(device))
    }

    pub async fn obtain_device_by_id(
        &self,
        token: &str,
        device_id: &str,
        performed_by_root: bool,
    ) -> Result<Option<Device>, Error> {
        let request = GetByIdRequestBuilder::new()
            .id(device_id)
            .pluck(Device::pluck())
            .table(DBTable::Devices)
            .performed_by_root(performed_by_root)
            .build();

        let response = self.inner.clone().get_by_id(request, token).await?;
        if response.count == 0 {
            return Ok(None);
        }

        let json_data = json::parse_string(&response.data)?;
        let data = json::first_element_from_array(&json_data)?;

        let device = serde_json::from_value::<Device>(data).handle_err(location!())?;
        Ok(Some(device))
    }
}
