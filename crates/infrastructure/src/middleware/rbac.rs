//! Ubicación: `crates/infrastructure/src/middleware/rbac.rs`
//!
//! Descripción: Middleware de autorización basado en roles y permisos.
//!              Formato de permisos: resource:action (ej: devices:read)
//!
//! ADRs relacionados: 0003 (Axum 0.8), 0006 (RBAC)

use axum::{
    extract::Request,
    response::Response,
};
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Permission {
    pub resource: String,
    pub action: String,
}

impl Permission {
    pub fn from_str(perm: &str) -> Option<Self> {
        let parts: Vec<&str> = perm.split(':').collect();
        if parts.len() == 2 {
            Some(Self {
                resource: parts[0].to_string(),
                action: parts[1].to_string(),
            })
        } else {
            None
        }
    }

    pub fn matches(&self, resource: &str, action: &str) -> bool {
        self.resource == resource && self.action == action
    }
}

#[derive(Debug, Clone)]
pub struct UserPermissions {
    pub user_id: uuid::Uuid,
    pub role: String,
    pub permissions: HashSet<String>,
}

impl UserPermissions {
    pub fn new(user_id: uuid::Uuid, role: String, permissions: HashSet<String>) -> Self {
        Self {
            user_id,
            role,
            permissions,
        }
    }

    pub fn has_permission(&self, resource: &str, action: &str) -> bool {
        let permission = format!("{}:{}", resource, action);
        self.permissions.contains(&permission) || self.permissions.contains("*:*")
    }

    pub fn is_admin(&self) -> bool {
        self.role == "admin" || self.permissions.contains("*:*")
    }
}

pub struct RbacChecker {
    get_permissions: Arc<dyn Fn(uuid::Uuid) -> Option<UserPermissions> + Send + Sync>,
}

impl RbacChecker {
    pub fn new<F>(get_permissions: F) -> Self
    where
        F: Fn(uuid::Uuid) -> Option<UserPermissions> + Send + Sync + 'static,
    {
        Self {
            get_permissions: Arc::new(get_permissions),
        }
    }

    pub async fn check_permission(
        &self,
        request: &Request,
        resource: &str,
        action: &str,
    ) -> Result<(), RbacError> {
        let user_id = extract_user_id(request).ok_or(RbacError::NoUserId)?;

        let get_permissions = &self.get_permissions;
        let permissions = get_permissions(user_id).ok_or(RbacError::UserNotFound)?;

        if permissions.is_admin() {
            return Ok(());
        }

        if permissions.has_permission(resource, action) {
            Ok(())
        } else {
            Err(RbacError::PermissionDenied {
                resource: resource.to_string(),
                action: action.to_string(),
            })
        }
    }
}

#[derive(Debug)]
pub enum RbacError {
    NoUserId,
    UserNotFound,
    PermissionDenied { resource: String, action: String },
}

impl RbacError {
    pub fn into_response(self) -> Response {
        use axum::{
            http::StatusCode,
            response::IntoResponse,
            Json,
        };
        use serde_json::json;

        let (status, message) = match self {
            RbacError::NoUserId => (StatusCode::UNAUTHORIZED, "Authentication required".to_string()),
            RbacError::UserNotFound => (StatusCode::UNAUTHORIZED, "User not found".to_string()),
            RbacError::PermissionDenied { resource, action } => {
                (StatusCode::FORBIDDEN, format!("Permission denied: {}:{}", resource, action))
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

fn extract_user_id(request: &Request) -> Option<uuid::Uuid> {
    request.extensions().get::<uuid::Uuid>().copied()
}
