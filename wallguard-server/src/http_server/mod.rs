mod common;
mod proxy;
mod remote_access_request;
mod remote_access_terminate;
mod ssh_gateway;

use crate::app_context::AppContext;
use actix_cors::Cors;
use actix_web::{App, HttpServer, http, web};
use proxy::proxy;
use remote_access_request::remote_access_request;
use remote_access_terminate::remote_access_terminate;
use rustls::ServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, ec_private_keys, pkcs8_private_keys, rsa_private_keys};
use std::io::{Seek, SeekFrom};
use std::net::{SocketAddr, TcpListener};
use std::{fs::File, io::BufReader};

const ADDR: &str = "0.0.0.0";
const PORT: u16 = 4444;

const DEFAULT_CERT_PATH: &str = "./dev/cert.pem";
const DEFAULT_KEY_PATH: &str = "./dev/key.pem";

pub async fn run_http_server(context: AppContext, tls: bool) {
    let cert_path =
        std::env::var("TLS_CERT_PATH").unwrap_or_else(|_| DEFAULT_CERT_PATH.to_string());
    let key_path = std::env::var("TLS_KEY_PATH").unwrap_or_else(|_| DEFAULT_KEY_PATH.to_string());

    let config = match load_tls_config(&cert_path, &key_path) {
        Some(cfg) => cfg,
        None => panic!(
            "Unable to load TLS config from {:?} and {:?}",
            cert_path, key_path
        ),
    };

    let app_state = web::Data::new(context);

    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "DELETE"])
            .allowed_headers(vec![
                http::header::CONTENT_TYPE,
                http::header::AUTHORIZATION,
            ])
            .max_age(3600);
        App::new()
            .app_data(app_state.clone())
            .wrap(cors)
            .route(
                "/v1/api/remote_access",
                web::post().to(remote_access_request),
            )
            .route(
                "/v1/api/remote_access",
                web::delete().to(remote_access_terminate),
            )
            .route("/v1/api/ssh", web::to(ssh_gateway::open_ssh_session))
            .default_service(web::to(proxy))
    });

    if tls {
        log::info!("HTTP API listening on https://{ADDR}:{PORT}");
        server
            .bind_rustls_0_23(format!("{ADDR}:{PORT}"), config)
            .unwrap()
            .run()
            .await
            .unwrap()
    } else {
        log::info!("HTTP API listening on http://{ADDR}:{PORT}");
        let addr: SocketAddr = format!("{ADDR}:{PORT}").parse().unwrap();
        let listener = TcpListener::bind(addr).unwrap();
        server.listen(listener).unwrap().run().await.unwrap()
    }
}

fn load_tls_config(cert_path: &str, key_path: &str) -> Option<ServerConfig> {
    let cert_file = File::open(cert_path).ok()?;
    let key_file = File::open(key_path).ok()?;

    let mut cert_reader = BufReader::new(cert_file);
    let mut key_reader = BufReader::new(key_file);

    let cert_chain: Vec<CertificateDer> = certs(&mut cert_reader)
        .map(|r| r.ok())
        .collect::<Option<Vec<_>>>()?;

    if cert_chain.is_empty() {
        log::warn!("No certificates found.");
        return None;
    }

    let mut keys: Vec<PrivateKeyDer> = pkcs8_private_keys(&mut key_reader)
        .map(|r| r.ok().map(PrivateKeyDer::Pkcs8))
        .collect::<Option<Vec<_>>>()
        .unwrap_or_else(Vec::new);

    if keys.is_empty() {
        key_reader.get_mut().seek(SeekFrom::Start(0)).ok()?;
        keys = rsa_private_keys(&mut key_reader)
            .map(|r| r.ok().map(PrivateKeyDer::Pkcs1))
            .collect::<Option<Vec<_>>>()
            .unwrap_or_else(Vec::new);
    }

    if keys.is_empty() {
        key_reader.get_mut().seek(SeekFrom::Start(0)).ok()?;
        keys = ec_private_keys(&mut key_reader)
            .map(|r| r.ok().map(PrivateKeyDer::Sec1))
            .collect::<Option<Vec<_>>>()
            .unwrap_or_else(Vec::new);
    }

    if keys.is_empty() {
        log::warn!("No supported private keys found in any format.");
        return None;
    }

    if keys.is_empty() {
        log::warn!("No private keys found.");
        return None;
    }

    ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, keys.remove(0))
        .ok()
}
