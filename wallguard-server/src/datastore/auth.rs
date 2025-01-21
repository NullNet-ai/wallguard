use tonic::transport::Channel;

use crate::proto::dna_store::{
    store_service_client::StoreServiceClient, LoginBody, LoginData, LoginRequest,
};

use super::token::AuthToken;

pub async fn authentication_request_impl(
    mut client: StoreServiceClient<Channel>,
    email: String,
    password: String,
    token_leeway: u64,
) -> Result<AuthToken, String> {
    let request = LoginRequest {
        body: Some(LoginBody {
            data: Some(LoginData { email, password }),
        }),
    };

    let response = client
        .login(request)
        .await
        .map_err(|e| format!("Login request failed: {e}"))?;

    AuthToken::new(response.into_inner().token, token_leeway)
}
