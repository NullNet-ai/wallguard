use crate::{
    datastore::{
        AliasModel, Datastore,
        db_tables::DBTable,
    },
    utilities,
};
use nullnet_libdatastore::{BatchCreateRequestBuilder, CreateRequestBuilder};

use nullnet_liberror::{Error, ErrorHandler, Location, location};
use serde_json::json;
use wallguard_common::protobuf::wallguard_models::Alias;

impl Datastore {
    pub async fn create_alias(
        &self,
        token: &str,
        alias: &Alias,
        config_id: &str,
    ) -> Result<(), Error> {
        let alias_model = AliasModel {
            device_configuration_id: config_id.into(),
            r#type: alias.r#type.clone(),
            name: alias.name.clone(),
            description: alias.description.clone(),
            alias_status: String::new(),
            family: alias.family.clone(),
            table: alias.table.clone(),
        };

        let request = CreateRequestBuilder::new()
            .table(DBTable::DeviceAliases)
            .durability("hard")
            .pluck(["id"])
            .record(json!(alias_model).to_string())
            .build();

        let response = self.inner.clone().create(request, token).await?;

        if response.count != 0 {
            Err("Failed to create an alias").handle_err(location!())?;
        }

        let json_data = utilities::json::parse_string(&response.data)?;
        let data = utilities::json::first_element_from_array(&json_data)?;

        let alias_id = data["id"]
            .as_str()
            .ok_or("Missing or invalid 'id' field")
            .handle_err(location!())?
            .to_string();

        let ip_aliases = alias_model.extract_ip_aliases(alias, &alias_id);

        if !ip_aliases.is_empty() {
            let records: Vec<serde_json::Value> = ip_aliases
                .iter()
                .map(|record| serde_json::to_value(record).expect("Serialization failed"))
                .collect();

            let request = BatchCreateRequestBuilder::new()
                .durability("soft")
                .table(DBTable::IpAliases)
                .records(serde_json::to_string(&serde_json::Value::Array(records)).unwrap())
                .build();

            let _ = self.inner.clone().batch_create(request, token).await?;
        }

        let port_aliases = alias_model.extract_port_aliases(alias, &alias_id);

        if !port_aliases.is_empty() {
            let records: Vec<serde_json::Value> = port_aliases
                .iter()
                .map(|record| serde_json::to_value(record).expect("Serialization failed"))
                .collect();

            let request = BatchCreateRequestBuilder::new()
                .durability("soft")
                .table(DBTable::PortAliases)
                .records(serde_json::to_string(&serde_json::Value::Array(records)).unwrap())
                .build();

            let _ = self.inner.clone().batch_create(request, token).await?;
        }

        Ok(())
    }
}
