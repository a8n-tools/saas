# 02 - Database Schema & Migrations

## Overview

This document contains prompts for creating the database schema, migrations, and Rust models using SQLx.

## Prerequisites
- Completed 01-project-setup.md
- PostgreSQL running in Docker
- SQLx CLI installed (`cargo install sqlx-cli`)

---

## Prompt 2.1: Create Users Table Migration

```text
Create the first SQLx migration for the users table.

Run: sqlx migrate add create_users

In the generated migration file, create the users table:

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    password_hash VARCHAR(255),
    role VARCHAR(50) NOT NULL DEFAULT 'subscriber',
    stripe_customer_id VARCHAR(255) UNIQUE,
    subscription_status VARCHAR(50) NOT NULL DEFAULT 'none',
    price_locked BOOLEAN NOT NULL DEFAULT FALSE,
    locked_price_id VARCHAR(255),
    locked_price_amount INTEGER,
    grace_period_start TIMESTAMPTZ,
    grace_period_end TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_stripe_customer_id ON users(stripe_customer_id);
CREATE INDEX idx_users_subscription_status ON users(subscription_status);
```

In src/models/user.rs, create:

1. User struct matching the table:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
   pub struct User {
       pub id: Uuid,
       pub email: String,
       pub email_verified: bool,
       #[serde(skip_serializing)]
       pub password_hash: Option<String>,
       pub role: String,
       pub stripe_customer_id: Option<String>,
       pub subscription_status: String,
       pub price_locked: bool,
       pub locked_price_id: Option<String>,
       pub locked_price_amount: Option<i32>,
       pub grace_period_start: Option<DateTime<Utc>>,
       pub grace_period_end: Option<DateTime<Utc>>,
       pub created_at: DateTime<Utc>,
       pub updated_at: DateTime<Utc>,
       pub last_login_at: Option<DateTime<Utc>>,
       pub deleted_at: Option<DateTime<Utc>>,
   }
   ```

2. UserRole enum:
   ```rust
   #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
   #[serde(rename_all = "snake_case")]
   pub enum UserRole {
       Subscriber,
       Admin,
   }
   ```

3. SubscriptionStatus enum:
   ```rust
   #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
   #[serde(rename_all = "snake_case")]
   pub enum SubscriptionStatus {
       None,
       Active,
       PastDue,
       Canceled,
       GracePeriod,
   }
   ```

4. CreateUser struct for insertions
5. UserResponse struct (public fields only, no password_hash)

Create src/models/mod.rs to export all models.

Run the migration and verify with: sqlx migrate run
```

---

## Prompt 2.2: Create Token Tables Migrations

```text
Create migrations for authentication tokens.

Migration 1: Refresh Tokens
Run: sqlx migrate add create_refresh_tokens

```sql
CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    device_info VARCHAR(500),
    ip_address INET,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ
);

CREATE INDEX idx_refresh_tokens_user_id ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_token_hash ON refresh_tokens(token_hash);
CREATE INDEX idx_refresh_tokens_expires_at ON refresh_tokens(expires_at);
```

Migration 2: Magic Link Tokens
Run: sqlx migrate add create_magic_link_tokens

```sql
CREATE TABLE magic_link_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address INET
);

CREATE INDEX idx_magic_link_tokens_email ON magic_link_tokens(email);
CREATE INDEX idx_magic_link_tokens_token_hash ON magic_link_tokens(token_hash);
```

Migration 3: Password Reset Tokens
Run: sqlx migrate add create_password_reset_tokens

```sql
CREATE TABLE password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ip_address INET
);

CREATE INDEX idx_password_reset_tokens_user_id ON password_reset_tokens(user_id);
CREATE INDEX idx_password_reset_tokens_token_hash ON password_reset_tokens(token_hash);
```

Create corresponding Rust models in src/models/:
- refresh_token.rs with RefreshToken struct
- magic_link.rs with MagicLinkToken struct
- password_reset.rs with PasswordResetToken struct

Each model should have:
- Main struct with FromRow derive
- Create struct for insertions
- Methods for checking expiry and usage
```

---

## Prompt 2.3: Create Subscriptions and Payments Tables

