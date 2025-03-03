use crate::datastore::DatastoreWrapper;
use crate::proto::wallguard::Log;
use nullnet_libdatastore::{
    BatchCreateBody, BatchCreateRequest, CreateBody, CreateParams, CreateRequest, Query,
    ResponseData,
};
use nullnet_liberror::{location, Error, ErrorHandler, Location};

impl DatastoreWrapper {
    pub async fn logs_insert(&self, token: &str, logs: Vec<Log>) -> Result<ResponseData, Error> {
        return match logs.as_slice() {
            [] => Ok(ResponseData {
                count: 0,
                data: "".to_string(),
                encoding: "".to_string(),
            }),
            [log] => logs_insert_single(&mut self.clone(), log.clone(), token).await,
            _ => logs_insert_batch(&mut self.clone(), logs.clone(), token).await,
        };
    }
}

async fn logs_insert_single(
    datastore: &mut DatastoreWrapper,
    log: Log,
    token: &str,
) -> Result<ResponseData, Error> {
    let record = serde_json::to_string(&log).handle_err(location!())?;

    let request = CreateRequest {
        params: Some(CreateParams {
            table: String::from("wallguard_logs"),
        }),
        query: Some(Query {
            pluck: String::from("id"),
            durability: String::from("soft"),
        }),
        body: Some(CreateBody {
            record,
            entity_prefix: String::from("LO"),
        }),
    };

    // println!("Attempt to send 1 log entry to the datastore");

    datastore.inner.create(request, &token).await
}

async fn logs_insert_batch(
    datastore: &mut DatastoreWrapper,
    logs: Vec<Log>,
    token: &str,
) -> Result<ResponseData, Error> {
    let records = serde_json::to_string(&logs).handle_err(location!())?;

    let request = BatchCreateRequest {
        params: Some(CreateParams {
            table: String::from("wallguard_logs"),
        }),
        query: Some(Query {
            pluck: String::new(),
            durability: String::from("soft"),
        }),
        body: Some(BatchCreateBody {
            records,
            entity_prefix: String::from("LO"),
        }),
    };

    // println!(
    //     "Attempt to send {} log entries to the datastore",
    //     logs.len()
    // );

    datastore.inner.batch_create(request, &token).await
}
