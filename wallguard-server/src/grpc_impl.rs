use crate::proto::wallguard::wall_guard_server::WallGuard;
use crate::proto::wallguard::{SampleMessage, SampleResponse};
use tonic::{Request, Response, Status};

pub struct WallGuardImpl;

#[tonic::async_trait]
impl WallGuard for WallGuardImpl {
    async fn sample(
        &self,
        request: Request<SampleMessage>,
    ) -> Result<Response<SampleResponse>, Status> {
        let SampleMessage { value } = request.into_inner();
        let response = SampleResponse { value };
        Ok(Response::new(response))
    }
}
