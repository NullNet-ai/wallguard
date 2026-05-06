use crate::datastore::{
    Datastore,
    db_tables::DBTable,
    generated::{
        BatchDeleteDeviceInstancesRequest, BatchDeleteParams, DeleteDeviceInstancesRequest,
        DeleteParams, DeleteQuery, FilterCriteria, FilterOperator,
        batch_delete_device_instances_request,
    },
};
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn delete_device_instance(
        &self,
        token: &str,
        instance_id: &str,
    ) -> Result<(), Error> {
        let request = DeleteDeviceInstancesRequest {
            params: Some(DeleteParams {
                id: instance_id.to_string(),
                table: DBTable::DeviceInstances.into(),
                r#type: String::new(),
            }),
            query: Some(DeleteQuery {
                is_permanent: String::new(),
            }),
        };

        let mut grpc_request = tonic::Request::new(request);
        grpc_request.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", token)
                .parse()
                .handle_err(location!())?,
        );

        let _ = self
            .inner
            .clone()
            .delete_device_instances(grpc_request)
            .await
            .handle_err(location!())?;

        Ok(())
    }

    pub async fn delete_all_device_instances(
        &self,
        token: &str,
        perfomed_by_root: bool,
    ) -> Result<(), Error> {
        let request = BatchDeleteDeviceInstancesRequest {
            params: Some(BatchDeleteParams {
                table: DBTable::DeviceInstances.into(),
                r#type: if perfomed_by_root {
                    "root".to_string()
                } else {
                    String::new()
                },
            }),
            body: Some(batch_delete_device_instances_request::BatchDeleteBody {
                advance_filters: vec![FilterCriteria {
                    r#type: "criteria".to_string(),
                    field: Some("status".to_string()),
                    entity: Some(DBTable::DeviceInstances.into()),
                    operator: Some(FilterOperator::Equal as i32),
                    values: vec!["\"Active\"".to_string()],
                    ..Default::default()
                }],
            }),
        };

        let mut grpc_request = tonic::Request::new(request);
        grpc_request.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", token)
                .parse()
                .handle_err(location!())?,
        );

        let _ = self
            .inner
            .clone()
            .batch_delete_device_instances(grpc_request)
            .await
            .handle_err(location!())?;

        Ok(())
    }
}
