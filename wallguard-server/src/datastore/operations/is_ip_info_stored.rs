use crate::datastore::Datastore;
use crate::datastore::db_tables::DBTable;
use nullnet_libdatastore::AdvanceFilterBuilder;
use nullnet_libdatastore::GetByFilterRequestBuilder;
use nullnet_liberror::Error;

impl Datastore {
    pub async fn is_ip_info_stored(&self, ip: &str, token: &str) -> Result<bool, Error> {
        let filter = AdvanceFilterBuilder::new()
            .field("ip")
            .values(format!("[\"{ip}\"]"))
            .r#type("criteria")
            .operator("equal")
            .entity(DBTable::IpInfos)
            .build();

        let request = GetByFilterRequestBuilder::new()
            .table(DBTable::IpInfos)
            .plucks(vec!["id"])
            .limit(1)
            .advance_filter(filter)
            .build();

        let response = self.inner.clone().get_by_filter(request, token).await?;

        Ok(response.count > 0)
    }
}
