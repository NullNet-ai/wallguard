use crate::datastore::{
    Datastore,
    db_tables::DBTable,
    generated::{DeleteDeviceTunnelsRequest, DeleteParams, DeleteQuery},
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn delete_tunnel(&self, token: &str, tunnel_id: &str) -> Result<(), Error> {
        let request = DeleteDeviceTunnelsRequest {
            params: Some(DeleteParams {
                id: tunnel_id.to_string(),
                table: DBTable::DeviceTunnels.into(),
                r#type: String::new(),
            }),
            query: Some(DeleteQuery {
                is_permanent: String::new(),
            }),
        };

        let _ = self
            .inner
            .clone()
            .delete_device_tunnels(request)
            .await
            .handle_err(location!())?;

        Ok(())
    }
}
