# 10 - Email System

## Overview

This document contains prompts for implementing the email system including Stalwart configuration, email templates, and sending logic.

## Prerequisites
- Completed backend API
- DNS configured for email (SPF, DKIM, DMARC)

---

## Prompt 10.1: Email Service Setup

```text
Set up the email sending service with lettre.

Add dependencies to Cargo.toml:
- lettre = { version = "0.11", features = ["tokio1-native-tls", "builder"] }
- tera = "1"  # For templates

Create src/services/email.rs:
```rust
use lettre::{
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use tera::{Context, Tera};

pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,
    pub from_name: String,
    pub base_url: String,
}

pub struct EmailService {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    templates: Tera,
    config: EmailConfig,
}

impl EmailService {
    pub fn new(config: EmailConfig) -> Result<Self, anyhow::Error> {
        let creds = Credentials::new(
            config.smtp_username.clone(),
            config.smtp_password.clone(),
        );

        let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)?
            .port(config.smtp_port)
            .credentials(creds)
            .build();

        let templates = Tera::new("templates/emails/**/*")?;

        Ok(Self { transport, templates, config })
    }

    pub async fn send_email(
        &self,
        to: &str,
        subject: &str,
        html_body: String,
        text_body: String,
    ) -> Result<(), AppError> {
        let email = Message::builder()
            .from(format!("{} <{}>", self.config.from_name, self.config.from_email).parse()?)
            .to(to.parse()?)
            .subject(subject)
            .multipart(
                lettre::message::MultiPart::alternative()
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(lettre::message::header::ContentType::TEXT_PLAIN)
                            .body(text_body),
                    )
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(lettre::message::header::ContentType::TEXT_HTML)
                            .body(html_body),
                    ),
            )?;

        self.transport.send(email).await?;
        Ok(())
    }

    fn render_template(&self, name: &str, context: &Context) -> Result<(String, String), AppError> {
        let html = self.templates.render(&format!("{}.html", name), context)?;
        let text = self.templates.render(&format!("{}.txt", name), context)?;
        Ok((html, text))
    }
}
```

Load config from environment:
- SMTP_HOST
- SMTP_PORT
- SMTP_USERNAME
- SMTP_PASSWORD
- EMAIL_FROM
- EMAIL_FROM_NAME
- BASE_URL
```

---

## Prompt 10.2: Email Template Base

