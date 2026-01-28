use crate::{
    datastore::{Datastore, TunnelModel},
    utilities,
};
use nullnet_libdatastore::CreateRequestBuilder;
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn create_tunnel(&self, token: &str, tunnel: &TunnelModel) -> Result<String, Error> {
        let mut json = serde_json::to_value(tunnel).handle_err(location!())?;
        json.as_object_mut().unwrap().remove("id");

        let request = CreateRequestBuilder::new()
            .durability("hard")
            .pluck(TunnelModel::pluck())
            .table(TunnelModel::table())
            .record(json.to_string())
            .build();

        let response = self.inner.clone().create(request, token).await?;

        let json_data = utilities::json::parse_string(&response.data)?;
        let value = utilities::json::first_element_from_array(&json_data)?;
        let retval = serde_json::from_value::<TunnelModel>(value).handle_err(location!())?;

        Ok(retval.id)
    }
}
