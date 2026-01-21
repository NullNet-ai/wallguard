use crate::datastore::Datastore;
use crate::{control_service::service::WallGuardService, datastore::DeviceConfiguration};
use futures_util::future::try_join_all;
use nullnet_liberror::Error;
use nullnet_libtoken::Token;
use tonic::{Request, Response, Status};
use wallguard_common::protobuf::wallguard_models::{Alias, Configuration};
use wallguard_common::protobuf::wallguard_service::{ConfigSnapshot, ConfigStatus};

// @TODO
// Save & Update records "status": Active, Draft

impl WallGuardService {
    pub(crate) async fn handle_config_data_impl(
        &self,
        request: Request<ConfigSnapshot>,
    ) -> Result<Response<()>, Status> {
        let request = request.into_inner();

        let status = request.status();

        let Some(configuration) = request.configuration else {
            return Err(Status::internal("No configuration has been provided"));
        };

        let token =
            Token::from_jwt(&request.token).map_err(|_| Status::internal("Malformed JWT token"))?;

        let _ = self
            .ensure_device_exists_and_authrorized(&token)
            .await
            .map_err(|err| Status::internal(err.to_str()))?;

        let previous = self
            .context
            .datastore
            .obtain_config(&token.jwt, &token.account.device.as_ref().unwrap().id)
            .await
            .map_err(|err| Status::internal(err.to_str()))?;

        if let Some(mut prev) = previous {
            if prev.digest == configuration.digest {
                prev.version += 1;

                self.context
                    .datastore
                    .update_config(&token.jwt, &prev.id, &prev)
                    .await
                    .map_err(|err| Status::internal(err.to_str()))?;

                self.context
                    .datastore
                    .update_rules_status(&token.jwt, &prev.id, status)
                    .await
                    .map_err(|err| Status::internal(err.to_str()))?;

                Ok(Response::new(()))
            } else {
                insert_new_configuration(
                    self.context.datastore.clone(),
                    &token,
                    &configuration,
                    status,
                )
                .await
                .map_err(|err| Status::internal(err.to_str()))?;

                Ok(Response::new(()))
            }
        } else {
            insert_new_configuration(
                self.context.datastore.clone(),
                &token,
                &configuration,
                status,
            )
            .await
            .map_err(|err| Status::internal(err.to_str()))?;

            Ok(Response::new(()))
        }
    }
}

async fn insert_new_configuration(
    datastore: Datastore,
    token: &Token,
    conf: &Configuration,
    status: ConfigStatus,
) -> Result<(), Error> {
    let devcfg = DeviceConfiguration {
        device_id: token.account.device.as_ref().unwrap().id.clone(),
        digest: conf.digest.clone(),
        hostname: conf.hostname.clone(),
        version: 0,
        ..Default::default()
    };

    let config_id = datastore.create_config(&token.jwt, &devcfg).await?;

    let result = tokio::join!(
        datastore.create_filter_rules(&token.jwt, &conf.filter_rules, &config_id, status),
        datastore.create_nat_rules(&token.jwt, &conf.nat_rules, &config_id, status),
        create_aliases_inner(datastore.clone(), &token.jwt, &conf.aliases, &config_id),
        datastore.create_interfaces(&token.jwt, &conf.interfaces, &config_id)
    );

    result.0?;
    result.1?;
    result.2?;
    result.3?;

    Ok(())
}

async fn create_aliases_inner(
    datastore: Datastore,
    token: &str,
    aliases: &[Alias],
    config_id: &str,
) -> Result<(), Error> {
    let futures = aliases
        .iter()
        .map(|alias| datastore.create_alias(token, alias, config_id));

    try_join_all(futures).await?;
    Ok(())
}
