use crate::datastore::Datastore;
use crate::protocol::wallguard_service::ConfigSnapshot;
use crate::utilities;
use crate::{control_service::service::WallGuardService, datastore::DeviceConfiguration};
use libfireparse::{Configuration, FileData, FireparseError, Parser, Platform};
use nullnet_liberror::Error;
use nullnet_libtoken::Token;
use tonic::{Request, Response, Status};

// @TODO
// Save & Update records "status": Active, Draft

impl WallGuardService {
    pub(crate) async fn handle_config_data_impl(
        &self,
        request: Request<ConfigSnapshot>,
    ) -> Result<Response<()>, Status> {
        let request = request.into_inner();

        let token =
            Token::from_jwt(&request.token).map_err(|_| Status::internal("Malformed JWT token"))?;

        let snapshot = request
            .files
            .into_iter()
            .map(|sf| FileData {
                filename: sf.filename,
                content: sf.contents,
            })
            .collect();

        let configuration = Parser::parse(Platform::PfSense, snapshot)
            .map_err(|e| match e {
                FireparseError::UnsupportedPlatform(message)
                | FireparseError::ParserError(message) => message,
            })
            .map_err(|err| Status::internal(err))?;

        let previous = self
            .context
            .datastore
            .obtain_config(&token.jwt, &token.account.device.id)
            .await
            .map_err(|err| Status::internal(err.to_str()))?;

        if let Some(mut prev) = previous {
            let digest = utilities::hash::md5_digest(&configuration.raw_content);

            if prev.digest == digest {
                prev.version = prev.version + 1;

                self.context
                    .datastore
                    .update_config(&token.jwt, &prev.id, &prev)
                    .await
                    .map_err(|err| Status::internal(err.to_str()))?;

                return Ok(Response::new(()));
            } else {
                insert_new_configuration(self.context.datastore.clone(), &token, &configuration)
                    .await
                    .map_err(|err| Status::internal(err.to_str()))?;

                return Ok(Response::new(()));
            }
        } else {
            insert_new_configuration(self.context.datastore.clone(), &token, &configuration)
                .await
                .map_err(|err| Status::internal(err.to_str()))?;

            return Ok(Response::new(()));
        }
    }
}

async fn insert_new_configuration(
    datastore: Datastore,
    token: &Token,
    conf: &Configuration,
) -> Result<(), Error> {
    let mut devcfg = DeviceConfiguration::default();

    devcfg.device_id = token.account.device.id.clone();
    devcfg.digest = utilities::hash::md5_digest(&conf.raw_content);
    devcfg.hostname = conf.hostname.clone();
    devcfg.raw_content = conf.raw_content.clone();
    devcfg.version = 0;

    let config_id = datastore.create_config(&token.jwt, &devcfg).await?;

    let result = tokio::join!(
        datastore.create_rules(&token.jwt, &conf.rules, &config_id),
        datastore.create_aliases(&token.jwt, &conf.aliases, &config_id),
        datastore.create_interfaces(&token.jwt, &conf.interfaces, &config_id)
    );

    result.0?;
    result.1?;
    result.2?;

    Ok(())
}
