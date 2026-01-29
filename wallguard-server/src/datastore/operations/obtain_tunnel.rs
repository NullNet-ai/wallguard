use crate::datastore::db_tables::DBTable;
use crate::datastore::{Datastore, TunnelModel};
use crate::utilities::json;
use nullnet_libdatastore::GetByIdRequestBuilder;
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn obtain_tunnel(
        &self,
        token: &str,
        tunnel_id: &str,
        performed_by_root: bool,
    ) -> Result<Option<TunnelModel>, Error> {
        let request = GetByIdRequestBuilder::new()
            .table(DBTable::DeviceTunnels)
            .durability("hard")
            .id(tunnel_id)
            .pluck(TunnelModel::pluck())
            .performed_by_root(performed_by_root)
            .build();

        let response = self.inner.clone().get_by_id(request, token).await?;

        if response.count == 0 {
            return Ok(None);
        }

        let json_data = json::parse_string(&response.data)?;
        let data = json::first_element_from_array(&json_data)?;

        let tunnel = serde_json::from_value::<TunnelModel>(data).handle_err(location!())?;
        Ok(Some(tunnel))
    }
}
