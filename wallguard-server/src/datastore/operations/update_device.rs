use crate::datastore::{
    Datastore, Device,
    db_tables::DBTable,
    generated::{
        BatchUpdateDevicesRequest, BatchUpdateParams, Devices, FilterCriteria, FilterOperator,
        UpdateDevicesRequest, UpdateParams, UpdateQuery, batch_update_devices_request,
    },
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn update_device(
        &self,
        token: &str,
        device_id: &str,
        device: &Device,
    ) -> Result<bool, Error> {
        let request = UpdateDevicesRequest {
            device: Some(Devices {
                device_uuid: Some(device.uuid.clone()),
                is_traffic_monitoring_enabled: Some(device.traffic_monitoring),
                is_config_monitoring_enabled: Some(device.sysconf_monitoring),
                is_telemetry_monitoring_enabled: Some(device.telemetry_monitoring),
                is_device_authorized: Some(device.authorized),
                device_category: Some(device.category.clone()),
                device_type: Some(device.r#type.clone()),
                device_name: Some(device.name.clone()),
                device_os: Some(device.os.clone()),
                is_device_online: Some(device.online),
                organization_id: Some(device.organization.clone()),
                device_version: Some(device.version.clone()),
                ..Default::default()
            }),
            params: Some(UpdateParams {
                id: device_id.to_string(),
                table: DBTable::Devices.into(),
                r#type: String::new(),
            }),
            query: Some(UpdateQuery {
                pluck: String::new(),
            }),
        };

        let response = self
            .inner
            .clone()
            .update_devices(request)
            .await
            .handle_err(location!())?
            .into_inner();

        Ok(response.count == 1)
    }

    pub async fn update_device_online_status(
        &self,
        token: &str,
        device_id: &str,
        is_online: bool,
    ) -> Result<(), Error> {
        let request = UpdateDevicesRequest {
            device: Some(Devices {
                is_device_online: Some(is_online),
                ..Default::default()
            }),
            params: Some(UpdateParams {
                id: device_id.to_string(),
                table: DBTable::Devices.into(),
                r#type: String::new(),
            }),
            query: Some(UpdateQuery {
                pluck: String::new(),
            }),
        };

        let _ = self
            .inner
            .clone()
            .update_devices(request)
            .await
            .handle_err(location!())?;

        Ok(())
    }

    pub async fn update_all_devices_online_status(
        &self,
        token: &str,
        is_online: bool,
        performed_by_root: bool,
    ) -> Result<(), Error> {
        let request = BatchUpdateDevicesRequest {
            params: Some(BatchUpdateParams {
                table: DBTable::Devices.into(),
                r#type: if performed_by_root {
                    "root".to_string()
                } else {
                    String::new()
                },
            }),
            body: Some(batch_update_devices_request::BatchUpdateBody {
                advance_filters: vec![FilterCriteria {
                    r#type: "criteria".to_string(),
                    field: Some("is_device_online".to_string()),
                    entity: Some(DBTable::Devices.into()),
                    operator: Some(FilterOperator::Equal as i32),
                    values: vec!["true".to_string()],
                    ..Default::default()
                }],
                updates: Some(Devices {
                    is_device_online: Some(is_online),
                    ..Default::default()
                }),
            }),
        };

        let _ = self
            .inner
            .clone()
            .batch_update_devices(request)
            .await
            .handle_err(location!())?;

        Ok(())
    }
}
