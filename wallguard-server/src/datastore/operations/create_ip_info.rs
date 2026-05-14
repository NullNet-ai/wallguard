use crate::datastore::{
    Datastore,
    db_tables::DBTable,
    generated::{CreateIpInfosRequest, CreateParams, CreateQuery, IpInfos},
};
use chrono::Utc;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use nullnet_libipinfo::IpInfo;

impl Datastore {
    pub async fn create_ip_info(
        &self,
        token: &str,
        ip_info: &IpInfo,
        ip: &str,
    ) -> Result<(), Error> {
        let request = CreateIpInfosRequest {
            ip_infos: Some(IpInfos {
                timestamp: Some(Utc::now().to_rfc3339()),
                ip: Some(ip.to_string()),
                country: ip_info.country.clone(),
                asn: ip_info.asn.clone(),
                org: ip_info.org.clone(),
                continent_code: ip_info.continent_code.clone(),
                city: ip_info.city.clone(),
                region: ip_info.region.clone(),
                postal: ip_info.postal.clone(),
                timezone: ip_info.timezone.clone(),
                ..Default::default()
            }),
            params: Some(CreateParams {
                table: DBTable::IpInfos.into(),
                r#type: String::new(),
            }),
            query: Some(CreateQuery {
                pluck: String::new(),
                ..Default::default()
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
            .create_ip_infos(grpc_request)
            .await
            .handle_err(location!())?;

        Ok(())
    }
}
