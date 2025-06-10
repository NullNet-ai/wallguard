use chrono::Utc;
use nullnet_liberror::Error;
use nullnet_libipinfo::IpInfo;
use serde_json::json;

use crate::datastore::Datastore;
use crate::datastore::builders::CreateRequestBuilder;
use crate::datastore::db_tables::DBTable;

impl Datastore {
    pub async fn create_ip_info(
        &self,
        token: &str,
        ip_info: &IpInfo,
        ip: &str,
    ) -> Result<(), Error> {
        let json = json!({
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
        });

        let request = CreateRequestBuilder::new()
            .pluck(vec!["id"])
            .table(DBTable::IpInfos)
            .record(json.to_string())
            .entity_prefix("IP")
            .build();

        let _ = self.inner.clone().create(request, token).await?;

        Ok(())
    }
}
