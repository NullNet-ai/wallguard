use crate::datastore::DatastoreWrapper;
use crate::proto::wallguard::SystemResource;
use nullnet_libdatastore::{
    BatchCreateBody, BatchCreateRequest, CreateBody, CreateParams, CreateRequest, Query,
    ResponseData,
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl DatastoreWrapper {
    pub async fn system_resources_insert(
        &self,
        token: &str,
        resources: Vec<SystemResource>,
    ) -> Result<ResponseData, Error> {
        match resources.as_slice() {
            [] => Ok(ResponseData {
                count: 0,
                data: String::new(),
                encoding: String::new(),
            }),
            [res] => resources_insert_single(&mut self.clone(), res.to_owned(), token).await,
            _ => resources_insert_batch(&mut self.clone(), resources, token).await,
        }
    }
}

async fn resources_insert_single(
    datastore: &mut DatastoreWrapper,
    res: SystemResource,
    token: &str,
) -> Result<ResponseData, Error> {
    let record = serde_json::to_string(&res).handle_err(location!())?;

    let request = CreateRequest {
        params: Some(CreateParams {
            table: String::from("system_resources"),
        }),
        query: Some(Query {
            pluck: String::from("id"),
            durability: String::from("soft"),
        }),
        body: Some(CreateBody {
            record,
            entity_prefix: String::from("SR"),
        }),
    };

    datastore.inner.create(request, token).await
}

async fn resources_insert_batch(
    datastore: &mut DatastoreWrapper,
    resources: Vec<SystemResource>,
    token: &str,
) -> Result<ResponseData, Error> {
    let records = serde_json::to_string(&resources).handle_err(location!())?;

    let request = BatchCreateRequest {
        params: Some(CreateParams {
            table: String::from("system_resources"),
        }),
        query: Some(Query {
            pluck: String::new(),
            durability: String::from("soft"),
        }),
        body: Some(BatchCreateBody {
            records,
            entity_prefix: String::from("SR"),
        }),
    };

    datastore.inner.batch_create(request, token).await
}
