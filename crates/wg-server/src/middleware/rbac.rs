use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use wg_shared::types::Role;

use super::auth::RequestContext;

impl RequestContext {
    /// Return `Ok(())` if the caller's role satisfies `required`.
    ///
    /// Returns a ready-to-return `403 Forbidden` response otherwise.
    /// Handlers call this at the top of their body:
    ///
    /// ```ignore
    /// ctx.require_role(Role::Admin)?;
    /// ```
    pub fn require_role(&self, required: Role) -> Result<(), Response> {
        if self.role.satisfies(required) {
            Ok(())
        } else {
            Err(forbidden())
        }
    }
}

fn forbidden() -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(json!({
            "error": {
                "message": "insufficient permissions",
                "code":    "FORBIDDEN"
            }
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn ctx(role: Role) -> RequestContext {
        RequestContext {
            user_id: Uuid::new_v4(),
            org_id:  Uuid::new_v4(),
            role,
        }
    }

    #[test]
    fn owner_satisfies_all() {
        let c = ctx(Role::Owner);
        assert!(c.require_role(Role::Owner).is_ok());
        assert!(c.require_role(Role::Admin).is_ok());
        assert!(c.require_role(Role::Operator).is_ok());
        assert!(c.require_role(Role::Viewer).is_ok());
    }

    #[test]
    fn viewer_only_satisfies_viewer() {
        let c = ctx(Role::Viewer);
        assert!(c.require_role(Role::Viewer).is_ok());
        assert!(c.require_role(Role::Operator).is_err());
        assert!(c.require_role(Role::Admin).is_err());
        assert!(c.require_role(Role::Owner).is_err());
    }

    #[test]
    fn operator_satisfies_operator_and_viewer() {
        let c = ctx(Role::Operator);
        assert!(c.require_role(Role::Viewer).is_ok());
        assert!(c.require_role(Role::Operator).is_ok());
        assert!(c.require_role(Role::Admin).is_err());
    }

    #[test]
    fn role_mismatch_returns_403() {
        let c   = ctx(Role::Viewer);
        let err = c.require_role(Role::Admin).unwrap_err();
        // Extract status code from the response.
        use axum::response::IntoResponse;
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }
}
