use nullnet_liberror::{Error, ErrorHandler, Location, location};

use crate::datastore::{
    Datastore,
    generated::{LoginBody, LoginData, LoginParams, LoginRequest},
};

impl Datastore {
    pub async fn login(
        &self,
        app_id: &str,
        app_secret: &str,
        is_root: bool,
    ) -> Result<String, Error> {
        let request = LoginRequest {
            body: Some(LoginBody {
                data: Some(LoginData {
                    account_id: app_id.to_string(),
                    account_secret: app_secret.to_string(),
                }),
            }),
            params: Some(LoginParams {
                is_root: is_root.to_string(),
                t: String::new(),
            }),
        };

        let response = self
            .inner
            .clone()
            .login(request)
            .await
            .handle_err(location!())?
            .into_inner();

        validate_token(&response.token)?;

        Ok(response.token)
    }
}

fn validate_token(token: &str) -> Result<(), Error> {
    match token.is_empty() {
        true => Err("Unauthenticated: wrong app_id and/or app_secret").handle_err(location!()),
        false => Ok(()),
    }
}
