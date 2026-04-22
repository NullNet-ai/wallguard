use std::sync::Arc;

use sqlx::PgPool;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::pki::Ca;

// Generated types from provisioning.proto.
mod proto {
    tonic::include_proto!("wallguard.provisioning");
}

pub use proto::provisioning_server::ProvisioningServer;
use proto::{
    provisioning_server::Provisioning,
    EnrollRequest, EnrollResponse,
};

/// gRPC Provisioning service — device enrollment.
///
/// Runs over server-authenticated TLS only (no client cert required).
pub struct ProvisioningService {
    pub pool:        PgPool,
    pub ca:          Arc<Ca>,
    /// PEM of the Intermediate CA returned to agents for pinning.
    pub ca_cert_pem: String,
    /// Canonical server hostname written into the agent config.toml.
    pub server_name: String,
}

#[tonic::async_trait]
impl Provisioning for ProvisioningService {
    async fn enroll(
        &self,
        request: Request<EnrollRequest>,
    ) -> Result<Response<EnrollResponse>, Status> {
        let req = request.into_inner();

        if req.installation_code.is_empty() || req.csr_pem.is_empty() {
            return Err(Status::invalid_argument("installation_code and csr_pem are required"));
        }

        // -----------------------------------------------------------------
        // Transaction: validate installation code, mark it used, insert device.
        // -----------------------------------------------------------------
        let mut tx = self.pool.begin().await.map_err(db_err)?;

        // Fetch the installation code row.  One atomic check: exists + not used + not expired.
        let row = sqlx::query_as::<_, (Uuid, Option<time::OffsetDateTime>)>(
            r#"
            SELECT org_id, used_at
            FROM   installation_codes
            WHERE  code = $1 AND expires_at > NOW()
            "#,
        )
        .bind(&req.installation_code)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db_err)?;

        let Some((org_id, used_at)) = row else {
            return Err(Status::not_found("installation code not found or expired"));
        };

        if used_at.is_some() {
            return Err(Status::already_exists("installation code already used"));
        }

        // Mark as used.
        sqlx::query("UPDATE installation_codes SET used_at = NOW() WHERE code = $1")
            .bind(&req.installation_code)
            .execute(&mut *tx)
            .await
            .map_err(db_err)?;

        // -----------------------------------------------------------------
        // Sign the CSR — server overrides O with the real org_id.
        // -----------------------------------------------------------------
        let (cert_pem, device_id, cert_expires_at) = self
            .ca
            .sign_enrollment_csr(&req.csr_pem, org_id)
            .map_err(|e| Status::invalid_argument(format!("invalid CSR: {e}")))?;

        let firewall_kind = normalise_firewall_kind(&req.firewall_kind);
        let display_name  = format!("Device {}", &device_id.to_string()[..8]);

        // -----------------------------------------------------------------
        // Insert device row.
        // -----------------------------------------------------------------
        sqlx::query(
            r#"
            INSERT INTO devices
                (id, org_id, display_name, firewall_kind, agent_version)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(device_id)
        .bind(org_id)
        .bind(&display_name)
        .bind(&firewall_kind)
        .bind(if req.agent_version.is_empty() { None } else { Some(&req.agent_version) })
        .execute(&mut *tx)
        .await
        .map_err(db_err)?;

        // Insert device certificate audit row.
        sqlx::query(
            r#"
            INSERT INTO device_certificates
                (device_id, cert_pem, issued_at, expires_at)
            VALUES ($1, $2, NOW(), $3)
            "#,
        )
        .bind(device_id)
        .bind(&cert_pem)
        .bind(cert_expires_at)
        .execute(&mut *tx)
        .await
        .map_err(db_err)?;

        tx.commit().await.map_err(db_err)?;

        tracing::info!(
            device_id = %device_id,
            org_id    = %org_id,
            firewall  = %firewall_kind,
            "device enrolled"
        );

        Ok(Response::new(EnrollResponse {
            device_id:       device_id.to_string(),
            device_cert_pem: cert_pem,
            ca_cert_pem:     self.ca_cert_pem.clone(),
            server_name:     self.server_name.clone(),
        }))
    }
}

fn db_err(e: sqlx::Error) -> Status {
    tracing::error!("db error during enrollment: {e}");
    Status::internal("internal error")
}

fn normalise_firewall_kind(s: &str) -> String {
    match s.to_lowercase().as_str() {
        "pfsense"  => "pfsense",
        "opnsense" => "opnsense",
        "nftables" => "nftables",
        _          => "none",
    }
    .to_owned()
}
