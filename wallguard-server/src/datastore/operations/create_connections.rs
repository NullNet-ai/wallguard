use crate::datastore::db_tables::DBTable;
use crate::datastore::{Datastore, builders::BatchCreateRequestBuilder};
use crate::traffic_handler::parsed_message::ParsedMessage;
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn create_connections(&self, token: &str, data: ParsedMessage) -> Result<(), Error> {
        let records = serde_json::to_string(&data).handle_err(location!())?;

        let request = BatchCreateRequestBuilder::new()
            .table(DBTable::Connections)
            .entity_prefix("CN")
            .records(records)
            .build();

        self.inner
            .clone()
            .batch_create(request, token)
            .await
            .map(|_| ())
    }
}
