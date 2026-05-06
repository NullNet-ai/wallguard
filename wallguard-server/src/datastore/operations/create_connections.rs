use crate::datastore::{
    Datastore,
    db_tables::DBTable,
    generated::{
        BatchInsertConnectionsRequest, BatchInsertParams, BatchInsertQuery, Connections,
        batch_insert_connections_request,
    },
};
use crate::traffic_handler::parsed_message::ParsedMessage;
use nullnet_liberror::{Error, ErrorHandler, Location, location};

impl Datastore {
    pub async fn create_connections(&self, token: &str, data: ParsedMessage) -> Result<(), Error> {
        let records: Vec<Connections> = data
            .records
            .into_iter()
            .map(|record| Connections {
                device_id: Some(record.connection_key.device_id),
                interface_name: Some(record.connection_key.interface_name),
                source_ip: Some(record.connection_key.ip_header.source_ip.to_string()),
                destination_ip: Some(record.connection_key.ip_header.destination_ip.to_string()),
                source_port: record
                    .connection_key
                    .transport_header
                    .source_port
                    .map(|p| p as i32),
                destination_port: record
                    .connection_key
                    .transport_header
                    .destination_port
                    .map(|p| p as i32),
                protocol: Some(String::from(record.connection_key.transport_header.protocol)),
                timestamp: Some(record.connection_value.timestamp),
                total_packet: Some(record.connection_value.total_packet as i32),
                total_byte: Some(record.connection_value.total_byte as i32),
                remote_ip: record.connection_value.remote_ip.map(|ip| ip.to_string()),
                ..Default::default()
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
            format!("Bearer {}", token).parse().handle_err(location!())?,
        );

        self.inner
            .clone()
            .batch_insert_connections(grpc_request)
            .await
            .handle_err(location!())
            .map(|_| ())
    }
}
