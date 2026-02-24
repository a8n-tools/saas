# 13 - Security Hardening

## Overview

This document contains prompts for implementing security measures including rate limiting, input validation, security headers, and secure configuration.

## Prerequisites
- All application features complete
- Infrastructure setup done

---

## Prompt 13.1: Rate Limiting Implementation

```text
Implement comprehensive rate limiting.

Create src/middleware/rate_limit.rs:
```rust
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct RateLimitConfig {
    pub action: String,
    pub max_requests: i32,
    pub window_seconds: i64,
}

pub struct RateLimiter {
    store: Arc<RwLock<HashMap<String, (i32, i64)>>>,  // (count, window_start)
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub async fn check(&self, key: &str) -> Result<(), AppError> {
        let mut store = self.store.write().await;
        let now = Utc::now().timestamp();

        let (count, window_start) = store
            .entry(key.to_string())
            .or_insert((0, now));

        // Reset if window expired
        if now - *window_start > self.config.window_seconds {
            *count = 0;
            *window_start = now;
        }

        *count += 1;

        if *count > self.config.max_requests {
            let retry_after = self.config.window_seconds - (now - *window_start);
            return Err(AppError::RateLimited { retry_after: retry_after as u64 });
        }

        Ok(())
    }
}

// Rate limit middleware factory
pub struct RateLimitMiddleware {
    limiter: Arc<RateLimiter>,
    key_extractor: fn(&ServiceRequest) -> String,
}

impl RateLimitMiddleware {
    pub fn new(
        config: RateLimitConfig,
        key_extractor: fn(&ServiceRequest) -> String,
    ) -> Self {
        Self {
            limiter: Arc::new(RateLimiter::new(config)),
            key_extractor,
        }
    }
}
```

Apply rate limits to routes:
```rust
// Login: 5 per minute per IP
.route("/auth/login",
    web::post()
        .wrap(RateLimitMiddleware::new(
            RateLimitConfig {
                action: "login".to_string(),
                max_requests: 5,
                window_seconds: 60,
            },
            |req| extract_ip(req),
        ))
        .to(login)
)

// Magic link: 3 per 10 minutes per email
.route("/auth/magic-link",
    web::post()
        .wrap(RateLimitMiddleware::new(
            RateLimitConfig {
                action: "magic_link".to_string(),
                max_requests: 3,
                window_seconds: 600,
            },
            |req| extract_email_from_body(req),
        ))
        .to(request_magic_link)
)
```

Use Redis for distributed rate limiting in production:
```rust
pub struct RedisRateLimiter {
    redis: redis::Client,
    config: RateLimitConfig,
}

impl RedisRateLimiter {
    pub async fn check(&self, key: &str) -> Result<(), AppError> {
        let mut conn = self.redis.get_async_connection().await?;
        let redis_key = format!("ratelimit:{}:{}", self.config.action, key);

        let count: i32 = redis::cmd("INCR")
            .arg(&redis_key)
            .query_async(&mut conn)
            .await?;

        if count == 1 {
            redis::cmd("EXPIRE")
                .arg(&redis_key)
                .arg(self.config.window_seconds)
                .query_async(&mut conn)
                .await?;
        }

        if count > self.config.max_requests {
            let ttl: i64 = redis::cmd("TTL")
                .arg(&redis_key)
                .query_async(&mut conn)
                .await?;
            return Err(AppError::RateLimited { retry_after: ttl as u64 });
        }

        Ok(())
    }
}
```
```

---

## Prompt 13.2: Input Validation and Sanitization

```text
Implement comprehensive input validation.

Create src/validation/sanitize.rs:
```rust
use ammonia::Builder;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(
        r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
    ).unwrap();

    static ref SLUG_REGEX: Regex = Regex::new(
        r"^[a-z0-9-]+$"
    ).unwrap();

    static ref SQL_INJECTION_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"(?i)(union|select|insert|update|delete|drop|create|alter|exec|execute)").unwrap(),
        Regex::new(r"['\";]").unwrap(),
        Regex::new(r"--").unwrap(),
    ];
}

pub fn sanitize_html(input: &str) -> String {
    Builder::default()
        .clean(input)
        .to_string()
}

pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    if email.len() > 255 {
        return Err(ValidationError::new("email", "Email too long"));
    }
    if !EMAIL_REGEX.is_match(email) {
        return Err(ValidationError::new("email", "Invalid email format"));
    }
    Ok(())
}

