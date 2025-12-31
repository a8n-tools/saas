//! Email service (placeholder for now)

use crate::errors::AppError;

/// Email service for sending transactional emails
pub struct EmailService {
    // Configuration will be added when implementing actual email sending
    enabled: bool,
}

impl EmailService {
    pub fn new() -> Self {
        Self { enabled: false }
    }

    /// Send magic link email
    pub async fn send_magic_link(&self, email: &str, token: &str) -> Result<(), AppError> {
        let link = format!("https://app.a8n.tools/auth/magic-link?token={}", token);

        if self.enabled {
            // TODO: Implement actual email sending
            tracing::info!(email = %email, "Would send magic link email");
        } else {
            // Development: log the link
            tracing::info!(
                email = %email,
                link = %link,
                "Magic link (dev mode - not sending email)"
            );
        }

        Ok(())
    }

    /// Send password reset email
    pub async fn send_password_reset(&self, email: &str, token: &str) -> Result<(), AppError> {
        let link = format!("https://app.a8n.tools/auth/reset-password?token={}", token);

        if self.enabled {
            // TODO: Implement actual email sending
            tracing::info!(email = %email, "Would send password reset email");
        } else {
            tracing::info!(
                email = %email,
                link = %link,
                "Password reset link (dev mode - not sending email)"
            );
        }

        Ok(())
    }

    /// Send welcome email after subscription
    pub async fn send_welcome(&self, email: &str) -> Result<(), AppError> {
        if self.enabled {
            tracing::info!(email = %email, "Would send welcome email");
        } else {
            tracing::info!(email = %email, "Welcome email (dev mode)");
        }

        Ok(())
    }

    /// Send payment failed email
    pub async fn send_payment_failed(&self, email: &str, days_remaining: i32) -> Result<(), AppError> {
        if self.enabled {
            tracing::info!(email = %email, days = days_remaining, "Would send payment failed email");
        } else {
            tracing::info!(
                email = %email,
                days = days_remaining,
                "Payment failed email (dev mode)"
            );
        }

        Ok(())
    }

    /// Send subscription canceled email
    pub async fn send_subscription_canceled(&self, email: &str) -> Result<(), AppError> {
        if self.enabled {
            tracing::info!(email = %email, "Would send subscription canceled email");
        } else {
            tracing::info!(email = %email, "Subscription canceled email (dev mode)");
        }

        Ok(())
    }
}

impl Default for EmailService {
    fn default() -> Self {
        Self::new()
    }
}
