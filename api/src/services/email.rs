//! Email service with Lettre and Tera templates

use chrono::{DateTime, Datelike, Utc};
use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use tera::{Context, Tera};

use crate::config::EmailConfig;
use crate::errors::AppError;

/// Email service for sending transactional emails
pub struct EmailService {
    /// SMTP transport for sending emails
    transport: Option<AsyncSmtpTransport<Tokio1Executor>>,
    /// Template engine
    templates: Tera,
    /// Email configuration
    config: EmailConfig,
}

impl EmailService {
    /// Create a new email service
    pub fn new(config: EmailConfig) -> Result<Self, AppError> {
        let transport = if config.enabled {
            let creds = Credentials::new(
                config.smtp_username.clone(),
                config.smtp_password.clone(),
            );

            // Port 465 uses implicit TLS (SMTPS), port 587 uses STARTTLS
            let transport = if config.smtp_port == 465 {
                // Use SMTPS (implicit TLS) for port 465
                AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)
                    .map_err(|e| AppError::internal(format!("SMTP connection error: {}", e)))?
                    .port(config.smtp_port)
                    .credentials(creds)
                    .tls(lettre::transport::smtp::client::Tls::Wrapper(
                        lettre::transport::smtp::client::TlsParameters::new(config.smtp_host.clone())
                            .map_err(|e| AppError::internal(format!("TLS error: {}", e)))?,
                    ))
                    .build()
            } else {
                // Use STARTTLS for other ports (587, 25, etc.)
                AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)
                    .map_err(|e| AppError::internal(format!("SMTP connection error: {}", e)))?
                    .port(config.smtp_port)
                    .credentials(creds)
                    .build()
            };