```text
Create migrations for subscription and payment tracking.

Migration 1: Subscriptions
Run: sqlx migrate add create_subscriptions

```sql
CREATE TABLE subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    stripe_subscription_id VARCHAR(255) NOT NULL UNIQUE,
    stripe_price_id VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL,
    current_period_start TIMESTAMPTZ NOT NULL,
    current_period_end TIMESTAMPTZ NOT NULL,
    cancel_at_period_end BOOLEAN NOT NULL DEFAULT FALSE,
    canceled_at TIMESTAMPTZ,
    amount INTEGER NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'usd',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_subscriptions_user_id ON subscriptions(user_id);
CREATE INDEX idx_subscriptions_stripe_subscription_id ON subscriptions(stripe_subscription_id);
CREATE INDEX idx_subscriptions_status ON subscriptions(status);
```

Migration 2: Payment History
Run: sqlx migrate add create_payment_history

```sql
CREATE TABLE payment_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    subscription_id UUID REFERENCES subscriptions(id),
    stripe_payment_intent_id VARCHAR(255) UNIQUE,
    stripe_invoice_id VARCHAR(255),
    amount INTEGER NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'usd',
    status VARCHAR(50) NOT NULL,
    failure_reason TEXT,
    refunded_at TIMESTAMPTZ,
    refund_amount INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_payment_history_user_id ON payment_history(user_id);
CREATE INDEX idx_payment_history_subscription_id ON payment_history(subscription_id);
CREATE INDEX idx_payment_history_created_at ON payment_history(created_at);
```

Create Rust models:
1. src/models/subscription.rs:
   - Subscription struct
   - SubscriptionStatus enum (Active, PastDue, Canceled, Trialing, Incomplete, IncompleteExpired, Unpaid, Paused)
   - CreateSubscription struct
   - SubscriptionResponse struct

2. src/models/payment.rs:
   - PaymentHistory struct
   - PaymentStatus enum (Succeeded, Failed, Pending, Refunded)
   - CreatePayment struct
   - PaymentResponse struct

Update src/models/mod.rs to export new models.
```

---

## Prompt 2.4: Create Applications Table

```text
Create migration for the applications table.

Run: sqlx migrate add create_applications

```sql
CREATE TABLE applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    slug VARCHAR(100) NOT NULL UNIQUE,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    icon_url VARCHAR(500),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    maintenance_mode BOOLEAN NOT NULL DEFAULT FALSE,
    maintenance_message TEXT,
    container_name VARCHAR(255) NOT NULL,
    health_check_url VARCHAR(500),
    version VARCHAR(50),
    source_code_url VARCHAR(500),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_applications_slug ON applications(slug);
CREATE INDEX idx_applications_is_active ON applications(is_active);
```

Create seed migration:
Run: sqlx migrate add seed_applications

```sql
INSERT INTO applications (name, slug, display_name, description, container_name, health_check_url, source_code_url, version)
VALUES
    ('rus', 'rus', 'RUS - URL Shortener', 'Fast, simple URL shortening with QR code generation. Built with Rust for maximum performance.', 'rus', 'http://rus:8080/health', 'https://github.com/example/rus', '1.0.0'),
    ('rustylinks', 'rustylinks', 'Rusty Links', 'Bookmark management made simple. Organize, tag, and access your bookmarks from anywhere.', 'rustylinks', 'http://rustylinks:8080/health', 'https://github.com/example/rustylinks', '1.0.0');
```

Create src/models/application.rs:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Application {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub display_name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub is_active: bool,
    pub maintenance_mode: bool,
    pub maintenance_message: Option<String>,
    pub container_name: String,
    pub health_check_url: Option<String>,
    pub version: Option<String>,
    pub source_code_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

Add ApplicationResponse for public API responses (exclude internal fields like container_name).
```

---

## Prompt 2.5: Create Audit Logs and Admin Tables

