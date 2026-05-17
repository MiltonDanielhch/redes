//! Ubicación: `crates/email/src/templates.rs`
//!
//! Descripción: Templates de email para notificaciones.

#[derive(Debug, Clone)]
pub struct EmailTemplate {
    pub subject: String,
    pub html: String,
}

pub fn alert_notification(
    device_name: &str,
    alert_type: &str,
    severity: &str,
    message: &str,
) -> EmailTemplate {
    let severity_color = match severity.to_lowercase().as_str() {
        "critical" => "#dc2626",
        "high" => "#ea580c",
        "medium" => "#ca8a04",
        "low" => "#16a34a",
        _ => "#6b7280",
    };

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: {}; color: white; padding: 20px; text-align: center; }}
        .content {{ padding: 20px; background: #f9fafb; }}
        .alert-info {{ background: white; padding: 15px; margin: 10px 0; border-radius: 8px; }}
        .label {{ font-weight: bold; color: #666; }}
        .footer {{ text-align: center; padding: 20px; color: #666; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🔔 Alerta de Monitoreo</h1>
        </div>
        <div class="content">
            <div class="alert-info">
                <p><span class="label">Dispositivo:</span> {}</p>
                <p><span class="label">Tipo:</span> {}</p>
                <p><span class="label">Severidad:</span> {}</p>
                <p><span class="label">Mensaje:</span> {}</p>
            </div>
            <p>Esta alerta fue generada automáticamente por el sistema de monitoreo.</p>
        </div>
        <div class="footer">
            <p>Monitoreo de Infraestructura Regional - Red.es</p>
        </div>
    </div>
</body>
</html>"#,
        severity_color, device_name, alert_type, severity, message
    );

    EmailTemplate {
        subject: format!("[{}] Alerta: {} - {}", severity.to_uppercase(), device_name, alert_type),
        html,
    }
}

pub fn password_reset(token: &str, expiry_hours: i64) -> EmailTemplate {
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: #2563eb; color: white; padding: 20px; text-align: center; }}
        .content {{ padding: 20px; background: #f9fafb; }}
        .token {{ background: #e5e7eb; padding: 15px; font-family: monospace; font-size: 18px; text-align: center; margin: 20px 0; }}
        .warning {{ background: #fef3c7; padding: 15px; border-left: 4px solid #f59e0b; margin: 15px 0; }}
        .footer {{ text-align: center; padding: 20px; color: #666; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🔐 Restablecer Contraseña</h1>
        </div>
        <div class="content">
            <p>Has solicitado restablecer tu contraseña. Usa el siguiente token:</p>
            <div class="token">{}</div>
            <p>Este token expirará en {} horas.</p>
            <div class="warning">
                <p><strong>⚠️ Importante:</strong> Si no solicitaste este cambio, ignora este email.</p>
            </div>
        </div>
        <div class="footer">
            <p>Monitoreo de Infraestructura Regional - Red.es</p>
        </div>
    </div>
</body>
</html>"#,
        token, expiry_hours
    );

    EmailTemplate {
        subject: "Restablecer Contraseña - Monitoreo Red.es".to_string(),
        html,
    }
}