            Some(transport)
        } else {
            None
        };

        // Initialize Tera templates with inline templates
        let mut templates = Tera::default();

        // Register base template
        templates.add_raw_template("base.html", include_str!("../../templates/emails/base.html"))
            .map_err(|e| AppError::internal(format!("Template error: {}", e)))?;
        templates.add_raw_template("base.txt", include_str!("../../templates/emails/base.txt"))
            .map_err(|e| AppError::internal(format!("Template error: {}", e)))?;

        // Register email templates
        templates.add_raw_template("magic_link.html", include_str!("../../templates/emails/magic_link.html"))
            .map_err(|e| AppError::internal(format!("Template error: {}", e)))?;
        templates.add_raw_template("magic_link.txt", include_str!("../../templates/emails/magic_link.txt"))
            .map_err(|e| AppError::internal(format!("Template error: {}", e)))?;

        templates.add_raw_template("password_reset.html", include_str!("../../templates/emails/password_reset.html"))
            .map_err(|e| AppError::internal(format!("Template error: {}", e)))?;
        templates.add_raw_template("password_reset.txt", include_str!("../../templates/emails/password_reset.txt"))
            .map_err(|e| AppError::internal(format!("Template error: {}", e)))?;

        templates.add_raw_template("welcome.html", include_str!("../../templates/emails/welcome.html"))
            .map_err(|e| AppError::internal(format!("Template error: {}", e)))?;
        templates.add_raw_template("welcome.txt", include_str!("../../templates/emails/welcome.txt"))
            .map_err(|e| AppError::internal(format!("Template error: {}", e)))?;

        templates.add_raw_template("account_created.html", include_str!("../../templates/emails/account_created.html"))
            .map_err(|e| AppError::internal(format!("Template error: {}", e)))?;
        templates.add_raw_template("account_created.txt", include_str!("../../templates/emails/account_created.txt"))
            .map_err(|e| AppError::internal(format!("Template error: {}", e)))?;

        templates.add_raw_template("payment_failed.html", include_str!("../../templates/emails/payment_failed.html"))
            .map_err(|e| AppError::internal(format!("Template error: {}", e)))?;
        templates.add_raw_template("payment_failed.txt", include_str!("../../templates/emails/payment_failed.txt"))
            .map_err(|e| AppError::internal(format!("Template error: {}", e)))?;

        templates.add_raw_template("grace_period_reminder.html", include_str!("../../templates/emails/grace_period_reminder.html"))
            .map_err(|e| AppError::internal(format!("Template error: {}", e)))?;
        templates.add_raw_template("grace_period_reminder.txt", include_str!("../../templates/emails/grace_period_reminder.txt"))
            .map_err(|e| AppError::internal(format!("Template error: {}", e)))?;

        templates.add_raw_template("membership_canceled.html", include_str!("../../templates/emails/membership_canceled.html"))
            .map_err(|e| AppError::internal(format!("Template error: {}", e)))?;
        templates.add_raw_template("membership_canceled.txt", include_str!("../../templates/emails/membership_canceled.txt"))
            .map_err(|e| AppError::internal(format!("Template error: {}", e)))?;

        templates.add_raw_template("payment_succeeded.html", include_str!("../../templates/emails/payment_succeeded.html"))
            .map_err(|e| AppError::internal(format!("Template error: {}", e)))?;
        templates.add_raw_template("payment_succeeded.txt", include_str!("../../templates/emails/payment_succeeded.txt"))
            .map_err(|e| AppError::internal(format!("Template error: {}", e)))?;

        Ok(Self {
            transport,
            templates,
            config,
        })
    }

    /// Create a new email service with default configuration (dev mode)
    pub fn new_dev() -> Self {
        let config = EmailConfig {
            smtp_host: "localhost".to_string(),
            smtp_port: 587,
            smtp_username: String::new(),
            smtp_password: String::new(),
            from_email: "noreply@a8n.tools".to_string(),
            from_name: "a8n.tools".to_string(),
            base_url: "https://app.a8n.tools".to_string(),
            enabled: false,
        };

        // Create with minimal template setup for dev
        let templates = Tera::default();

        Self {
            transport: None,
            templates,
            config,
        }
    }

    /// Send an email
    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        html_body: String,
        text_body: String,
    ) -> Result<(), AppError> {
        let from = format!("{} <{}>", self.config.from_name, self.config.from_email);

        if let Some(ref transport) = self.transport {
            let email = Message::builder()
                .from(from.parse().map_err(|e| AppError::internal(format!("Invalid from address: {}", e)))?)
                .to(to.parse().map_err(|e| AppError::internal(format!("Invalid to address: {}", e)))?)
                .subject(subject)
                .multipart(
                    lettre::message::MultiPart::alternative()
                        .singlepart(
                            lettre::message::SinglePart::builder()
                                .header(ContentType::TEXT_PLAIN)
                                .body(text_body),
                        )
                        .singlepart(
                            lettre::message::SinglePart::builder()
                                .header(ContentType::TEXT_HTML)
                                .body(html_body),
                        ),
                )
                .map_err(|e| AppError::internal(format!("Email build error: {}", e)))?;

            transport
                .send(email)
                .await
                .map_err(|e| AppError::internal(format!("Email send error: {}", e)))?;

            tracing::info!(to = %to, subject = %subject, "Email sent successfully");
        } else {
            tracing::info!(
                to = %to,
                subject = %subject,
                "Email not sent (dev mode)"
            );
        }

        Ok(())
    }

    /// Render a template
    fn render_template(&self, name: &str, context: &Context) -> Result<(String, String), AppError> {
        let html = self.templates
            .render(&format!("{}.html", name), context)
            .map_err(|e| AppError::internal(format!("Template render error: {}", e)))?;
        let text = self.templates
            .render(&format!("{}.txt", name), context)
            .map_err(|e| AppError::internal(format!("Template render error: {}", e)))?;
        Ok((html, text))
    }

    /// Get base context with common variables
    fn base_context(&self) -> Context {
        let mut context = Context::new();
        context.insert("base_url", &self.config.base_url);
        context.insert("year", &Utc::now().year());
        context
    }

    /// Send magic link email
    pub async fn send_magic_link(&self, email: &str, token: &str) -> Result<(), AppError> {
        let magic_link_url = format!(
            "{}/auth/magic-link?token={}",
            self.config.base_url, token
        );

        if !self.config.enabled {
            tracing::info!(
                email = %email,
                link = %magic_link_url,
                "Magic link (dev mode - not sending email)"
            );
            return Ok(());
        }

        let mut context = self.base_context();
        context.insert("magic_link_url", &magic_link_url);

        let (html, text) = self.render_template("magic_link", &context)?;
        self.send_email(email, "Sign in to a8n.tools", html, text).await
    }

    /// Send password reset email
    pub async fn send_password_reset(&self, email: &str, token: &str) -> Result<(), AppError> {
        let reset_url = format!(
            "{}/auth/reset-password?token={}",
            self.config.base_url, token
        );

        if !self.config.enabled {
            tracing::info!(
                email = %email,
                link = %reset_url,
                "Password reset link (dev mode - not sending email)"
            );
            return Ok(());
        }

        let mut context = self.base_context();
        context.insert("reset_url", &reset_url);

        let (html, text) = self.render_template("password_reset", &context)?;
        self.send_email(email, "Reset your a8n.tools password", html, text).await
    }

    /// Send account creation email
    pub async fn send_account_created(&self, email: &str) -> Result<(), AppError> {
        if !self.config.enabled {
            tracing::info!(email = %email, "Account created email (dev mode - not sending)");
            return Ok(());
        }

        let mut context = self.base_context();
        context.insert("dashboard_url", &format!("{}/dashboard", self.config.base_url));

        let (html, text) = self.render_template("account_created", &context)?;
        self.send_email(email, "Welcome to a8n.tools!", html, text).await
    }

    /// Send welcome email after membership activation
    pub async fn send_welcome(&self, email: &str, price_cents: i32) -> Result<(), AppError> {
        if !self.config.enabled {
            tracing::info!(email = %email, price = price_cents, "Welcome email (dev mode)");
            return Ok(());
        }

        let mut context = self.base_context();
        context.insert("dashboard_url", &format!("{}/dashboard", self.config.base_url));
        context.insert("price", &format!("{:.2}", price_cents as f64 / 100.0));

        let (html, text) = self.render_template("welcome", &context)?;
        self.send_email(email, "Welcome to a8n.tools!", html, text).await
    }

    /// Send payment failed email
    pub async fn send_payment_failed(&self, email: &str, days_remaining: i32) -> Result<(), AppError> {
        if !self.config.enabled {
            tracing::info!(
                email = %email,
                days = days_remaining,
                "Payment failed email (dev mode)"
            );
            return Ok(());
        }

        let mut context = self.base_context();
        context.insert("billing_url", &format!("{}/dashboard/membership", self.config.base_url));
        context.insert("days_remaining", &days_remaining);

        let (html, text) = self.render_template("payment_failed", &context)?;
        self.send_email(email, "Action required: Payment failed", html, text).await
    }

    /// Send grace period reminder email
    pub async fn send_grace_period_reminder(&self, email: &str, days_remaining: i32) -> Result<(), AppError> {
        if !self.config.enabled {
            tracing::info!(
                email = %email,
                days = days_remaining,
                "Grace period reminder email (dev mode)"
            );
            return Ok(());
        }

        let mut context = self.base_context();
        context.insert("billing_url", &format!("{}/dashboard/membership", self.config.base_url));
        context.insert("days_remaining", &days_remaining);

        let (html, text) = self.render_template("grace_period_reminder", &context)?;
        self.send_email(
            email,
            &format!("Only {} days left to update payment", days_remaining),
            html,
            text,
        ).await
    }

    /// Send membership canceled email
    pub async fn send_membership_canceled(&self, email: &str, end_date: DateTime<Utc>) -> Result<(), AppError> {
        if !self.config.enabled {
            tracing::info!(email = %email, end_date = %end_date, "Membership canceled email (dev mode)");
            return Ok(());
        }

        let mut context = self.base_context();
        context.insert("end_date", &end_date.format("%B %d, %Y").to_string());
        context.insert("resubscribe_url", &format!("{}/pricing", self.config.base_url));

        let (html, text) = self.render_template("membership_canceled", &context)?;
        self.send_email(email, "Your a8n.tools membership has been canceled", html, text).await
    }

    /// Send payment succeeded (receipt) email
    pub async fn send_payment_succeeded(&self, email: &str, amount_cents: i32) -> Result<(), AppError> {
        if !self.config.enabled {
            tracing::info!(email = %email, amount = amount_cents, "Payment succeeded email (dev mode)");
            return Ok(());
        }

        let mut context = self.base_context();
        context.insert("amount", &format!("{:.2}", amount_cents as f64 / 100.0));
        context.insert("dashboard_url", &format!("{}/dashboard", self.config.base_url));

        let (html, text) = self.render_template("payment_succeeded", &context)?;
        self.send_email(email, "Payment received - a8n.tools", html, text).await
    }
}

impl Default for EmailService {
    fn default() -> Self {
        Self::new_dev()
    }
}
