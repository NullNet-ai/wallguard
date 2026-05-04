
use tonic::transport::Channel;
use super::generated::store_service_client::StoreServiceClient;

pub struct DatastoreClient {
    pub client: StoreServiceClient<Channel>,
}
