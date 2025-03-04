use super::token_wrapper::TokenWrapper;
use nullnet_libwallguard::WallGuardGrpcInterface;

pub async fn request_impl(
    addr: &str,
    port: u16,
    app_id: String,
    app_secret: String,
) -> Result<TokenWrapper, String> {
    let jwt: String = WallGuardGrpcInterface::new(addr, port)
        .await
        .login(app_id, app_secret)
        .await?;

    let token = TokenWrapper::from_jwt(jwt)?;
    Ok(token)
}