```text
Create migrations for audit logging and admin features.

Migration 1: Audit Logs
Run: sqlx migrate add create_audit_logs

```sql
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id UUID REFERENCES users(id),
    actor_email VARCHAR(255),
    actor_role VARCHAR(50),
    actor_ip_address INET,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(100),
    resource_id UUID,
    old_values JSONB,
    new_values JSONB,
    metadata JSONB,
    is_admin_action BOOLEAN NOT NULL DEFAULT FALSE,
    severity VARCHAR(20) NOT NULL DEFAULT 'info',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_actor_id ON audit_logs(actor_id);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);
CREATE INDEX idx_audit_logs_resource_type ON audit_logs(resource_type);
CREATE INDEX idx_audit_logs_is_admin_action ON audit_logs(is_admin_action);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);
CREATE INDEX idx_audit_logs_severity ON audit_logs(severity);
```

Migration 2: Admin Notifications
Run: sqlx migrate add create_admin_notifications

```sql
CREATE TABLE admin_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    type VARCHAR(100) NOT NULL,
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    metadata JSONB,
    user_id UUID REFERENCES users(id),
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    read_by UUID REFERENCES users(id),
    read_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_admin_notifications_type ON admin_notifications(type);
CREATE INDEX idx_admin_notifications_is_read ON admin_notifications(is_read);
CREATE INDEX idx_admin_notifications_created_at ON admin_notifications(created_at);
```

Create Rust models:

1. src/models/audit_log.rs:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   #[serde(rename_all = "snake_case")]
   pub enum AuditAction {
       UserLogin,
       UserLogout,
       UserRegistered,
       MagicLinkRequested,
       MagicLinkUsed,
       PasswordResetRequested,
       PasswordResetCompleted,
       PasswordChanged,
       SubscriptionCreated,
       SubscriptionCanceled,
       SubscriptionReactivated,
       PaymentSucceeded,
       PaymentFailed,
       GracePeriodStarted,
       GracePeriodEnded,
       AdminUserImpersonated,
       AdminSubscriptionGranted,
       AdminSubscriptionRevoked,
       AdminUserDeactivated,
       AdminUserActivated,
       ApplicationMaintenanceToggled,
   }

   #[derive(Debug, Clone, Serialize, Deserialize)]
   #[serde(rename_all = "snake_case")]
   pub enum AuditSeverity {
       Info,
       Warning,
       Error,
       Critical,
   }
   ```

2. src/models/admin_notification.rs:
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   #[serde(rename_all = "snake_case")]
   pub enum NotificationType {
       NewSignup,
       PaymentFailed,
       SubscriptionCanceled,
       GracePeriodExpiring,
       SystemAlert,
   }
   ```
```

---

## Prompt 2.6: Create Rate Limits Table

```text
Create migration for rate limiting.

Run: sqlx migrate add create_rate_limits

```sql
CREATE TABLE rate_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key VARCHAR(255) NOT NULL,
    action VARCHAR(100) NOT NULL,
    count INTEGER NOT NULL DEFAULT 1,
    window_start TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_rate_limit UNIQUE (key, action)
);

CREATE INDEX idx_rate_limits_key_action ON rate_limits(key, action);
CREATE INDEX idx_rate_limits_window_start ON rate_limits(window_start);
```

Create src/models/rate_limit.rs:
```rust
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RateLimit {
    pub id: Uuid,
    pub key: String,
    pub action: String,
    pub count: i32,
    pub window_start: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub action: &'static str,
    pub max_requests: i32,
    pub window_seconds: i64,
}

impl RateLimitConfig {
    pub const LOGIN: Self = Self {
        action: "login",
        max_requests: 5,
        window_seconds: 60,
    };

    pub const MAGIC_LINK: Self = Self {
        action: "magic_link",
        max_requests: 3,
        window_seconds: 600,
    };

    pub const PASSWORD_RESET: Self = Self {
        action: "password_reset",
        max_requests: 3,
        window_seconds: 3600,
    };

    pub const API_AUTH: Self = Self {
        action: "api_auth",
        max_requests: 100,
        window_seconds: 60,
    };

    pub const API_UNAUTH: Self = Self {
        action: "api_unauth",
        max_requests: 20,
        window_seconds: 60,
    };
}
```
```

---

## Prompt 2.7: Create Database Repository Layer

