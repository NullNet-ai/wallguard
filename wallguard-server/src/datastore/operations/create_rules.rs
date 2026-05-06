use crate::datastore::{
    Datastore,
    db_tables::DBTable,
    generated::{
        BatchInsertDeviceFilterRulesRequest, BatchInsertDeviceNatRulesRequest, BatchInsertParams,
        BatchInsertQuery, DeviceFilterRules, DeviceNatRules,
        batch_insert_device_filter_rules_request, batch_insert_device_nat_rules_request,
    },
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use wallguard_common::protobuf::{
    wallguard_models::{FilterRule, NatRule},
    wallguard_service::ConfigStatus,
};

impl Datastore {
    pub async fn create_filter_rules(
        &self,
        token: &str,
        rules: &[FilterRule],
        config_id: &str,
        status: ConfigStatus,
    ) -> Result<(), Error> {
        if rules.is_empty() {
            return Ok(());
        }

        let status_str = config_status_str(status);

        let records: Vec<DeviceFilterRules> = rules
            .iter()
            .map(|rule| DeviceFilterRules {
                device_configuration_id: Some(config_id.to_string()),
                device_rule_status: Some(status_str.to_string()),
                disabled: Some(rule.disabled),
                policy: Some(rule.policy.clone()),
                protocol: Some(rule.protocol.clone()),
                ipprotocol: Some(rule.ipprotocol.clone()),
                source_inversed: Some(rule.source_inversed),
                source_port_value: rule.source_port.as_ref().map(|p| p.value.clone()),
                source_port_operator: rule.source_port.as_ref().map(|p| p.operator.clone()),
                source_ip_value: rule.source_addr.as_ref().map(|a| a.value.clone()),
                source_ip_operator: rule.source_addr.as_ref().map(|a| a.operator.clone()),
                source_ip_version: rule.source_addr.as_ref().map(|a| a.version),
                source_type: Some(rule.source_type.clone()),
                destination_inversed: Some(rule.destination_inversed),
                destination_port_value: rule.destination_port.as_ref().map(|p| p.value.clone()),
                destination_port_operator: rule
                    .destination_port
                    .as_ref()
                    .map(|p| p.operator.clone()),
                destination_ip_value: rule.destination_addr.as_ref().map(|a| a.value.clone()),
                destination_ip_operator: rule.destination_addr.as_ref().map(|a| a.operator.clone()),
                destination_ip_version: rule.destination_addr.as_ref().map(|a| a.version),
                destination_type: Some(rule.destination_type.clone()),
                description: Some(rule.description.clone()),
                interface: Some(rule.interface.clone()),
                order: Some(rule.order as i32),
                associated_rule_id: Some(rule.associated_rule_id.clone()),
                table: Some(rule.table.clone()),
                chain: Some(rule.chain.clone()),
                family: Some(rule.family.clone()),
                floating: Some(rule.floating),
                ..Default::default()
            })
            .collect();

        let request = BatchInsertDeviceFilterRulesRequest {
            params: Some(BatchInsertParams {
                table: DBTable::DeviceFilterRules.into(),
                r#type: String::new(),
            }),
            query: Some(BatchInsertQuery {
                pluck: String::new(),
            }),
            body: Some(batch_insert_device_filter_rules_request::BatchBody {
                device_filter_rules: records,
            }),
        };

        let mut grpc_request = tonic::Request::new(request);
        grpc_request.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", token)
                .parse()
                .handle_err(location!())?,
        );

        let _ = self
            .inner
            .clone()
            .batch_insert_device_filter_rules(grpc_request)
            .await
            .handle_err(location!())?;

        Ok(())
    }

    pub async fn create_nat_rules(
        &self,
        token: &str,
        rules: &[NatRule],
        config_id: &str,
        status: ConfigStatus,
    ) -> Result<(), Error> {
        if rules.is_empty() {
            return Ok(());
        }

        let status_str = config_status_str(status);

        let records: Vec<DeviceNatRules> = rules
            .iter()
            .map(|rule| DeviceNatRules {
                device_configuration_id: Some(config_id.to_string()),
                device_rule_status: Some(status_str.to_string()),
                disabled: Some(rule.disabled),
                protocol: Some(rule.protocol.clone()),
                ipprotocol: Some(rule.ipprotocol.clone()),
                source_inversed: Some(rule.source_inversed),
                source_port_value: rule.source_port.as_ref().map(|p| p.value.clone()),
                source_port_operator: rule.source_port.as_ref().map(|p| p.operator.clone()),
                source_ip_value: rule.source_addr.as_ref().map(|a| a.value.clone()),
                source_ip_operator: rule.source_addr.as_ref().map(|a| a.operator.clone()),
                source_ip_version: rule.source_addr.as_ref().map(|a| a.version),
                source_type: Some(rule.source_type.clone()),
                destination_inversed: Some(rule.destination_inversed),
                destination_port_value: rule.destination_port.as_ref().map(|p| p.value.clone()),
                destination_port_operator: rule
                    .destination_port
                    .as_ref()
                    .map(|p| p.operator.clone()),
                destination_ip_value: rule.destination_addr.as_ref().map(|a| a.value.clone()),
                destination_ip_operator: rule.destination_addr.as_ref().map(|a| a.operator.clone()),
                destination_ip_version: rule.destination_addr.as_ref().map(|a| a.version),
                destination_type: Some(rule.destination_type.clone()),
                description: Some(rule.description.clone()),
                interface: Some(rule.interface.clone()),
                redirect_ip: Some(rule.redirect_ip.clone()),
                redirect_port: Some(rule.redirect_port as i32),
                order: Some(rule.order as i32),
                associated_rule_id: Some(rule.associated_rule_id.clone()),
                table: Some(rule.table.clone()),
                chain: Some(rule.chain.clone()),
                family: Some(rule.family.clone()),
                ..Default::default()
            })
            .collect();

        let request = BatchInsertDeviceNatRulesRequest {
            params: Some(BatchInsertParams {
                table: DBTable::DeviceNatRules.into(),
                r#type: String::new(),
            }),
            query: Some(BatchInsertQuery {
                pluck: String::new(),
            }),
            body: Some(batch_insert_device_nat_rules_request::BatchBody {
                device_nat_rules: records,
            }),
        };

        let mut grpc_request = tonic::Request::new(request);
        grpc_request.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", token)
                .parse()
                .handle_err(location!())?,
        );

        let _ = self
            .inner
            .clone()
            .batch_insert_device_nat_rules(grpc_request)
            .await
            .handle_err(location!())?;

        Ok(())
    }
}

fn config_status_str(status: ConfigStatus) -> &'static str {
    match status {
        ConfigStatus::CsDraft => "Draft",
        ConfigStatus::CsApplied => "Applied",
        ConfigStatus::CsUndefined => "Undefined",
    }
}