pub fn validate_password(password: &str) -> Result<(), ValidationErrors> {
    let mut errors = Vec::new();

    if password.len() < 12 {
        errors.push("Password must be at least 12 characters");
    }
    if password.len() > 128 {
        errors.push("Password too long");
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        errors.push("Password must contain uppercase letter");
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        errors.push("Password must contain lowercase letter");
    }
    if !password.chars().any(|c| c.is_numeric()) {
        errors.push("Password must contain a number");
    }
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        errors.push("Password must contain a special character");
    }

    // Check against common passwords
    if COMMON_PASSWORDS.contains(&password.to_lowercase().as_str()) {
        errors.push("Password is too common");
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(ValidationErrors { errors })
    }
}

pub fn check_sql_injection(input: &str) -> bool {
    SQL_INJECTION_PATTERNS.iter().any(|pattern| pattern.is_match(input))
}

pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect()
}
```

Apply validation in handlers:
```rust
#[derive(Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(custom = "validate_email")]
    pub email: String,
    #[validate(custom = "validate_password")]
    pub password: String,
}

pub async fn register(
    req: web::Json<RegisterRequest>,
) -> Result<HttpResponse, AppError> {
    req.validate()?;
    // Proceed with registration
}
```
```

---

## Prompt 13.3: Security Headers

```text
Configure comprehensive security headers.

Create src/middleware/security_headers.rs:
```rust
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};

pub struct SecurityHeaders;

impl<S, B> Transform<S, ServiceRequest> for SecurityHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    // Implementation adds headers to all responses
}

fn add_security_headers<B>(response: &mut ServiceResponse<B>) {
    let headers = response.headers_mut();

    // Prevent clickjacking
    headers.insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    );

    // Prevent MIME type sniffing
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );

    // XSS protection (legacy but still useful)
    headers.insert(
        header::X_XSS_PROTECTION,
        HeaderValue::from_static("1; mode=block"),
    );

    // Referrer policy
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // HSTS
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
    );

    // Content Security Policy
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(concat!(
            "default-src 'self'; ",
            "script-src 'self' 'unsafe-inline' https://js.stripe.com; ",
            "style-src 'self' 'unsafe-inline'; ",
            "img-src 'self' data: https:; ",
            "font-src 'self'; ",
            "frame-src https://js.stripe.com https://hooks.stripe.com; ",
            "connect-src 'self' https://api.stripe.com; ",
            "object-src 'none'; ",
            "base-uri 'self'; ",
            "form-action 'self';"
        )),
    );

    // Permissions Policy
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static(
            "accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(self), usb=()"
        ),
    );
}
```

Add to Actix app:
```rust
App::new()
    .wrap(SecurityHeaders)
```
```

---

## Prompt 13.4: CSRF Protection

```text
Implement CSRF protection for state-changing requests.

Create src/middleware/csrf.rs:
```rust
use actix_web::cookie::Cookie;
use rand::Rng;

const CSRF_COOKIE_NAME: &str = "csrf_token";
const CSRF_HEADER_NAME: &str = "X-CSRF-Token";

pub fn generate_csrf_token() -> String {
    let token: [u8; 32] = rand::thread_rng().gen();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(token)
}

pub fn create_csrf_cookie(token: &str) -> Cookie {
    Cookie::build(CSRF_COOKIE_NAME, token.to_string())
        .domain(".example.com")
        .path("/")
        .http_only(false)  // JavaScript needs to read this
        .secure(true)
        .same_site(actix_web::cookie::SameSite::Strict)
        .finish()
}

pub struct CsrfMiddleware;

impl<S, B> Transform<S, ServiceRequest> for CsrfMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    // For state-changing methods (POST, PUT, DELETE):
    // 1. Get token from cookie
    // 2. Get token from header
    // 3. Compare tokens
    // 4. Reject if mismatch
}

// In handlers that need CSRF:
pub async fn create_checkout(
    req: HttpRequest,
    csrf: CsrfValidator,  // Extractor that validates CSRF
) -> Result<HttpResponse, AppError> {
    csrf.validate(&req)?;
    // Proceed
}
```

Frontend integration:
```typescript
// Read CSRF token from cookie
function getCsrfToken(): string {
  const match = document.cookie.match(/csrf_token=([^;]+)/);
  return match ? match[1] : '';
}

// Add to API client
apiClient.interceptors.request.use((config) => {
  if (['post', 'put', 'delete', 'patch'].includes(config.method?.toLowerCase() || '')) {
    config.headers['X-CSRF-Token'] = getCsrfToken();
  }
  return config;
});
```
```

---

## Prompt 13.5: Secrets Management

```text
Implement secure secrets management.

Create src/config/secrets.rs:
```rust
use secrecy::{ExposeSecret, Secret};

#[derive(Clone)]
pub struct Secrets {
    pub database_url: Secret<String>,
    pub jwt_private_key: Secret<String>,
    pub stripe_secret_key: Secret<String>,
    pub stripe_webhook_secret: Secret<String>,
}