```text
Create the base email template with a8n.tools branding.

Create templates/emails/base.html:
```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{{ subject }}</title>
  <style>
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      line-height: 1.6;
      color: #1e293b;
      background-color: #f8fafc;
      margin: 0;
      padding: 0;
    }
    .wrapper {
      max-width: 600px;
      margin: 0 auto;
      padding: 40px 20px;
    }
    .card {
      background: white;
      border-radius: 12px;
      box-shadow: 0 1px 3px rgba(0,0,0,0.1);
      overflow: hidden;
    }
    .header {
      background: linear-gradient(135deg, #f97316 0%, #b7410e 100%);
      padding: 24px;
      text-align: center;
    }
    .logo {
      color: white;
      font-size: 24px;
      font-weight: bold;
      text-decoration: none;
    }
    .content {
      padding: 32px;
    }
    .button {
      display: inline-block;
      background: #f97316;
      color: white !important;
      text-decoration: none;
      padding: 12px 24px;
      border-radius: 8px;
      font-weight: 600;
      margin: 16px 0;
    }
    .button:hover {
      background: #ea580c;
    }
    .footer {
      text-align: center;
      padding: 24px;
      color: #64748b;
      font-size: 14px;
    }
    .footer a {
      color: #f97316;
    }
    @media (prefers-color-scheme: dark) {
      body { background-color: #0f172a; }
      .card { background: #1e293b; }
      .content { color: #e2e8f0; }
    }
  </style>
</head>
<body>
  <div class="wrapper">
    <div class="card">
      <div class="header">
        <a href="{{ base_url }}" class="logo">a8n.tools</a>
      </div>
      <div class="content">
        {% block content %}{% endblock %}
      </div>
    </div>
    <div class="footer">
      <p>© {{ year }} a8n.tools. All rights reserved.</p>
      <p>
        <a href="{{ base_url }}/privacy">Privacy Policy</a> |
        <a href="{{ base_url }}/terms">Terms of Service</a>
      </p>
    </div>
  </div>
</body>
</html>
```

Create templates/emails/base.txt:
```text
a8n.tools
=========

{% block content %}{% endblock %}

---
© {{ year }} a8n.tools
Privacy: {{ base_url }}/privacy
Terms: {{ base_url }}/terms
```
```

---

## Prompt 10.3: Transactional Email Templates

```text
Create templates for all transactional emails.

1. templates/emails/magic_link.html:
```html
{% extends "base.html" %}
{% block content %}
<h1>Sign in to a8n.tools</h1>
<p>Click the button below to sign in to your account. This link expires in 15 minutes.</p>
<p style="text-align: center;">
  <a href="{{ magic_link_url }}" class="button">Sign In</a>
</p>
<p style="color: #64748b; font-size: 14px;">
  If you didn't request this link, you can safely ignore this email.
</p>
<p style="color: #64748b; font-size: 12px; word-break: break-all;">
  Or copy this link: {{ magic_link_url }}
</p>
{% endblock %}
```

2. templates/emails/password_reset.html:
```html
{% extends "base.html" %}
{% block content %}
<h1>Reset Your Password</h1>
<p>We received a request to reset your password. Click the button below to create a new password.</p>
<p style="text-align: center;">
  <a href="{{ reset_url }}" class="button">Reset Password</a>
</p>
<p style="color: #64748b; font-size: 14px;">
  This link expires in 1 hour. If you didn't request this, you can safely ignore this email.
</p>
{% endblock %}
```

3. templates/emails/welcome.html:
```html
{% extends "base.html" %}
{% block content %}
<h1>Welcome to a8n.tools! 🎉</h1>
<p>Thanks for subscribing! You now have access to all our developer tools.</p>
<h2>What's included:</h2>
<ul>
  <li><strong>RUS</strong> - Fast URL shortening with QR codes</li>
  <li><strong>Rusty Links</strong> - Bookmark management</li>
  <li>All future applications at no extra cost</li>
</ul>
<p>Your price of <strong>${{ price }}/month</strong> is locked forever.</p>
<p style="text-align: center;">
  <a href="{{ dashboard_url }}" class="button">Go to Dashboard</a>
</p>
{% endblock %}
```

4. templates/emails/payment_failed.html:
```html
{% extends "base.html" %}
{% block content %}
<h1>Payment Failed</h1>
<p>We couldn't process your payment for a8n.tools subscription.</p>
<p>Don't worry - you still have access for the next 30 days while you update your payment method.</p>
<p style="text-align: center;">
  <a href="{{ billing_url }}" class="button">Update Payment Method</a>
</p>
<p style="color: #64748b; font-size: 14px;">
  If you have questions, reply to this email and we'll help you out.
</p>
{% endblock %}
```

5. templates/emails/grace_period_reminder.html (for days 7, 14, 25)

6. templates/emails/subscription_canceled.html

7. templates/emails/payment_succeeded.html

Create corresponding .txt versions for each template.
```

---

## Prompt 10.4: Email Sending Methods

```text
Add specific email sending methods to EmailService.

Extend src/services/email.rs:
```rust
impl EmailService {
    pub async fn send_magic_link(
        &self,
        to: &str,
        token: &str,
    ) -> Result<(), AppError> {
        let mut context = Context::new();
        context.insert("base_url", &self.config.base_url);
        context.insert("magic_link_url", &format!(
            "{}/magic-link/verify?token={}",
            self.config.base_url,
            token
        ));
        context.insert("year", &Utc::now().year());

        let (html, text) = self.render_template("magic_link", &context)?;
        self.send_email(to, "Sign in to a8n.tools", html, text).await
    }

    pub async fn send_password_reset(
        &self,
        to: &str,
        token: &str,
    ) -> Result<(), AppError> {
        let mut context = Context::new();
        context.insert("base_url", &self.config.base_url);
        context.insert("reset_url", &format!(
            "{}/password-reset?token={}",
            self.config.base_url,
            token
        ));
        context.insert("year", &Utc::now().year());

        let (html, text) = self.render_template("password_reset", &context)?;
        self.send_email(to, "Reset your a8n.tools password", html, text).await
    }

    pub async fn send_welcome(
        &self,
        to: &str,
        price_cents: i32,
    ) -> Result<(), AppError> {
        let mut context = Context::new();
        context.insert("base_url", &self.config.base_url);
        context.insert("dashboard_url", &format!("{}/dashboard", self.config.base_url));
        context.insert("price", &format!("{:.2}", price_cents as f64 / 100.0));
        context.insert("year", &Utc::now().year());

        let (html, text) = self.render_template("welcome", &context)?;
        self.send_email(to, "Welcome to a8n.tools!", html, text).await
    }

    pub async fn send_payment_failed(
        &self,
        to: &str,
    ) -> Result<(), AppError> {
        let mut context = Context::new();
        context.insert("base_url", &self.config.base_url);
        context.insert("billing_url", &format!("{}/dashboard/subscription", self.config.base_url));
        context.insert("year", &Utc::now().year());

        let (html, text) = self.render_template("payment_failed", &context)?;
        self.send_email(to, "Action required: Payment failed", html, text).await
    }

    pub async fn send_grace_period_reminder(
        &self,
        to: &str,
        days_remaining: i32,
    ) -> Result<(), AppError> {
        let mut context = Context::new();
        context.insert("base_url", &self.config.base_url);
        context.insert("billing_url", &format!("{}/dashboard/subscription", self.config.base_url));
        context.insert("days_remaining", &days_remaining);
        context.insert("year", &Utc::now().year());

        let (html, text) = self.render_template("grace_period_reminder", &context)?;
        self.send_email(
            to,
            &format!("Only {} days left to update payment", days_remaining),
            html,
            text,
        ).await
    }

    pub async fn send_subscription_canceled(
        &self,
        to: &str,
        end_date: DateTime<Utc>,
    ) -> Result<(), AppError>;

    pub async fn send_payment_succeeded(
        &self,
        to: &str,
        amount_cents: i32,
    ) -> Result<(), AppError>;
}
```
```

---

## Prompt 10.5: Email Queue (Optional)

```text
Implement an async email queue for reliability.

Create src/services/email_queue.rs:
```rust
use tokio::sync::mpsc;

pub enum EmailJob {
    MagicLink { to: String, token: String },
    PasswordReset { to: String, token: String },
    Welcome { to: String, price_cents: i32 },
    PaymentFailed { to: String },
    GracePeriodReminder { to: String, days_remaining: i32 },
    SubscriptionCanceled { to: String, end_date: DateTime<Utc> },
    PaymentSucceeded { to: String, amount_cents: i32 },
}

pub struct EmailQueue {
    sender: mpsc::Sender<EmailJob>,
}

impl EmailQueue {
    pub fn new(email_service: Arc<EmailService>) -> Self {
        let (sender, mut receiver) = mpsc::channel::<EmailJob>(100);

        tokio::spawn(async move {
            while let Some(job) = receiver.recv().await {
                let result = match job {
                    EmailJob::MagicLink { to, token } => {
                        email_service.send_magic_link(&to, &token).await
                    }
                    EmailJob::PasswordReset { to, token } => {
                        email_service.send_password_reset(&to, &token).await
                    }
                    // ... handle other job types
                };

                if let Err(e) = result {
                    tracing::error!(error = ?e, "failed to send email");
                    // Could implement retry logic here
                }
            }
        });

        Self { sender }
    }

    pub async fn enqueue(&self, job: EmailJob) -> Result<(), AppError> {
        self.sender.send(job).await
            .map_err(|_| AppError::InternalError {
                message: "email queue full".to_string()
            })
    }
}
```

Use the queue in auth service:
```rust
// Instead of:
email_service.send_magic_link(email, token).await?;

// Use:
email_queue.enqueue(EmailJob::MagicLink {
    to: email.to_string(),
    token: token.to_string(),
}).await?;
```

This allows the API to return immediately while emails send in background.
```

---

## Prompt 10.6: Stalwart Configuration

```text
Configure Stalwart mail server in Docker.

Add to docker-compose.yml:
```yaml
stalwart:
  image: stalwartlabs/mail-server:latest
  container_name: a8n-stalwart
  restart: unless-stopped
  ports:
    - "25:25"      # SMTP
    - "587:587"    # Submission
    - "465:465"    # SMTPS
    - "143:143"    # IMAP
    - "993:993"    # IMAPS
  volumes:
    - stalwart-data:/opt/stalwart-mail
    - ./stalwart/config.toml:/opt/stalwart-mail/etc/config.toml
  environment:
    - STALWART_ADMIN_PASSWORD=${STALWART_ADMIN_PASSWORD}
  networks:
    - a8n-network
  labels:
    - "traefik.enable=true"
    - "traefik.http.routers.stalwart.rule=Host(`mail.a8n.tools`)"
```

Create stalwart/config.toml:
```toml
[server]
hostname = "mail.a8n.tools"

[server.listener."smtp"]
bind = ["0.0.0.0:25"]
protocol = "smtp"

[server.listener."submission"]
bind = ["0.0.0.0:587"]
protocol = "smtp"
tls.implicit = false

[server.listener."smtps"]
bind = ["0.0.0.0:465"]
protocol = "smtp"
tls.implicit = true

[authentication]
mechanisms = ["PLAIN", "LOGIN"]

[storage]
data = "rocksdb"
blob = "rocksdb"
lookup = "rocksdb"
directory = "internal"

[directory."internal"]
type = "internal"
store = "rocksdb"
```

DNS records required:
- MX record: a8n.tools -> mail.a8n.tools
- SPF: v=spf1 ip4:YOUR_IP include:_spf.a8n.tools ~all
- DKIM: Generate and add TXT record
- DMARC: v=DMARC1; p=quarantine; rua=mailto:dmarc@a8n.tools
```

---

## Validation Checklist

After completing all prompts in this section, verify:

- [ ] SMTP connection works
- [ ] Magic link emails send and are deliverable
- [ ] Password reset emails work
- [ ] Welcome email sends on subscription
- [ ] Payment failed email sends
- [ ] Grace period reminders send on schedule
- [ ] Emails render correctly in major clients
- [ ] Plain text fallback works
- [ ] Email queue processes reliably
- [ ] SPF/DKIM/DMARC pass (use mail-tester.com)

---

## Next Steps

Proceed to **[11-infrastructure.md](./11-infrastructure.md)** to set up production infrastructure.
