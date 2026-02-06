use nullnet_libdatastore::DeleteRequestBuilder;
use nullnet_liberror::Error;

use crate::datastore::{Datastore, TunnelModel};

impl Datastore {
    pub async fn delete_tunnel(&self, token: &str, instance_id: &str) -> Result<(), Error> {
        let request = DeleteRequestBuilder::new()
            .id(instance_id)
            .table(TunnelModel::table())
            .build();

        let _ = self.inner.clone().delete(request, token).await?;

        Ok(())
    }
}