impl Secrets {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        Ok(Self {
            database_url: Secret::new(
                std::env::var("DATABASE_URL")
                    .context("DATABASE_URL must be set")?
            ),
            jwt_private_key: Secret::new(
                std::fs::read_to_string(
                    std::env::var("JWT_PRIVATE_KEY_PATH")
                        .context("JWT_PRIVATE_KEY_PATH must be set")?
                )?
            ),
            stripe_secret_key: Secret::new(
                std::env::var("STRIPE_SECRET_KEY")
                    .context("STRIPE_SECRET_KEY must be set")?
            ),
            stripe_webhook_secret: Secret::new(
                std::env::var("STRIPE_WEBHOOK_SECRET")
                    .context("STRIPE_WEBHOOK_SECRET must be set")?
            ),
        })
    }
}

// Usage - secrets are never accidentally logged
impl JwtService {
    pub fn new(secrets: &Secrets) -> Self {
        let key = secrets.jwt_private_key.expose_secret();
        // Use the key
    }
}
```

Key file permissions:
```bash
# JWT private key - only readable by owner
chmod 600 keys/jwt_private.pem

# JWT public key - readable by all
chmod 644 keys/jwt_public.pem

# .env file
chmod 600 .env
```

Never log secrets:
```rust
// WRONG
tracing::info!("Connecting to {}", database_url);

// RIGHT
tracing::info!("Connecting to database...");
```
```

---

## Prompt 13.6: Audit Logging for Security Events

```text
Ensure all security events are logged.

Create src/services/audit.rs:
```rust
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum SecurityEvent {
    LoginSuccess { user_id: Uuid },
    LoginFailed { email: String, reason: String },
    LogoutSuccess { user_id: Uuid },
    PasswordChanged { user_id: Uuid },
    PasswordResetRequested { email: String },
    PasswordResetCompleted { user_id: Uuid },
    MagicLinkRequested { email: String },
    MagicLinkUsed { user_id: Uuid },
    TokenRefreshed { user_id: Uuid },
    RateLimitExceeded { key: String, action: String },
    SuspiciousActivity { description: String },
    AdminAction { admin_id: Uuid, action: String, target: String },
}

pub struct AuditService {
    pool: PgPool,
}

impl AuditService {
    pub async fn log_security_event(
        &self,
        event: SecurityEvent,
        ip_address: Option<IpAddr>,
        user_agent: Option<&str>,
    ) -> Result<(), AppError> {
        let (action, actor_id, severity, metadata) = match &event {
            SecurityEvent::LoginSuccess { user_id } => (
                "login_success",
                Some(*user_id),
                "info",
                serde_json::json!({}),
            ),
            SecurityEvent::LoginFailed { email, reason } => (
                "login_failed",
                None,
                "warning",
                serde_json::json!({ "email": email, "reason": reason }),
            ),
            SecurityEvent::RateLimitExceeded { key, action } => (
                "rate_limit_exceeded",
                None,
                "warning",
                serde_json::json!({ "key": key, "action": action }),
            ),
            SecurityEvent::SuspiciousActivity { description } => (
                "suspicious_activity",
                None,
                "critical",
                serde_json::json!({ "description": description }),
            ),
            // ... handle other events
        };

        sqlx::query!(
            r#"
            INSERT INTO audit_logs (
                actor_id, action, actor_ip_address,
                metadata, severity, created_at
            ) VALUES ($1, $2, $3, $4, $5, NOW())
            "#,
            actor_id,
            action,
            ip_address.map(|ip| ip.to_string()),
            metadata,
            severity,
        )
        .execute(&self.pool)
        .await?;

        // Also log to tracing for real-time monitoring
        match severity {
            "critical" => tracing::error!(event = ?event, "security event"),
            "warning" => tracing::warn!(event = ?event, "security event"),
            _ => tracing::info!(event = ?event, "security event"),
        }

        Ok(())
    }
}
```
```

---

## Validation Checklist

After completing all prompts in this section, verify:

- [ ] Rate limiting blocks excessive requests
- [ ] Rate limit headers returned (X-RateLimit-*)
- [ ] Input validation rejects malicious input
- [ ] SQL injection attempts blocked
- [ ] XSS attempts sanitized
- [ ] Security headers present on all responses
- [ ] CSRF protection works
- [ ] Secrets not logged anywhere
- [ ] Audit logs capture all security events
- [ ] HTTPS enforced everywhere

---

## Security Testing Commands

```bash
# Test security headers
curl -I https://api.example.com/health

# Test rate limiting
for i in {1..10}; do curl -X POST https://api.example.com/v1/auth/login; done

# Test CSRF
curl -X POST https://api.example.com/v1/subscriptions/checkout \
  -H "Cookie: access_token=..." \
  # Should fail without CSRF token
```

---

## Next Steps

Proceed to **[14-testing-strategy.md](./14-testing-strategy.md)** to implement the testing strategy.
