use crate::datastore::db_tables::DBTable;
use crate::datastore::{Datastore, TunnelModel};

use nullnet_libdatastore::{AdvanceFilterBuilder, GetByFilterRequestBuilder};
use nullnet_liberror::Error;

impl Datastore {
    pub async fn does_tunnel_for_service_exist(
        &self,
        token: &str,
        service_id: &str,
        performed_by_root: bool,
    ) -> Result<bool, Error> {
        let filter = AdvanceFilterBuilder::new()
            .field("service_id")
            .values(format!("[\"{service_id}\"]"))
            .r#type("criteria")
            .operator("equal")
            .entity(DBTable::DeviceTunnels)
            .build();

        let request = GetByFilterRequestBuilder::new()
            .table(DBTable::DeviceTunnels)
            .plucks(TunnelModel::pluck())
            .advance_filter(filter)
            .performed_by_root(performed_by_root)
            .build();

        let response = self.inner.clone().get_by_filter(request, token).await?;

        Ok(response.count > 0)
    }
}
