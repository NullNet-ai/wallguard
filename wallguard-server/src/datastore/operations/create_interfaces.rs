use crate::datastore::{
    Datastore,
    db_tables::DBTable,
    generated::{
        BatchInsertDeviceInterfaceAddressesRequest, BatchInsertDeviceInterfacesRequest,
        BatchInsertParams, BatchInsertQuery, DeviceInterfaceAddresses, DeviceInterfaces,
        batch_insert_device_interface_addresses_request, batch_insert_device_interfaces_request,
    },
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use std::collections::HashMap;
use wallguard_common::protobuf::wallguard_models::NetworkInterface;

impl Datastore {
    pub async fn create_interfaces(
        &self,
        token: &str,
        interfaces: &Vec<NetworkInterface>,
        config_id: &str,
    ) -> Result<(), Error> {
        if interfaces.is_empty() {
            return Ok(());
        }

        let iface_records: Vec<DeviceInterfaces> = interfaces
            .iter()
            .map(|iface| DeviceInterfaces {
                device_configuration_id: Some(config_id.to_string()),
                name: Some(iface.name.clone()),
                device: Some(iface.device.clone()),
                description: Some(iface.description.clone()),
                ..Default::default()
            })
            .collect();

        let request = BatchInsertDeviceInterfacesRequest {
            params: Some(BatchInsertParams {
                table: DBTable::DeviceInterfaces.into(),
                r#type: String::new(),
            }),
            query: Some(BatchInsertQuery {
                pluck: "id,device".to_string(),
            }),
            body: Some(batch_insert_device_interfaces_request::BatchBody {
                device_interfaces: iface_records,
            }),
        };

        let response = self
            .inner
            .clone()
            .batch_insert_device_interfaces(request)
            .await
            .handle_err(location!())?
            .into_inner();

        // Build a map from device name → interface ID for address linking
        let id_by_device: HashMap<String, String> = response
            .data
            .into_iter()
            .filter_map(|d| d.device.zip(d.id))
            .collect();

        let mut addr_records: Vec<DeviceInterfaceAddresses> = vec![];

        for iface in interfaces {
            if let Some(iface_id) = id_by_device.get(&iface.device) {
                for address in &iface.addresses {
                    addr_records.push(DeviceInterfaceAddresses {
                        device_interface_id: Some(iface_id.clone()),
                        address: Some(address.address.clone()),
                        ..Default::default()
                    });
                }
            }
        }

        if addr_records.is_empty() {
            return Ok(());
        }

        let request = BatchInsertDeviceInterfaceAddressesRequest {
            params: Some(BatchInsertParams {
                table: DBTable::DeviceInterfaceAddresses.into(),
                r#type: String::new(),
            }),
            query: Some(BatchInsertQuery {
                pluck: String::new(),
            }),
            body: Some(batch_insert_device_interface_addresses_request::BatchBody {
                device_interface_addresses: addr_records,
            }),
        };

        let _ = self
            .inner
            .clone()
            .batch_insert_device_interface_addresses(request)
            .await
            .handle_err(location!())?;

        Ok(())
    }
}
