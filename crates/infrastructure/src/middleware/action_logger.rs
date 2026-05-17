//! Ubicación: `crates/infrastructure/src/middleware/action_logger.rs`
//!
//! Descripción: Middleware para capturar requests y escribir audit logs de forma async.
//!
//! ADRs relacionados: 0003 (Axum 0.8), 0006 (RBAC)

use super::audit_channel::{AuditChannel, AuditLogBuilder};
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::{
    sync::Arc,
    time::Instant,
};

pub struct ActionLogger {
    channel: Arc<AuditChannel>,
}

impl ActionLogger {
    pub fn new(channel: Arc<AuditChannel>) -> Self {
        Self { channel }
    }

    pub async fn layer(self, request: Request, next: Next) -> Response {
        let method = request.method().to_string();
        let uri = request.uri().to_string();
        let start = Instant::now();

        let ip_address = request
            .headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or(s).trim().to_string());

        let user_agent = request
            .headers()
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let user_id = extract_user_id(&request);

        let response = next.run(request).await;

        let duration = start.elapsed();
        let status = response.status().as_u16().to_string();

        let details = format!(
            r#"{{"method":"{}","uri":"{}","status":"{}","duration_ms":{}}}"#,
            method, uri, status, duration.as_millis()
        );

        let resource = extract_resource(&uri);
        let action = format!("{} {}", method, resource);

        let mut builder = AuditLogBuilder::new(action, resource)
            .ip_address(ip_address.unwrap_or_else(|| "unknown".to_string()))
            .user_agent(user_agent.unwrap_or_default())
            .details(details);

        if let Some(uid) = user_id {
            builder = builder.user_id(uid);
        }

        if let Some(rid) = extract_resource_id(&uri) {
            builder = builder.resource_id(rid);
        }

        let entry = builder.build();

        if let Err(e) = self.channel.send(entry).await {
            tracing::warn!(error = ?e, "Failed to queue audit log");
        }

        response
    }
}

fn extract_user_id(request: &Request) -> Option<uuid::Uuid> {
    request
        .extensions()
        .get::<uuid::Uuid>()
        .copied()
}

fn extract_resource(uri: &str) -> String {
    uri.split('/')
        .skip(2)
        .next()
        .unwrap_or("unknown")
        .split('?')
        .next()
        .unwrap_or("unknown")
        .to_string()
}

fn extract_resource_id(uri: &str) -> Option<String> {
    let segments: Vec<&str> = uri.split('/').collect();
    segments.get(4).map(|s| s.to_string())
}