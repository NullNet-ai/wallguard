use crate::datastore::{Datastore, TunnelModel};
use nullnet_libdatastore::CreateRequestBuilder;
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn create_tunnel(&self, token: &str, tunnel: &TunnelModel) -> Result<(), Error> {
        let mut json = serde_json::to_value(tunnel).handle_err(location!())?;
        json.as_object_mut().unwrap().remove("id");

        let request = CreateRequestBuilder::new()
            .pluck(TunnelModel::pluck())
            .table(TunnelModel::table())
            .record(json.to_string())
            .build();

        let _ = self.inner.clone().create(request, token).await?;
        Ok(())
    }
}
