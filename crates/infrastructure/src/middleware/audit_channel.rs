//! Ubicación: `crates/infrastructure/src/middleware/audit_channel.rs`
//!
//! Descripción: Canal asíncrono para escritura no-bloqueante de audit logs a PostgreSQL.
//!
//! ADRs relacionados: 0003 (Axum 0.8), 0006 (RBAC)

use domain::entities::AuditLog;
use domain::ports::AuditRepository;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub struct AuditChannel {
    sender: mpsc::Sender<AuditLog>,
    worker_handle: JoinHandle<()>,
}

impl AuditChannel {
    pub fn new<A: AuditRepository + 'static>(
        repository: Arc<A>,
        buffer_size: usize,
    ) -> Self {
        let (tx, mut rx) = mpsc::channel::<AuditLog>(buffer_size);

        let worker_handle = tokio::spawn(async move {
            while let Some(entry) = rx.recv().await {
                if let Err(e) = repository.log(&entry) {
                    tracing::error!(error = %e, "Failed to write audit log: {}", e);
                }
            }
        });

        Self {
            sender: tx,
            worker_handle,
        }
    }

    pub async fn send(&self, entry: AuditLog) -> Result<(), AuditChannelError> {
        self.sender.send(entry).await.map_err(|_| AuditChannelError::Closed)
    }

    pub async fn shutdown(self) {
        drop(self.sender);
        let _ = self.worker_handle.await;
    }
}

#[derive(Debug)]
pub enum AuditChannelError {
    Closed,
}

#[derive(Clone)]
pub struct AuditLogBuilder {
    user_id: Option<uuid::Uuid>,
    action: String,
    resource: String,
    resource_id: Option<String>,
    details: Option<String>,
    ip_address: Option<String>,
    user_agent: Option<String>,
}

impl AuditLogBuilder {
    pub fn new(action: impl Into<String>, resource: impl Into<String>) -> Self {
        Self {
            user_id: None,
            action: action.into(),
            resource: resource.into(),
            resource_id: None,
            details: None,
            ip_address: None,
            user_agent: None,
        }
    }

    pub fn user_id(mut self, user_id: uuid::Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn resource_id(mut self, resource_id: impl Into<String>) -> Self {
        self.resource_id = Some(resource_id.into());
        self
    }

    pub fn details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn ip_address(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    pub fn user_agent(mut self, ua: impl Into<String>) -> Self {
        self.user_agent = Some(ua.into());
        self
    }

    pub fn build(self) -> AuditLog {
        AuditLog {
            id: uuid::Uuid::new_v4(),
            user_id: self.user_id,
            action: self.action,
            resource: self.resource,
            resource_id: self.resource_id,
            details: self.details,
            ip_address: self.ip_address,
            user_agent: self.user_agent,
            created_at: time::OffsetDateTime::now_utc(),
        }
    }
}