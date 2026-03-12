//! Webhook service for outbound notifications to child apps

use hmac::{Hmac, Mac};
use sha2::Sha256;
use tracing::{error, info};

use crate::models::Application;

type HmacSha256 = Hmac<Sha256>;

pub struct WebhookService {
    client: reqwest::Client,
    signing_secret: String,
}

impl WebhookService {
    pub fn new(signing_secret: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to build webhook HTTP client");

        Self {
            client,
            signing_secret,
        }
    }

    /// Notify a child app that its maintenance mode has changed.
    pub async fn notify_maintenance_change(&self, app: &Application) {
        let payload = serde_json::json!({
            "event": "maintenance_mode_changed",
            "slug": app.slug,
            "maintenance_mode": app.maintenance_mode,
            "maintenance_message": app.maintenance_message,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        self.send(app, payload).await;
    }

    /// Notify a child app that its active status has changed.
    pub async fn notify_active_change(&self, app: &Application) {
        let payload = serde_json::json!({
            "event": "active_changed",
            "slug": app.slug,
            "is_active": app.is_active,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        self.send(app, payload).await;
    }

    /// Send a signed JSON payload to the app's webhook URL.
    /// Fires and forgets — logs success/failure but never errors out.
    async fn send(&self, app: &Application, payload: serde_json::Value) {
        let webhook_url = match &app.webhook_url {
            Some(url) if !url.is_empty() => url,
            _ => return,
        };

        let body = serde_json::to_string(&payload).unwrap_or_default();
        let signature = self.sign(&body);

        match self
            .client
            .post(webhook_url)
            .header("Content-Type", "application/json")
            .header("X-Webhook-Signature", &signature)
            .body(body)
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    info!(
                        app_slug = %app.slug,
                        webhook_url = %webhook_url,
                        status = %status,
                        "Webhook notification delivered"
                    );
                } else {
                    error!(
                        app_slug = %app.slug,
                        webhook_url = %webhook_url,
                        status = %status,
                        "Webhook notification failed with non-success status"
                    );
                }
            }
            Err(e) => {
                error!(
                    app_slug = %app.slug,
                    webhook_url = %webhook_url,
                    error = %e,
                    "Webhook notification delivery failed"
                );
            }
        }
    }

    fn sign(&self, payload: &str) -> String {
        let mut mac =
            HmacSha256::new_from_slice(self.signing_secret.as_bytes()).expect("HMAC accepts any key size");
        mac.update(payload.as_bytes());
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }
}
