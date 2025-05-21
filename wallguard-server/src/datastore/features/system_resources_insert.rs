use crate::proto::wallguard::SystemResources;
use crate::{datastore::DatastoreWrapper};
use nullnet_libdatastore::{
    CreateBody, CreateParams, CreateRequest, Query,
    ResponseData,
};
use nullnet_liberror::{Error};
use serde_json::json;

impl DatastoreWrapper {
    pub async fn system_resources_insert(
        &self,
        token: &str,
        resources: SystemResources,
    ) -> Result<ResponseData, Error> {
        let record = json!({
            "timestamp": resources.timestamp,
            "num_cpus": resources.num_cpus,
            "global_cpu_usage": resources.global_cpu_usage,
            "cpu_usages": resources.cpu_usages,
            "total_memory": resources.total_memory,
            "used_memory": resources.used_memory,
            "total_disk_space": resources.total_disk_space,
            "available_disk_space": resources.available_disk_space,
            "read_bytes": resources.read_bytes,
            "written_bytes": resources.written_bytes,
            "temperatures": resources.temperatures,
        })
        .to_string();

        let request = CreateRequest {
            params: Some(CreateParams {
                table: String::from("system_resources"),
            }),
            query: Some(Query {
                pluck: String::new(),
                durability: String::from("soft"),
            }),
            body: Some(CreateBody {
                record,
                entity_prefix: String::from("SR"),
            }),
        };

        let response = self.inner.clone().create(request, token).await?;

        Ok(response)
    }
}
