use crate::datastore::{
    Datastore,
    db_tables::DBTable,
    generated::{
        BatchUpdateDeviceFilterRulesRequest, BatchUpdateDeviceNatRulesRequest, BatchUpdateParams,
        DeviceFilterRules, DeviceNatRules, FilterCriteria, FilterOperator,
        batch_update_device_filter_rules_request, batch_update_device_nat_rules_request,
    },
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use wallguard_common::protobuf::wallguard_service::ConfigStatus;

impl Datastore {
    pub async fn update_rules_status(
        &self,
        token: &str,
        config_id: &str,
        status: ConfigStatus,
    ) -> Result<(), Error> {
        let (r1, r2) = tokio::join!(
            self.update_filter_rules_status(token, config_id, status),
            self.update_nat_rules_status(token, config_id, status),
        );

        r1?;
        r2?;

        Ok(())
    }

    async fn update_filter_rules_status(
        &self,
        token: &str,
        config_id: &str,
        status: ConfigStatus,
    ) -> Result<(), Error> {
        let request = BatchUpdateDeviceFilterRulesRequest {
            params: Some(BatchUpdateParams {
                table: DBTable::DeviceFilterRules.into(),
                r#type: String::new(),
            }),
            body: Some(batch_update_device_filter_rules_request::BatchUpdateBody {
                advance_filters: vec![FilterCriteria {
                    r#type: "criteria".to_string(),
                    field: Some("device_configuration_id".to_string()),
                    entity: Some(DBTable::DeviceFilterRules.into()),
                    operator: Some(FilterOperator::Equal as i32),
                    values: vec![format!("\"{}\"", config_id)],
                    ..Default::default()
                }],
                updates: Some(DeviceFilterRules {
                    device_rule_status: Some(config_status_str(status).to_string()),
                    ..Default::default()
                }),
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
            .batch_update_device_filter_rules(grpc_request)
            .await
            .handle_err(location!())?;

        Ok(())
    }

    async fn update_nat_rules_status(
        &self,
        token: &str,
        config_id: &str,
        status: ConfigStatus,
    ) -> Result<(), Error> {
        let request = BatchUpdateDeviceNatRulesRequest {
            params: Some(BatchUpdateParams {
                table: DBTable::DeviceNatRules.into(),
                r#type: String::new(),
            }),
            body: Some(batch_update_device_nat_rules_request::BatchUpdateBody {
                advance_filters: vec![FilterCriteria {
                    r#type: "criteria".to_string(),
                    field: Some("device_configuration_id".to_string()),
                    entity: Some(DBTable::DeviceNatRules.into()),
                    operator: Some(FilterOperator::Equal as i32),
                    values: vec![format!("\"{}\"", config_id)],
                    ..Default::default()
                }],
                updates: Some(DeviceNatRules {
                    device_rule_status: Some(config_status_str(status).to_string()),
                    ..Default::default()
                }),
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
            .batch_update_device_nat_rules(grpc_request)
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
