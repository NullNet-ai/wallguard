use crate::datastore::DatastoreWrapper;
use chrono::Utc;
use nullnet_libdatastore::{
    AdvanceFilter, CreateBody, CreateParams, CreateRequest, GetByFilterBody, GetByFilterRequest,
    Params, Query, ResponseData,
};
use nullnet_liberror::Error;
use nullnet_libipinfo::IpInfo;
use serde_json::json;
use std::collections::HashMap;

impl DatastoreWrapper {
    // SELECT COUNT(*) FROM ip_info WHERE ip = {ip}
    pub(crate) async fn is_ip_info_stored(&mut self, ip: &str, token: &str) -> Result<bool, Error> {
        let table = "ip_info";

        let request = GetByFilterRequest {
            params: Some(Params {
                id: String::new(),
                table: table.into(),
            }),
            body: Some(GetByFilterBody {
                pluck: vec!["id".to_string()],
                advance_filters: vec![AdvanceFilter {
                    r#type: "criteria".to_string(),
                    field: "ip".to_string(),
                    operator: "equal".to_string(),
                    entity: table.to_string(),
                    values: format!("[\"{ip}\"]"),
                }],
                order_by: String::new(),
                limit: 1,
                offset: 0,
                order_direction: String::new(),
                joins: vec![],
                multiple_sort: vec![],
                pluck_object: HashMap::default(),
                date_format: String::new(),
            }),
        };

        let result = self.inner.get_by_filter(request, token).await?.count > 0;

        Ok(result)
    }

    pub(crate) async fn insert_ip_info(
        &mut self,
        ip: &str,
        ip_info: IpInfo,
        token: &str,
    ) -> Result<ResponseData, Error> {
        let table = "ip_info";

        let record = json!({
            "timestamp": Utc::now().to_rfc3339(),
            "ip": ip,
            "country": ip_info.country,
            "asn": ip_info.asn,
            "org": ip_info.org,
            "continent_code": ip_info.continent_code,
            "city": ip_info.city,
            "region": ip_info.region,
            "postal": ip_info.postal,
            "timezone": ip_info.timezone,
        })
        .to_string();

        let request = CreateRequest {
            params: Some(CreateParams {
                table: table.into(),
            }),
            query: Some(Query {
                pluck: String::from("id"),
                durability: String::from("soft"),
            }),
            body: Some(CreateBody {
                record,
                entity_prefix: String::from("IP"),
            }),
        };

        self.inner.create(request, token).await
    }
}
