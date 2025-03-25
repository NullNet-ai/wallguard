use nullnet_libtoken::Token;
use nullnet_libwallguard::WallGuardGrpcInterface;

pub async fn request_impl(
    addr: &str,
    port: u16,
    app_id: String,
    app_secret: String,
) -> Result<Token, String> {
    let jwt: String = WallGuardGrpcInterface::new(addr, port)
        .await
        .login(app_id, app_secret)
        .await?;

    let token = Token::from_jwt(jwt.as_str())?;
    Ok(token)
}
