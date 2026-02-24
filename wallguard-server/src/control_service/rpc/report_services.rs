use std::collections::HashSet;

use crate::{control_service::service::WallGuardService, datastore::ServiceInfo};
use nullnet_libtoken::Token;
use tonic::{Request, Response, Status};
use wallguard_common::protobuf::wallguard_service::ServicesMessage;

impl WallGuardService {
    pub(crate) async fn report_services_impl(
        &self,
        request: Request<ServicesMessage>,
    ) -> Result<Response<()>, Status> {
        let data = request.into_inner();

        let token =
            Token::from_jwt(&data.token).map_err(|_| Status::internal("Malformed JWT token"))?;

        let device = self
            .ensure_device_exists_and_authrorized(&token)
            .await
            .map_err(|err| Status::internal(err.to_str()))?;

        let new_list: Vec<ServiceInfo> = data
            .services
            .into_iter()
            .map(|value| ServiceInfo::new(value, device.id.clone()))
            .collect();

        let new_list = unique_services(new_list);

        let old_list = self
            .context
            .datastore
            .obtain_services(&token.jwt, &device.id, false)
            .await
            .map_err(|err| Status::internal(err.to_str()))?
            .unwrap_or_default();

        let new_keys: HashSet<_> = new_list.iter().map(service_key).collect();
        let old_keys: HashSet<_> = old_list.iter().map(service_key).collect();

        let keys_to_insert: HashSet<_> = new_keys.difference(&old_keys).cloned().collect();
        let keys_to_delete: HashSet<_> = old_keys.difference(&new_keys).cloned().collect();

        let to_insert: Vec<_> = new_list
            .into_iter()
            .filter(|svc| keys_to_insert.contains(&service_key(svc)))
            .collect();

        let to_delete: Vec<_> = old_list
            .into_iter()
            .filter(|svc| keys_to_delete.contains(&service_key(svc)))
            .collect();

        for sd in to_delete.iter() {
            self.context
                .tunnels_manager
                .on_service_deleted(&sd.id)
                .await;
        }

        self.context
            .datastore
            .delete_services(&token.jwt, &to_delete)
            .await
            .map_err(|_| Status::internal("Datastore operation failed"))?;

        self.context
            .datastore
            .create_services(&token.jwt, &to_insert)
            .await
            .map_err(|_| Status::internal("Datastore operation failed"))?;

        Ok(Response::new(()))
    }
}

fn service_key(svc: &ServiceInfo) -> (String, u16, String, String) {
    (
        svc.address.clone(),
        svc.port,
        svc.protocol.clone(),
        svc.program.clone(),
    )
}

fn unique_services(services: Vec<ServiceInfo>) -> Vec<ServiceInfo> {
    let mut seen: HashSet<(String, u16, String, String)> = HashSet::new();
    let mut unique: Vec<ServiceInfo> = Vec::new();

    for svc in services.into_iter() {
        let key = service_key(&svc);

        if seen.insert(key) {
            unique.push(svc);
        }
    }

    unique
}
