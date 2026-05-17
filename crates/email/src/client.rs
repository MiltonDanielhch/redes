//! Ubicación: `crates/email/src/client.rs`
//!
//! Descripción: Cliente para API de Resend.

use serde::{Deserialize, Serialize};
use crate::error::EmailError;

#[derive(Debug, Clone)]
pub struct ResendClient {
    api_key: String,
    from_email: String,
    client: reqwest::Client,
}

#[derive(Debug, Serialize)]
struct SendEmailRequest {
    from: String,
    to: Vec<String>,
    subject: String,
    html: String,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ResendResponse {
    id: String,
    #[serde(default)]
    object: Option<String>,
}

impl ResendClient {
    pub fn new(api_key: impl Into<String>, from_email: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            from_email: from_email.into(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn send_email(
        &self,
        to: &[String],
        subject: &str,
        html: &str,
        text: Option<&str>,
    ) -> Result<String, EmailError> {
        let request = SendEmailRequest {
            from: self.from_email.clone(),
            to: to.to_vec(),
            subject: subject.to_string(),
            html: html.to_string(),
            text: text.map(String::from),
        };

        let response = self
            .client
            .post("https://api.resend.com/emails")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| EmailError::SendFailed(e.to_string()))?;

        if response.status().is_success() {
            let resend_resp: ResendResponse = response
                .json()
                .await
                .map_err(|e| EmailError::SendFailed(e.to_string()))?;
            Ok(resend_resp.id)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(EmailError::SendFailed(format!(
                "Status {}: {}",
                status, body
            )))
        }
    }

    pub async fn send_template(
        &self,
        to: &[String],
        subject: &str,
        template: &str,
        data: &serde_json::Value,
    ) -> Result<String, EmailError> {
        let html = Self::render_template(template, data)?;
        let text = Self::render_template(template, data)
            .ok()
            .map(|t| Self::strip_html(&t));
        self.send_email(to, subject, &html, text.as_deref()).await
    }

    fn render_template(template: &str, data: &serde_json::Value) -> Result<String, EmailError> {
        let mut html = template.to_string();
        if let Some(obj) = data.as_object() {
            for (key, value) in obj {
                let placeholder = format!("{{{{{}}}}}", key);
                let replacement = value
                    .as_str()
                    .unwrap_or(&value.to_string());
                html = html.replace(&placeholder, replacement);
            }
        }
        Ok(html)
    }

    fn strip_html(html: &str) -> String {
        html.replace("<", "")
            .replace(">", "")
            .replace("&nbsp;", " ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_template() {
        let template = "<h1>Hello {{name}}</h1>";
        let data = serde_json::json!({"name": "Admin"});
        let result = ResendClient::render_template(template, &data).unwrap();
        assert_eq!(result, "<h1>Hello Admin</h1>");
    }

    #[test]
    fn test_strip_html() {
        let html = "<h1>Hello</h1>";
        let result = ResendClient::strip_html(html);
        assert_eq!(result, "Hello");
    }
}