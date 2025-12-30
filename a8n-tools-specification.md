# a8n.tools — SaaS Platform Technical Specification

**Version:** 1.0  
**Date:** December 30, 2024  
**Status:** Ready for Implementation  
**Target Launch:** End of January 2025 (ideal) / Late February 2025 (deadline)  
**Team Size:** 3 full-stack developers

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Business Requirements](#2-business-requirements)
3. [System Architecture](#3-system-architecture)
4. [Authentication & Authorization](#4-authentication--authorization)
5. [Database Design](#5-database-design)
6. [API Specification](#6-api-specification)
7. [Frontend Application](#7-frontend-application)
8. [Stripe Integration](#8-stripe-integration)
9. [Email System](#9-email-system)
10. [Admin Panel](#10-admin-panel)
11. [Infrastructure & Deployment](#11-infrastructure--deployment)
12. [Monitoring & Observability](#12-monitoring--observability)
13. [Security](#13-security)
14. [Testing Strategy](#14-testing-strategy)
15. [CI/CD Pipeline](#15-cicd-pipeline)
16. [Error Handling](#16-error-handling)
17. [Legal Pages](#17-legal-pages)
18. [Phase 1 Scope & Deliverables](#18-phase-1-scope--deliverables)
19. [Phase 2+ Roadmap](#19-phase-2-roadmap)
20. [Appendices](#20-appendices)

---

## 1. Executive Summary

### 1.1 Product Overview

**a8n.tools** is a Software-as-a-Service platform that hosts developer and productivity tools. The platform provides managed, hosted versions of open-source applications, selling convenience, reliability, and support rather than proprietary software.

### 1.2 Value Proposition

- **Convenience:** No server setup, maintenance, or updates required
- **Reliability:** Managed infrastructure with monitoring and backups
- **Support:** Dedicated support for subscribers
- **Cost-effective:** $3/month for access to all current and future tools
- **Early Adopter Reward:** Fixed price for life — subscribers lock in their rate forever

### 1.3 Initial Applications

1. **RUS (Rust URL Shortener):** URL shortening service with QR code generation
2. **Rusty Links:** Bookmark management application

### 1.4 Technology Stack

| Layer | Technology |
|-------|------------|
| Backend API | Rust, Actix-Web (latest) |
| Frontend | React 18+, Vite, TypeScript |
| Styling | Tailwind CSS, shadcn/ui |
| Database | PostgreSQL 16+ |
| Containerization | Docker, Docker Compose |
| Reverse Proxy | Traefik |
| Email | Stalwart (self-hosted) |
| Monitoring | Prometheus, Grafana |
| Error Tracking | GlitchTip (self-hosted) |

---

## 2. Business Requirements

### 2.1 Subscription Model

| Attribute | Value |
|-----------|-------|
| Monthly Price | $3.00 USD |
| Billing Cycle | Monthly (recurring) |
| Free Tier | None |
| Trial Period | None |

### 2.2 Fixed Price for Life

- The subscription price at signup is locked forever
- Future price increases do not affect existing subscribers
- Access to all future applications included at no additional cost
- Tracked via database flag AND Stripe Price ID

### 2.3 User Roles

| Role | Description |
|------|-------------|
| Subscriber | Paying customer - access all apps, manage account |
| Admin | Platform operators - full system access |

### 2.4 Account Structure

- **Phase 1:** Individual accounts only
- **Phase 2+:** Organization/team accounts

---

## 3. System Architecture

### 3.1 Domain Structure

| Subdomain | Purpose |
|-----------|---------|
| `a8n.tools` | Marketing/landing page |
| `app.a8n.tools` | User dashboard |
| `api.a8n.tools` | Backend API |
| `admin.a8n.tools` | Admin panel |
| `rus.a8n.tools` | RUS application |
| `rustylinks.a8n.tools` | Rusty Links application |

### 3.2 Container Architecture

```yaml
services:
  traefik:        # Reverse proxy & SSL
  api:            # Actix-Web backend
  frontend:       # React applications
  postgres:       # Platform database
  rus:            # RUS application
  rus-db:         # RUS database
  rustylinks:     # Rusty Links application
  rustylinks-db:  # Rusty Links database
  stalwart:       # Email server
  prometheus:     # Metrics
  grafana:        # Dashboards
  glitchtip:      # Error tracking
```

### 3.3 JWT Configuration

| Parameter | Value |
|-----------|-------|
| Algorithm | EdDSA (Ed25519) |
| Access Token Expiry | 15 minutes |
| Refresh Token Expiry | 30 days |
| Cookie Domain | `.a8n.tools` |
| Cookie Flags | HttpOnly, Secure, SameSite=Lax |

### 3.4 JWT Payload

```json
{
  "sub": "user_uuid",
  "email": "user@example.com",
  "role": "subscriber",
  "subscription_status": "active",
  "price_locked": true,
  "price_id": "price_xxx",
  "iat": 1704067200,
  "exp": 1704068100,
  "jti": "unique_token_id"
}
```

### 3.5 Inter-App Authentication

Apps validate JWTs locally using the platform's public key. No callback to the platform required. JWT is passed via HTTP-only cookie on `.a8n.tools` domain.

---

## 4. Authentication & Authorization

### 4.1 Authentication Methods

**Email + Password:**
- Minimum 12 characters
- Argon2id hashing
- Password strength validation

**Magic Link (Passwordless):**
- 32 bytes, URL-safe base64 encoded
- 15 minute expiry
- Single use
- Rate limit: 3 per email per 10 minutes

### 4.2 Session Management

| Token | Storage | Expiry |
|-------|---------|--------|
| Access Token | HTTP-only cookie | 15 minutes |
| Refresh Token | HTTP-only cookie | 30 days |

- Multi-device support
- "Remember me" functionality

### 4.3 Password Reset

- 32 byte token, 1 hour expiry
- Single use
- Rate limit: 3 per email per hour

### 4.4 Permission Matrix

| Permission | Subscriber | Admin |
|------------|------------|-------|
| AccessApps | ✓ (if active) | ✓ |
| ManageOwnAccount | ✓ | ✓ |
| ViewOwnSubscription | ✓ | ✓ |
| CancelSubscription | ✓ | ✓ |
| ViewAllUsers | ✗ | ✓ |
| ManageUsers | ✗ | ✓ |
| ImpersonateUsers | ✗ | ✓ |
| ManageSubscriptions | ✗ | ✓ |
| ManageApps | ✗ | ✓ |
| ViewAuditLogs | ✗ | ✓ |

---

## 5. Database Design

### 5.1 Users Table

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

### 5.2 Refresh Tokens Table

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
```

### 5.3 Magic Link Tokens Table

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

### 5.4 Password Reset Tokens Table

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

### 5.5 Subscriptions Table

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
```

### 5.6 Payment History Table

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
CREATE INDEX idx_payment_history_created_at ON payment_history(created_at);
```

### 5.7 Applications Table

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

### 5.8 Audit Logs Table

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
CREATE INDEX idx_audit_logs_is_admin_action ON audit_logs(is_admin_action);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);
```

### 5.9 Admin Notifications Table

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

### 5.10 Rate Limits Table

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

### 5.11 Migration Files

```
migrations/
├── 20250101000001_create_users.sql
├── 20250101000002_create_refresh_tokens.sql
├── 20250101000003_create_magic_link_tokens.sql
├── 20250101000004_create_password_reset_tokens.sql
├── 20250101000005_create_subscriptions.sql
├── 20250101000006_create_payment_history.sql
├── 20250101000007_create_applications.sql
├── 20250101000008_create_audit_logs.sql
├── 20250101000009_create_admin_notifications.sql
├── 20250101000010_create_rate_limits.sql
└── 20250101000011_seed_applications.sql
```

---

## 6. API Specification

### 6.1 Overview

| Attribute | Value |
|-----------|-------|
| Base URL | `https://api.a8n.tools` |
| Format | JSON |
| Authentication | JWT via HTTP-only cookie |
| Versioning | URL path (`/v1/...`) |

### 6.2 Standard Response Format

**Success:**
```json
{
  "success": true,
  "data": { ... },
  "meta": { "request_id": "req_xxx", "timestamp": "..." }
}
```

**Error:**
```json
{
  "success": false,
  "error": { "code": "...", "message": "...", "details": { } },
  "meta": { "request_id": "req_xxx", "timestamp": "..." }
}
```

### 6.3 Error Codes

| Code | Status | Description |
|------|--------|-------------|
| VALIDATION_ERROR | 400 | Request validation failed |
| INVALID_CREDENTIALS | 401 | Email or password incorrect |
| TOKEN_EXPIRED | 401 | JWT has expired |
| UNAUTHORIZED | 401 | Authentication required |
| FORBIDDEN | 403 | Insufficient permissions |
| NOT_FOUND | 404 | Resource not found |
| CONFLICT | 409 | Resource already exists |
| RATE_LIMITED | 429 | Too many requests |
| INTERNAL_ERROR | 500 | Server error |

### 6.4 Authentication Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | /v1/auth/register | Register new user |
| POST | /v1/auth/login | Login with email/password |
| POST | /v1/auth/magic-link | Request magic link |
| POST | /v1/auth/magic-link/verify | Verify magic link |
| POST | /v1/auth/refresh | Refresh access token |
| POST | /v1/auth/logout | Logout |
| POST | /v1/auth/password-reset | Request password reset |
| POST | /v1/auth/password-reset/confirm | Complete reset |

### 6.5 User Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | /v1/users/me | Get current user |
| PUT | /v1/users/me/password | Update password |

### 6.6 Subscription Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | /v1/subscriptions/me | Get subscription |
| POST | /v1/subscriptions/checkout | Create checkout session |
| POST | /v1/subscriptions/cancel | Cancel subscription |
| POST | /v1/subscriptions/reactivate | Reactivate |
| POST | /v1/subscriptions/billing-portal | Get billing portal URL |

### 6.7 Application Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | /v1/applications | List applications |

### 6.8 Webhook Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | /v1/webhooks/stripe | Handle Stripe webhooks |

### 6.9 Admin Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | /v1/admin/users | List users |
| GET | /v1/admin/users/:id | Get user details |
| POST | /v1/admin/users/:id/activate | Activate user |
| POST | /v1/admin/users/:id/deactivate | Deactivate user |
| POST | /v1/admin/users/:id/reset-password | Trigger reset email |
| POST | /v1/admin/users/:id/impersonate | Impersonate user |
| POST | /v1/admin/users/:id/subscription/grant | Grant subscription |
| POST | /v1/admin/users/:id/subscription/revoke | Revoke subscription |
| POST | /v1/admin/users/:id/grace-period/extend | Extend grace period |
| GET | /v1/admin/applications | List applications |
| PUT | /v1/admin/applications/:id | Update application |
| POST | /v1/admin/applications/:id/maintenance | Toggle maintenance |
| GET | /v1/admin/audit-logs | Get audit logs |
| GET | /v1/admin/notifications | Get notifications |
| POST | /v1/admin/notifications/:id/read | Mark read |
| GET | /v1/admin/health | System health |

---

## 7. Frontend Application

### 7.1 Technology Stack

| Technology | Purpose |
|------------|---------|
| React 18+ | UI framework |
| Vite 5+ | Build tool |
| TypeScript 5+ | Type safety |
| Tailwind CSS 3+ | Styling |
| shadcn/ui | Component library |
| React Router 6+ | Routing |
| TanStack Query 5+ | Data fetching |
| Zustand 4+ | State management |
| React Hook Form 7+ | Forms |
| Zod 3+ | Validation |

### 7.2 Project Structure

```
frontend/
├── src/
│   ├── api/          # API calls
│   ├── components/   # UI components
│   ├── pages/        # Page components
│   ├── hooks/        # Custom hooks
│   ├── stores/       # State stores
│   ├── lib/          # Utilities
│   ├── types/        # TypeScript types
│   └── styles/       # Global styles
├── public/
└── package.json
```

### 7.3 Routes

**Public:** `/`, `/pricing`, `/login`, `/register`, `/magic-link`, `/password-reset`, `/terms`, `/privacy`

**Protected:** `/dashboard`, `/dashboard/apps`, `/dashboard/account`, `/dashboard/subscription`

**Admin:** `/admin`, `/admin/users`, `/admin/subscriptions`, `/admin/applications`, `/admin/audit-logs`, `/admin/notifications`, `/admin/health`

**Errors:** `/403`, `/500`, `/subscription-required`, `*` (404)

### 7.4 Color Theme

```css
--primary-500: #f97316;  /* Main orange */
--rust: #b7410e;         /* Rust accent */
```

---

## 8. Stripe Integration

### 8.1 Product Configuration

- Product: a8n.tools Subscription
- Price: $3.00 USD / month
- Price ID: `price_a8n_monthly_v1`

### 8.2 Checkout Flow

1. User clicks "Subscribe"
2. Frontend calls `/v1/subscriptions/checkout`
3. API creates Stripe Checkout Session
4. User redirected to Stripe Checkout
5. User completes payment
6. Stripe sends webhook (`checkout.session.completed`)
7. API updates user subscription status
8. Set `price_locked = true` and `locked_price_id`

### 8.3 Webhook Events

- `checkout.session.completed`
- `customer.subscription.created`
- `customer.subscription.updated`
- `customer.subscription.deleted`
- `invoice.payment_succeeded`
- `invoice.payment_failed`

### 8.4 Grace Period

- Duration: 30 days
- Triggered on: `invoice.payment_failed`
- Access continues during grace period
- Scheduled emails: Day 1, 7, 14, 25, 30
- Access revoked after 30 days

---

## 9. Email System

### 9.1 Infrastructure

| Component | Technology |
|-----------|------------|
| Mail Server | Stalwart (self-hosted) |
| DNS | SPF, DKIM, DMARC configured |
| Sending Library | lettre (Rust) |
| Templates | tera or handlebars |

### 9.2 Email Templates

| Template | Trigger |
|----------|---------|
| Magic Link | Magic link request |
| Password Reset | Password reset request |
| Welcome | Subscription created |
| Payment Failed | Payment failure |
| Grace Period Warning | Days 7, 14, 25 of grace |
| Subscription Canceled | Cancellation |
| Payment Succeeded | Successful payment |

All templates include:
- a8n.tools logo
- Rust/orange color scheme
- Dark mode support
- Mobile-responsive design

---

## 10. Admin Panel

### 10.1 Dashboard Widgets

- Active Users (total + new this month)
- Revenue (MTD + trend)
- Active Subscriptions (total + churn rate)
- System Health
- Recent Activity
- Unread Notifications

### 10.2 User Management

- Paginated user list with filters
- User detail view with history
- Actions: activate, deactivate, reset password, impersonate, grant/revoke subscription

### 10.3 Subscription Management

- All subscriptions with status
- Grant complimentary access
- Extend grace periods
- Link to Stripe for refunds

### 10.4 Application Management

- Toggle active/inactive
- Enable/disable maintenance mode
- View usage stats

### 10.5 Audit Logs

Filterable by: action type, actor, admin actions only, date range, severity

### 10.6 Notifications Dashboard

Types: new_signup, payment_failed, subscription_canceled, support_request

---

## 11. Infrastructure & Deployment

### 11.1 Docker Compose Services

```yaml
version: '3.8'

services:
  traefik:
    image: traefik:v3.0
    ports: ["80:80", "443:443"]

  api:
    build: ./api
    environment:
      - DATABASE_URL
      - JWT_PRIVATE_KEY_PATH
      - STRIPE_SECRET_KEY

  frontend:
    build: ./frontend

  postgres:
    image: postgres:16-alpine
    volumes: [postgres-data:/var/lib/postgresql/data]

  redis:
    image: redis:7-alpine

  stalwart:
    image: stalwartlabs/mail-server:latest

  rus:
    build: ./apps/rus

  rus-db:
    image: postgres:16-alpine

  rustylinks:
    build: ./apps/rustylinks

  rustylinks-db:
    image: postgres:16-alpine

  prometheus:
    image: prom/prometheus:v2.48.0

  grafana:
    image: grafana/grafana:10.2.0

  glitchtip:
    image: glitchtip/glitchtip:latest
```

### 11.2 Environment Variables

```bash
DB_PASSWORD=
STRIPE_SECRET_KEY=
STRIPE_WEBHOOK_SECRET=
STRIPE_PRICE_ID=
TRAEFIK_DASHBOARD_AUTH=
GRAFANA_PASSWORD=
GLITCHTIP_SECRET_KEY=
RUS_DB_PASSWORD=
RUSTYLINKS_DB_PASSWORD=
```

### 11.3 API Dockerfile

```dockerfile
FROM rust:1.75-alpine AS builder
RUN apk add --no-cache musl-dev openssl-dev pkgconfig
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:3.19
RUN apk add --no-cache ca-certificates libgcc
COPY --from=builder /app/target/release/a8n-api /app/a8n-api
EXPOSE 8080
CMD ["/app/a8n-api"]
```

### 11.4 Frontend Dockerfile

```dockerfile
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

---

## 12. Monitoring & Observability

### 12.1 Prometheus Metrics

- HTTP requests (total, duration)
- Login attempts
- Active sessions
- Active subscriptions
- Payment failures
- Revenue
- Database connections

### 12.2 Grafana Dashboards

- API Overview
- Business Metrics
- Application Health

### 12.3 Alerting Rules

- High error rate (>0.1 5xx/sec for 5min)
- API down (1min)
- High response time (p95 > 1s for 5min)
- Payment failure spike

### 12.4 Structured Logging

JSON format with: target, level, file, line number, request context

---

## 13. Security

### 13.1 Rate Limiting

| Endpoint | Limit | Window |
|----------|-------|--------|
| Login | 5 | 1 minute |
| Magic Link | 3 | 10 minutes |
| Password Reset | 3 | 1 hour |
| API (auth) | 100 | 1 minute |
| API (unauth) | 20 | 1 minute |

### 13.2 Input Validation

- Email format validation
- Password strength (12+ chars, mixed)
- SQL injection prevention (parameterized queries)
- XSS prevention (React escaping, CSP headers)

### 13.3 Security Headers

```yaml
browserXssFilter: true
contentTypeNosniff: true
frameDeny: true
stsIncludeSubdomains: true
stsPreload: true
stsSeconds: 31536000
```

### 13.4 Secrets Management

- JWT keys: Ed25519 key pair
- Environment variables for secrets
- chmod 600 for private keys

### 13.5 Audit Logging

Events: UserLogin, UserLogout, MagicLinkUsed, PasswordResetCompleted, SubscriptionCreated, SubscriptionCanceled, PaymentSucceeded, PaymentFailed, AdminUserImpersonated, AdminSubscriptionGranted

---

## 14. Testing Strategy

### 14.1 Coverage Target

- Overall: 80-90%
- Critical paths: 95%+ (auth, payments)

### 14.2 Test Types

- **Unit tests:** Password hashing, JWT creation, validation logic
- **Integration tests:** API endpoints, database operations
- **E2E tests:** User flows (Playwright)

---

## 15. CI/CD Pipeline

### 15.1 Pipeline Jobs

1. **rust-checks:** fmt, clippy, audit
2. **rust-tests:** cargo test
3. **frontend-tests:** lint, test:coverage
4. **build-images:** docker build, push
5. **deploy:** docker-compose pull, up

### 15.2 Code Quality Tools

**Rust:** cargo fmt, clippy, cargo audit

**React:** ESLint, Prettier, TypeScript strict

**Pre-commit:** Husky + lint-staged

---

## 16. Error Handling

### 16.1 Response Format

```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable message",
    "details": { ... }
  },
  "meta": { "request_id": "req_abc123" }
}
```

### 16.2 Frontend Handling

- Toast notifications for errors
- User-friendly messages with optional technical details
- Error tracking via GlitchTip

### 16.3 Custom Error Pages

- 404 Not Found
- 500 Server Error
- 403 Forbidden
- Subscription Required

---

## 17. Legal Pages

### 17.1 Terms of Service

Sections: Introduction, Service Description, Account Terms, Billing ($3/month, fixed price for life), Cancellation, Acceptable Use, IP, Liability, Termination, Changes

### 17.2 Privacy Policy

Sections: Info Collected, Usage, Sharing, Security, Retention, Rights, Cookies, Contact

### 17.3 Cookie Policy

Sections: Definition, Cookies Used (auth only), Third-Party (Stripe), Managing Cookies

---

## 18. Phase 1 Scope & Deliverables

### 18.1 MVP Features (P0)

- Landing page
- User registration (email/password)
- Magic link authentication
- JWT authentication system
- Password reset flow
- Stripe checkout integration
- Subscription management
- Fixed price for life tracking
- User dashboard
- Application listing
- RUS integration
- Rusty Links integration
- Subdomain routing
- Basic admin panel
- Terms of Service
- Privacy Policy

### 18.2 Should Have (P1)

- Grace period handling
- All email notifications
- Admin user management
- Admin subscription management
- Admin audit logs
- Admin notifications dashboard
- Rate limiting
- Prometheus metrics
- Grafana dashboards
- Error tracking

### 18.3 Development Timeline (8 Weeks)

| Week | Focus |
|------|-------|
| 1-2 | Foundation (setup, database, JWT auth) |
| 3 | Authentication (magic links, password reset) |
| 4 | Stripe integration |
| 5 | Dashboard & app integration |
| 6 | Admin panel |
| 7 | Polish & infrastructure |
| 8 | Testing & launch prep |

---

## 19. Phase 2+ Roadmap

### Phase 2: Enhanced Authentication
- OAuth login (Google, GitHub)
- 2FA (TOTP)
- Session management (view/revoke)
- Email change
- Self-service account deletion

### Phase 3: Teams & Organizations
- Organization accounts
- Team roles
- Invite system
- Per-app access control
- Per-seat billing

### Phase 4: Platform Enhancements
- OIDC provider
- Subscription pausing
- Download my data (GDPR)
- API keys
- Usage analytics

---

## 20. Appendices

### 20.1 Tagline Ideas

1. "Developer tools, automated."
2. "Your tools, our servers."
3. "Open source. Managed for you."
4. "Tools that just work."
5. "Build more. Manage less."

### 20.2 Color Palette

```css
--primary-500: #f97316;  /* Main orange */
--rust: #b7410e;         /* Rust accent */
```

### 20.3 JWT Key Generation

```bash
openssl genpkey -algorithm Ed25519 -out jwt_private.pem
openssl pkey -in jwt_private.pem -pubout -out jwt_public.pem
chmod 600 jwt_private.pem
chmod 644 jwt_public.pem
```

### 20.4 Useful Commands

```bash
# Start development
docker-compose -f docker-compose.dev.yml up -d

# Run migrations
cd api && cargo sqlx migrate run

# Run tests
cargo test
cd frontend && npm test

# Deploy
docker-compose pull && docker-compose up -d

# View logs
docker-compose logs -f api

# Connect to database
docker exec -it a8n-postgres psql -U a8n -d a8n_platform
```

### 20.5 Database Connection Strings

```
Platform: postgres://a8n:${DB_PASSWORD}@postgres:5432/a8n_platform
RUS: postgres://rus:${RUS_DB_PASSWORD}@rus-db:5432/rus
Rusty Links: postgres://rustylinks:${RUSTYLINKS_DB_PASSWORD}@rustylinks-db:5432/rustylinks
```

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2024-12-30 | Initial specification |

---

*End of Specification Document*
