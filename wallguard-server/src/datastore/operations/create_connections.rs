use crate::datastore::{
    Datastore,
    db_tables::DBTable,
    generated::{
        BatchInsertConnectionsRequest, BatchInsertParams, BatchInsertQuery, Connections,
        batch_insert_connections_request,
    },
};
use crate::token::Token;
use nullnet_liberror::{Error, ErrorHandler, Location, location};
use nullnet_libipinfo::get_ip_to_lookup;
use std::net::IpAddr;
use wallguard_common::protobuf::wallguard_service::ConnectionsData;

impl Datastore {
    pub async fn create_connections(
        &self,
        token: &str,
        auth_token: &Token,
        data: ConnectionsData,
    ) -> Result<(), Error> {
        let device_id = auth_token
            .account
            .device_id()
            .unwrap_or_default()
            .to_string();

        let records: Vec<Connections> = data
            .connections
            .into_iter()
            .map(|conn| {
                let src: Option<IpAddr> = conn.source_ip.parse().ok();
                let dst: Option<IpAddr> = conn.destination_ip.parse().ok();
                let remote_ip = src
                    .zip(dst)
                    .and_then(|(s, d)| get_ip_to_lookup(s, d))
                    .map(|ip| ip.to_string());

                Connections {
                    device_id: Some(device_id.clone()),
                    interface_name: Some(conn.interface),
                    source_ip: Some(conn.source_ip),
                    destination_ip: Some(conn.destination_ip),
                    source_port: conn.source_port.map(|p| p as i32),
                    destination_port: conn.destination_port.map(|p| p as i32),
                    protocol: Some(conn.protocol),
                    timestamp: Some(conn.timestamp),
                    total_packet: Some(conn.total_packet as i32),
                    total_byte: Some(conn.total_byte as i32),
                    remote_ip,
                    status: Some(String::from("Active")),
                    ..Default::default()
                }
            })
            .collect();

        let request = BatchInsertConnectionsRequest {
            params: Some(BatchInsertParams {
                table: DBTable::Connections.into(),
                r#type: String::new(),
            }),
            query: Some(BatchInsertQuery {
                pluck: String::new(),
            }),
            body: Some(batch_insert_connections_request::BatchBody {
                connections: records,
            }),
        };

        let mut grpc_request = tonic::Request::new(request);
        grpc_request.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", token)
                .parse()
                .handle_err(location!())?,
        );

        self.inner
            .clone()
            .batch_insert_connections(grpc_request)
            .await
            .handle_err(location!())
            .map(|_| ())
    }
}
