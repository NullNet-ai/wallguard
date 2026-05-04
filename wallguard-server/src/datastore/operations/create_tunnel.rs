use crate::datastore::{
    Datastore, TunnelModel,
    db_tables::DBTable,
    generated::{CreateDeviceTunnelsRequest, CreateParams, CreateQuery, DeviceTunnels},
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn create_tunnel(&self, token: &str, tunnel: &TunnelModel) -> Result<String, Error> {
        let request = CreateDeviceTunnelsRequest {
            device_tunnels: Some(DeviceTunnels {
                device_id: Some(tunnel.device_id.clone()),
                tunnel_type: Some(tunnel.tunnel_type.to_string()),
                service_id: Some(tunnel.service_id.clone()),
                tunnel_status: Some(tunnel.tunnel_status.to_string()),
                ..Default::default()
            }),
            params: Some(CreateParams {
                table: DBTable::DeviceTunnels.into(),
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
            .create_device_tunnels(request)
            .await
            .handle_err(location!())?
            .into_inner();

        let id = response
            .data
            .and_then(|d| d.id)
            .ok_or("Missing 'id' in tunnel response")
            .handle_err(location!())?;

        Ok(id)
    }
}
