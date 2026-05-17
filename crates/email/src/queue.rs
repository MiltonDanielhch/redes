//! Ubicación: `crates/email/src/queue.rs`
//!
//! Descripción: Cola async para envío de emails sin bloquear.

use std::sync::Arc;
use tokio::sync::mpsc;
use crate::client::ResendClient;
use crate::error::EmailError;
use crate::templates::{alert_notification, password_reset};

#[derive(Debug, Clone)]
pub struct EmailJob {
    pub to: Vec<String>,
    pub template: EmailJobTemplate,
}

#[derive(Debug, Clone)]
pub enum EmailJobTemplate {
    AlertNotification {
        device_name: String,
        alert_type: String,
        severity: String,
        message: String,
    },
    PasswordReset {
        token: String,
        expiry_hours: i64,
    },
}

pub struct EmailQueue {
    _client: Arc<ResendClient>,
    sender: mpsc::Sender<EmailJob>,
}

impl EmailQueue {
    pub fn new(client: ResendClient) -> Self {
        let client = Arc::new(client);
        let (sender, mut receiver) = mpsc::channel::<EmailJob>(100);

        let client_clone = Arc::clone(&client);
        tokio::spawn(async move {
            while let Some(job) = receiver.recv().await {
                if let Err(e) = Self::process_job(&client_clone, job).await {
                    tracing::error!("Failed to send email: {}", e);
                }
            }
        });

        Self { _client: client, sender }
    }

    async fn process_job(client: &ResendClient, job: EmailJob) -> Result<(), EmailError> {
        let template = match job.template {
            EmailJobTemplate::AlertNotification {
                device_name,
                alert_type,
                severity,
                message,
            } => alert_notification(&device_name, &alert_type, &severity, &message),
            EmailJobTemplate::PasswordReset { token, expiry_hours } => {
                password_reset(&token, expiry_hours)
            }
        };

        client
            .send_email(&job.to, &template.subject, &template.html, None)
            .await?;

        Ok(())
    }

    pub async fn enqueue(&self, job: EmailJob) -> Result<(), EmailError> {
        self.sender
            .send(job)
            .await
            .map_err(|_| EmailError::QueueError("Failed to enqueue email".to_string()))?;
        Ok(())
    }

    pub async fn send_alert(
        &self,
        to: Vec<String>,
        device_name: String,
        alert_type: String,
        severity: String,
        message: String,
    ) -> Result<(), EmailError> {
        self.enqueue(EmailJob {
            to,
            template: EmailJobTemplate::AlertNotification {
                device_name,
                alert_type,
                severity,
                message,
            },
        })
        .await
    }

    pub async fn send_password_reset(
        &self,
        to: Vec<String>,
        token: String,
    ) -> Result<(), EmailError> {
        self.enqueue(EmailJob {
            to,
            template: EmailJobTemplate::PasswordReset {
                token,
                expiry_hours: 24,
            },
        })
        .await
    }
}