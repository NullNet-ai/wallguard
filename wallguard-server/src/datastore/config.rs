#[derive(Debug, Clone)]
pub struct DatastoreConfig {
    pub email: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub tls: bool,
    pub token_leeway: u64,
}

impl DatastoreConfig {
    pub fn from_env() -> Self {
        Self {
            host: read_host_value_from_env(String::from("127.0.0.1")),
            port: read_port_value_from_env(6000),
            tls: real_tls_value_from_env(false),
            email: read_email_value_from_env(String::from("admin")),
            password: read_password_value_from_env(String::from("admin")),
            token_leeway: read_token_leeway_from_env(60 * 5),
        }
    }
}

fn read_host_value_from_env(default: String) -> String {
    std::env::var("DATASTORE_HOST").unwrap_or_else(|err| {
        eprintln!("Failed to read 'DATASTORE_HOST' env var: {err}. Using default value ...");
        default
    })
}

fn read_port_value_from_env(default: u16) -> u16 {
    match std::env::var("DATASTORE_PORT") {
        Ok(value) => value.parse::<u16>().unwrap_or_else(|err| {
            eprintln!(
                "Failed to parse 'DATASTORE_PORT' ({value}) as u16: {err}. Using default value ..."
            );
            default
        }),
        Err(err) => {
            eprintln!("Failed to read 'DATASTORE_PORT' env var: {err}. Using default value ...");
            default
        }
    }
}

fn real_tls_value_from_env(default: bool) -> bool {
    match std::env::var("DATASTORE_PORT") {
        Ok(value) => value.to_lowercase() == "true",
        Err(err) => {
            eprintln!("Failed to read 'DATASTORE_TLS' env var: {err}. Using default value ...");
            default
        }
    }
}

fn read_email_value_from_env(default: String) -> String {
    match std::env::var("DATASTORE_EMAIL") {
        Ok(value) => value,
        Err(err) => {
            eprintln!("Failed to read 'DATASTORE_EMAIL' env var: {err}. Using default value ...");
            default
        }
    }
}

fn read_password_value_from_env(default: String) -> String {
    match std::env::var("DATASTORE_PASSWORD") {
        Ok(value) => value,
        Err(err) => {
            eprintln!(
                "Failed to read 'DATASTORE_PASSWORD' env var: {err}. Using default value ..."
            );
            default
        }
    }
}

fn read_token_leeway_from_env(default: u64) -> u64 {
    match std::env::var("DATASTORE_TOKEN_LEEWAY") {
        Ok(value) => value.parse::<u64>().unwrap_or_else(|err| {
            eprintln!(
                "Failed to parse 'DATASTORE_TOKEN_LEEWAY' ({value}) as u64: {err}. Using default value ..."
            );
            default
        }),
        Err(err) => {
            eprintln!("Failed to read 'DATASTORE_TOKEN_LEEWAY' env var: {err}. Using default value ...");
            default
        }
    }
}
