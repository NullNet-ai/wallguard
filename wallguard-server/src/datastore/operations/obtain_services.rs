use crate::datastore::db_tables::DBTable;
use crate::datastore::{Datastore, ServiceInfo};
use crate::utilities;
use nullnet_libdatastore::{AdvanceFilterBuilder, GetByFilterRequestBuilder};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn obtain_services(
        &self,
        token: &str,
        device_id: &str,
        performed_by_root: bool,
    ) -> Result<Option<Vec<ServiceInfo>>, Error> {
        let filter = AdvanceFilterBuilder::new()
            .field("device_id")
            .values(format!("[\"{device_id}\"]"))
            .r#type("criteria")
            .operator("equal")
            .entity(DBTable::DeviceServices)
            .build();

        let request = GetByFilterRequestBuilder::new()
            .table(DBTable::DeviceServices)
            .plucks(ServiceInfo::pluck())
            .advance_filter(filter)
            .performed_by_root(performed_by_root)
            .build();

        let response = self.inner.clone().get_by_filter(request, token).await?;

        if response.count == 0 {
            return Ok(None);
        }

        let json_data = utilities::json::parse_string(&response.data)?;

        let services_array = json_data
            .as_array()
            .ok_or("Unexpected data format")
            .handle_err(location!())?;

        let mut services = Vec::with_capacity(services_array.len());

        for value in services_array {
            let service = serde_json::from_value::<ServiceInfo>(value.clone())
                .map_err(|e| format!("Failed to deserialize ServiceInfo: {}", e))
                .handle_err(location!())?;

            services.push(service);
        }

        Ok(Some(services))
    }
}
