use crate::{
    datastore::{
        AliasModel, Datastore,
        db_tables::DBTable,
        generated::{
            Aliases, BatchInsertIpAliasesRequest, BatchInsertParams, BatchInsertPortAliasesRequest,
            BatchInsertQuery, CreateAliasesRequest, CreateParams, CreateQuery, IpAliases,
            PortAliases, batch_insert_ip_aliases_request, batch_insert_port_aliases_request,
        },
    },
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
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

        let request = CreateAliasesRequest {
            aliases: Some(Aliases {
                device_configuration_id: Some(config_id.to_string()),
                r#type: Some(alias_model.r#type.clone()),
                name: Some(alias_model.name.clone()),
                description: Some(alias_model.description.clone()),
                alias_status: Some(alias_model.alias_status.clone()),
                family: Some(alias_model.family.clone()),
                table: Some(alias_model.table.clone()),
                ..Default::default()
            }),
            params: Some(CreateParams {
                table: DBTable::DeviceAliases.into(),
                r#type: String::new(),
            }),
            query: Some(CreateQuery {
                pluck: "id".to_string(),
                ..Default::default()
            }),
        };

        let response = self
            .inner
            .clone()
            .create_aliases(request)
            .await
            .handle_err(location!())?
            .into_inner();

        if response.count == 0 {
            Err("Failed to create an alias").handle_err(location!())?;
        }

        let alias_id = response
            .data
            .and_then(|d| d.id)
            .ok_or("Missing 'id' in alias response")
            .handle_err(location!())?;

        let ip_aliases = alias_model.extract_ip_aliases(alias, &alias_id);

        if !ip_aliases.is_empty() {
            let records: Vec<IpAliases> = ip_aliases
                .iter()
                .map(|a| IpAliases {
                    alias_id: Some(a.alias_id.clone()),
                    ip: Some(a.ip.clone()),
                    prefix: Some(a.prefix),
                    ..Default::default()
                })
                .collect();

            let request = BatchInsertIpAliasesRequest {
                params: Some(BatchInsertParams {
                    table: DBTable::IpAliases.into(),
                    r#type: String::new(),
                }),
                query: Some(BatchInsertQuery {
                    pluck: String::new(),
                }),
                body: Some(batch_insert_ip_aliases_request::BatchBody {
                    ip_aliases: records,
                }),
            };

            let _ = self
                .inner
                .clone()
                .batch_insert_ip_aliases(request)
                .await
                .handle_err(location!())?;
        }

        let port_aliases = alias_model.extract_port_aliases(alias, &alias_id);

        if !port_aliases.is_empty() {
            let records: Vec<PortAliases> = port_aliases
                .iter()
                .map(|a| PortAliases {
                    alias_id: Some(a.alias_id.clone()),
                    lower_port: Some(a.lower_port),
                    upper_port: Some(a.upper_port),
                    ..Default::default()
                })
                .collect();

            let request = BatchInsertPortAliasesRequest {
                params: Some(BatchInsertParams {
                    table: DBTable::PortAliases.into(),
                    r#type: String::new(),
                }),
                query: Some(BatchInsertQuery {
                    pluck: String::new(),
                }),
                body: Some(batch_insert_port_aliases_request::BatchBody {
                    port_aliases: records,
                }),
            };

            let _ = self
                .inner
                .clone()
                .batch_insert_port_aliases(request)
                .await
                .handle_err(location!())?;
        }

        Ok(())
    }
}
