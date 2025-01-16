use tonic::transport::Certificate;

#[cfg(debug_assertions)]
pub const BUFFER_SIZE: usize = 128;
#[cfg(not(debug_assertions))]
pub const BUFFER_SIZE: usize = 1024;

pub static CA_CERT: once_cell::sync::Lazy<Certificate> = once_cell::sync::Lazy::new(|| {
    Certificate::from_pem(
        std::fs::read_to_string("tls/ca.pem").expect("Failed to read CA certificate"),
    )
});

pub static UUID: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| uuid::Uuid::new_v4().to_string());