```text
Create a repository layer for database operations.

Create src/repositories/mod.rs to organize all repositories.

Create src/repositories/user_repository.rs:
```rust
pub struct UserRepository;

impl UserRepository {
    pub async fn create(
        pool: &PgPool,
        email: &str,
        password_hash: Option<&str>,
    ) -> Result<User, AppError>;

    pub async fn find_by_id(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<Option<User>, AppError>;

    pub async fn find_by_email(
        pool: &PgPool,
        email: &str,
    ) -> Result<Option<User>, AppError>;

    pub async fn find_by_stripe_customer_id(
        pool: &PgPool,
        customer_id: &str,
    ) -> Result<Option<User>, AppError>;

    pub async fn update_password(
        pool: &PgPool,
        user_id: Uuid,
        password_hash: &str,
    ) -> Result<(), AppError>;

    pub async fn update_subscription_status(
        pool: &PgPool,
        user_id: Uuid,
        status: SubscriptionStatus,
    ) -> Result<(), AppError>;

    pub async fn update_stripe_customer_id(
        pool: &PgPool,
        user_id: Uuid,
        customer_id: &str,
    ) -> Result<(), AppError>;

    pub async fn lock_price(
        pool: &PgPool,
        user_id: Uuid,
        price_id: &str,
        amount: i32,
    ) -> Result<(), AppError>;

    pub async fn set_grace_period(
        pool: &PgPool,
        user_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<(), AppError>;

    pub async fn clear_grace_period(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<(), AppError>;

    pub async fn update_last_login(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<(), AppError>;

    pub async fn soft_delete(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<(), AppError>;

    pub async fn list_paginated(
        pool: &PgPool,
        page: i32,
        per_page: i32,
        search: Option<&str>,
        status_filter: Option<SubscriptionStatus>,
    ) -> Result<(Vec<User>, i64), AppError>;
}
```

Use sqlx::query! and sqlx::query_as! macros for compile-time verification.

Write integration tests that:
- Create a user
- Find by email
- Update fields
- Verify soft delete works
- Test pagination

Use a test database or transactions that roll back.
```

---

## Prompt 2.8: Create Remaining Repositories

```text
Create repository implementations for all other tables.

1. src/repositories/token_repository.rs:
   - RefreshToken CRUD
   - MagicLinkToken CRUD
   - PasswordResetToken CRUD
   - Methods to find valid (non-expired, non-used) tokens
   - Methods to revoke/mark as used
   - Cleanup method for expired tokens

2. src/repositories/subscription_repository.rs:
   - Create subscription
   - Find by user_id
   - Find by stripe_subscription_id
   - Update status
   - Mark as canceled
   - List subscriptions with pagination

3. src/repositories/payment_repository.rs:
   - Create payment record
   - Find by user_id
   - Find by stripe_payment_intent_id
   - Update status
   - List with pagination and date range filter

4. src/repositories/application_repository.rs:
   - List all active applications
   - Find by slug
   - Update maintenance mode
   - Toggle active status

5. src/repositories/audit_log_repository.rs:
   - Create log entry
   - List with filters (action, actor, date range, admin only)
   - Pagination support
   - Helper method for common audit scenarios

6. src/repositories/notification_repository.rs:
   - Create notification
   - List unread
   - Mark as read
   - Count unread

7. src/repositories/rate_limit_repository.rs:
   - Check and increment
   - Reset for key/action
   - Cleanup expired entries

Each repository should:
- Use AppError for error handling
- Log database errors with tracing
- Use parameterized queries (no string concatenation)
- Be stateless (no stored connection)

Export all from src/repositories/mod.rs
```

---

## Validation Checklist

After completing all prompts in this section, verify:

- [ ] All migrations run successfully: `sqlx migrate run`
- [ ] Database schema matches specification
- [ ] All models compile and derive correctly
- [ ] All repositories have basic CRUD operations
- [ ] Repository integration tests pass
- [ ] `cargo sqlx prepare` generates offline query data
- [ ] No N+1 query patterns in repository methods

---

## Next Steps

Proceed to **[03-authentication.md](./03-authentication.md)** to implement the authentication system.
