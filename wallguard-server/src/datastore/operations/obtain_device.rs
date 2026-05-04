use crate::datastore::{
    Datastore, Device,
    db_tables::DBTable,
    generated::{
        AggregationFilterParams, AggregationFilterRequest, FilterCriteria, FilterOperator,
        aggregation_filter_request,
    },
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn obtain_device_by_id(
        &self,
        token: &str,
        device_id: &str,
        performed_by_root: bool,
    ) -> Result<Option<Device>, Error> {
        let request = AggregationFilterRequest {
            params: Some(AggregationFilterParams {
                r#type: if performed_by_root {
                    "root".to_string()
                } else {
                    String::new()
                },
            }),
            body: Some(aggregation_filter_request::AggregationFilterBody {
                entity: DBTable::Devices.into(),
                advance_filters: vec![FilterCriteria {
                    r#type: "criteria".to_string(),
                    field: Some("id".to_string()),
                    entity: Some(DBTable::Devices.into()),
                    operator: Some(FilterOperator::Equal as i32),
                    values: vec![format!("\"{}\"", device_id)],
                    ..Default::default()
                }],
                limit: Some(1),
                ..Default::default()
            }),
        };

        let response = self
            .inner
            .clone()
            .aggregation_filter(request)
            .await
            .handle_err(location!())?
            .into_inner();

        if response.count == 0 {
            return Ok(None);
        }

        let data: Vec<serde_json::Value> =
            serde_json::from_str(&response.data).handle_err(location!())?;
        let first = data
            .into_iter()
            .next()
            .ok_or("Empty device data")
            .handle_err(location!())?;
        let device = serde_json::from_value::<Device>(first).handle_err(location!())?;

        Ok(Some(device))
    }
}
