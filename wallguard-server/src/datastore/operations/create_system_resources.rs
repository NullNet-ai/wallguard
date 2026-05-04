use crate::datastore::{
    Datastore,
    db_tables::DBTable,
    generated::{
        BatchInsertParams, BatchInsertQuery, BatchInsertSystemResourcesRequest, SystemResources,
        batch_insert_system_resources_request,
    },
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use wallguard_common::protobuf::wallguard_service::SystemResource;

impl Datastore {
    pub async fn create_system_resources(
        &self,
        token: &str,
        resources: Vec<SystemResource>,
        device_id: String,
    ) -> Result<(), Error> {
        if resources.is_empty() {
            return Ok(());
        }

        let records: Vec<SystemResources> = resources
            .into_iter()
            .map(|res| SystemResources {
                timestamp: Some(res.timestamp),
                num_cpus: Some(res.num_cpus as i32),
                global_cpu_usage: Some(res.global_cpu_usage.to_string()),
                cpu_usages: Some(res.cpu_usages),
                total_memory: Some(res.total_memory.to_string()),
                used_memory: Some(res.used_memory.to_string()),
                total_disk_space: Some(res.total_disk_space.to_string()),
                available_disk_space: Some(res.available_disk_space.to_string()),
                read_bytes: Some(res.read_bytes.to_string()),
                written_bytes: Some(res.written_bytes.to_string()),
                temperatures: Some(res.temperatures),
                device_id: Some(device_id.clone()),
                ..Default::default()
            })
            .collect();

        let request = BatchInsertSystemResourcesRequest {
            params: Some(BatchInsertParams {
                table: DBTable::SystemResources.into(),
                r#type: String::new(),
            }),
            query: Some(BatchInsertQuery {
                pluck: String::new(),
            }),
            body: Some(batch_insert_system_resources_request::BatchBody {
                system_resources: records,
            }),
        };

        let _ = self
            .inner
            .clone()
            .batch_insert_system_resources(request)
            .await
            .handle_err(location!())?;

        Ok(())
    }
}
