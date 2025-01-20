use crate::proto::name::name_server::Name;
use crate::proto::name::{NameResponse, NameMessage};
use tonic::{Request, Response, Status};

pub struct NameImpl;

#[tonic::async_trait]
impl Name for NameImpl {
    async fn name(
        &self,
        request: Request<NameMessage>,
    ) -> Result<Response<NameResponse>, Status> {
        let NameMessage { value } = request.into_inner();
        let response = NameResponse { value };
        Ok(Response::new(response))
    }
}