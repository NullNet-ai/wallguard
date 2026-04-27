use std::io::Write as _;
use std::net::{IpAddr, Ipv4Addr};
use std::path::Path;

use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType, Ia5String, IsCa, KeyPair,
    SanType,
};
use tracing::{info, warn};

/// Generate a self-signed dev PKI (CA + server cert) and write the four PEM
/// files to the configured paths.  No-ops if `ca_cert_path` already exists —
/// safe to call on every startup.
pub fn ensure_dev_pki(
    ca_cert_path:     &str,
    ca_key_path:      &str,
    server_cert_path: &str,
    server_key_path:  &str,
    server_name:      &str,
) {
    if Path::new(ca_cert_path).exists() {
        return;
    }

    warn!("dev PKI not found — generating self-signed certificates (NOT for production)");

    // CA keypair + self-signed cert
    let ca_key = gen_key("CA key");
    let mut ca_params = CertificateParams::default();
    let mut ca_dn = DistinguishedName::new();
    ca_dn.push(DnType::CommonName, "WallGuard Dev CA");
    ca_params.distinguished_name = ca_dn;
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    let ca_cert = ca_params.self_signed(&ca_key).unwrap_or_else(|e| fatal("CA cert", e));

    // Server keypair + cert signed by CA
    let srv_key = gen_key("server key");
    let mut srv_params = CertificateParams::default();
    let mut srv_dn = DistinguishedName::new();
    srv_dn.push(DnType::CommonName, server_name);
    srv_params.distinguished_name = srv_dn;
    srv_params.subject_alt_names = vec![
        SanType::DnsName(Ia5String::try_from("localhost").expect("localhost ia5")),
        SanType::DnsName(Ia5String::try_from(server_name).expect("server_name ia5")),
        SanType::IpAddress(IpAddr::V4(Ipv4Addr::LOCALHOST)),
    ];
    // Also add the server_name as SAN if it differs from "localhost".
    let srv_cert = srv_params
        .signed_by(&srv_key, &ca_cert, &ca_key)
        .unwrap_or_else(|e| fatal("server cert", e));

    write_pem(ca_cert_path,     &ca_cert.pem());
    write_pem(ca_key_path,      &ca_key.serialize_pem());
    write_pem(server_cert_path, &srv_cert.pem());
    write_pem(server_key_path,  &srv_key.serialize_pem());

    info!(
        ca_cert = ca_cert_path,
        server_cert = server_cert_path,
        "dev PKI generated"
    );
}

/// Seed an initial organisation + owner user if the `users` table is empty.
///
/// Reads credentials from env vars with safe dev defaults:
///   `SEED_ADMIN_EMAIL`    (default: admin@wallguard.local)
///   `SEED_ADMIN_PASSWORD` (default: password123)
///   `SEED_ADMIN_NAME`     (default: Admin)
///   `SEED_ORG_NAME`       (default: WallGuard)
///
/// No-ops (silently) if any users already exist.
pub async fn ensure_initial_user(pool: &sqlx::PgPool) {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    if count.0 > 0 {
        return;
    }

    let email    = env_or("SEED_ADMIN_EMAIL",    "admin@wallguard.local");
    let password = env_or("SEED_ADMIN_PASSWORD", "password123");
    let name     = env_or("SEED_ADMIN_NAME",     "Admin");
    let org_name = env_or("SEED_ORG_NAME",       "WallGuard");

    let hash = match crate::auth::password::hash_password(&password) {
        Ok(h)  => h,
        Err(e) => { tracing::error!("seed: password hash failed: {e}"); return; }
    };

    let result = async {
        let org_id: uuid::Uuid = sqlx::query_scalar(
            "INSERT INTO organizations (name) VALUES ($1) RETURNING id",
        )
        .bind(&org_name)
        .fetch_one(pool)
        .await?;

        sqlx::query(
            "INSERT INTO users (org_id, email, password_hash, display_name, role) \
             VALUES ($1, $2, $3, $4, 'owner')",
        )
        .bind(org_id)
        .bind(&email)
        .bind(&hash)
        .bind(&name)
        .execute(pool)
        .await?;

        Ok::<_, sqlx::Error>(())
    }
    .await;

    match result {
        Ok(_) => {
            warn!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            warn!(" First-run seed complete");
            warn!("   Email    : {email}");
            warn!("   Password : {password}");
            warn!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        }
        Err(e) => tracing::error!("seed: database error: {e}"),
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn gen_key(label: &str) -> KeyPair {
    KeyPair::generate().unwrap_or_else(|e| {
        eprintln!("dev PKI: cannot generate {label}: {e}");
        std::process::exit(1);
    })
}

fn fatal(label: &str, e: impl std::fmt::Display) -> ! {
    eprintln!("dev PKI: cannot generate {label}: {e}");
    std::process::exit(1);
}

fn write_pem(path: &str, data: &str) {
    if let Some(parent) = Path::new(path).parent() {
        std::fs::create_dir_all(parent).unwrap_or_else(|e| {
            eprintln!("dev PKI: cannot create directory {}: {e}", parent.display());
            std::process::exit(1);
        });
    }
    let mut f = std::fs::File::create(path).unwrap_or_else(|e| {
        eprintln!("dev PKI: cannot write {path}: {e}");
        std::process::exit(1);
    });
    f.write_all(data.as_bytes()).unwrap_or_else(|e| {
        eprintln!("dev PKI: write error {path}: {e}");
        std::process::exit(1);
    });
}
