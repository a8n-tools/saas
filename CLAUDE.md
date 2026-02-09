# CLAUDE.md - a8n.tools Project Context

> This file contains essential context for AI assistants working on the a8n.tools project.
> For the full technical specification, see `a8n-tools-specification.md`.

## Project Overview

**a8n.tools** is a SaaS platform hosting developer/productivity tools. We sell convenience and managed hosting for open-source applications.

- **Business Model:** Two-tier membership model:
  - **Personal:** $3/month — All apps, documentation access, community support
  - **Business (PSA):** $15/month — All apps, documentation access, priority support, invoice billing, team features (coming soon)
- **Key Differentiator:** Fixed price for life — early adopters lock in their rate forever
- **Target Launch:** End of January 2025 (ideal) / Late February 2025 (deadline)
- **Team:** 3 full-stack developers

## Current Applications

1. **RUS (Rust URL Shortener)** — URL shortening with QR codes (`rus.a8n.tools`)
2. **Rusty Links** — Bookmark management (`rustylinks.a8n.tools`)

## Tech Stack

| Layer | Technology |
|-------|------------|
| Backend | Rust, Actix-Web (latest stable) |
| Frontend | React 18+, Vite, TypeScript, Tailwind CSS, shadcn/ui |
| Database | PostgreSQL 16+ |
| Containerization | Docker, Docker Compose |
| Reverse Proxy | Traefik (wildcard SSL for *.a8n.tools) |
| Email | Stalwart (self-hosted, SPF/DKIM/DMARC configured) |
| Monitoring | Prometheus, Grafana |
| Error Tracking | GlitchTip (self-hosted) |
| Payments | Stripe (hosted checkout) |

## Domain Structure

| Subdomain | Purpose |
|-----------|---------|
| `a8n.tools` | Landing/marketing page |
| `app.a8n.tools` | User dashboard |
| `api.a8n.tools` | Backend API |
| `admin.a8n.tools` | Admin panel |
| `rus.a8n.tools` | RUS application |
| `rustylinks.a8n.tools` | Rusty Links application |

## Architecture Decisions

### Authentication
- **JWT with Ed25519** (asymmetric) — apps validate tokens locally using public key
- **Access tokens:** 15 min expiry, HTTP-only cookie on `.a8n.tools`
- **Refresh tokens:** 30 days expiry, stored in DB for tracking
- **Auth methods:** Email/password (Argon2id) + Magic links (passwordless)
- **Cookie flags:** HttpOnly, Secure, SameSite=Lax

### Membership Model
- Two-tier pricing:
  - **Personal:** $3/month — All apps, documentation access, community support
  - **Business:** $15/month — All apps, documentation access, priority support, invoice billing, team features (coming soon)
- `price_locked` boolean + `locked_price_id` string + `locked_price_amount` track fixed-price-for-life
- `membership_tier` tracks the user's tier (personal/business)
- **Documentation behind paywall:** All app docs, tutorials, and guides require active membership
- Grace period: 30 days after payment failure before access revoked
- No free tier, no trial

### User Roles
- **Member:** Access apps and documentation (if active), manage own account
- **Admin:** Full system access, user management, impersonation

### Data Isolation
- Platform DB: users, memberships, audit logs
- Each app has its own isolated PostgreSQL database
- Apps receive user ID from JWT, manage own user data

### JWT Authentication for Child Apps

Child applications (RUS, Rusty Links, etc.) authenticate users via shared JWT tokens:

```
┌─────────────────────────────────────────────────────────────────┐
│                     .a8n.tools domain                           │
│                                                                 │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐      │
│  │ app.a8n.tools│    │ rus.a8n.tools│    │rustylinks.   │      │
│  │  (Dashboard) │    │    (RUS)     │    │  a8n.tools   │      │
│  └──────────────┘    └──────────────┘    └──────────────┘      │
│         │                   │                   │               │
│         └───────────────────┼───────────────────┘               │
│                             │                                   │
│              HTTP-only cookie: access_token                     │
│              (automatically sent to all subdomains)             │
└─────────────────────────────────────────────────────────────────┘
```

**How it works:**
1. User logs in at `app.a8n.tools` → JWT cookie set on `.a8n.tools` domain
2. User visits `rus.a8n.tools` → browser automatically sends the cookie
3. Child app validates JWT locally using shared `JWT_SECRET` (no API call needed)
4. Child app extracts claims: `user_id`, `email`, `membership_status`, `membership_tier`

**Security factors at play here:**
1. HTTP-only cookies - JavaScript can't read the token
2. Secure flag - Cookie only sent over HTTPS
3. SameSite=Lax - Protects against CSRF
4. 15-minute expiry - Short window if anything goes wrong

**JWT Claims available to child apps:**
```json
{
  "sub": "user-uuid",              // User ID - use this to scope all data
  "email": "user@example.com",
  "role": "member",
  "membership_status": "active",   // "active", "grace_period", "canceled", etc.
  "membership_tier": "personal",   // "personal" or "business"
  "exp": 1704068100                // Expiration (15 min)
}
```

**Shared Auth Library (`apps/shared/a8n-auth/`):**
- `AuthenticatedUser` — Requires valid JWT, returns 401 if missing
- `MemberUser` — Requires valid JWT + active membership, returns 403 if no membership
- `OptionalUser` — Returns `None` for guests, never fails

**Usage in child apps:**
```rust
use a8n_auth::MemberUser;

async fn create_item(user: MemberUser, body: web::Json<CreateItem>) -> HttpResponse {
    // user.user_id() - UUID to scope data
    // user.membership_tier_enum() - Personal or Business
    sqlx::query("INSERT INTO items (user_id, ...) VALUES ($1, ...)")
        .bind(user.user_id())
        .execute(&pool)
        .await?;
}
```

**Important:** All child apps must use the same `JWT_SECRET` environment variable as the main platform.

## Database Tables

1. `users` — accounts, membership status, price locking
2. `refresh_tokens` — multi-device session tracking
3. `magic_link_tokens` — passwordless auth
4. `password_reset_tokens` — password recovery
5. `subscriptions` — Stripe membership data (table name kept for Stripe compatibility)
6. `payment_history` — payment records
7. `applications` — registered apps metadata
8. `audit_logs` — security event logging
9. `admin_notifications` — admin alerts
10. `rate_limits` — rate limiting tracking

## API Structure

Base URL: `https://api.a8n.tools/v1`

### Key Endpoints
- `POST /auth/register` — Create account
- `POST /auth/login` — Email/password login
- `POST /auth/magic-link` — Request magic link
- `POST /auth/magic-link/verify` — Verify magic link
- `POST /auth/refresh` — Refresh access token
- `POST /auth/logout` — Clear tokens
- `POST /auth/password-reset` — Request reset
- `POST /auth/password-reset/confirm` — Complete reset
- `GET /users/me` — Current user
- `GET /memberships/me` — Current membership
- `POST /memberships/checkout` — Create Stripe checkout
- `POST /memberships/cancel` — Cancel membership
- `GET /applications` — List available apps
- `POST /webhooks/stripe` — Stripe webhook handler

### Admin Endpoints (`/admin/*`)
- User management (list, view, activate, deactivate, impersonate)
- Membership management (grant, revoke, extend grace period)
- Application management (toggle active, maintenance mode)
- Audit logs, notifications, system health

## Stripe Integration

### Webhook Events to Handle
- `checkout.session.completed` — New membership
- `customer.subscription.updated` — Status changes (Stripe event name)
- `customer.subscription.deleted` — Cancellation (Stripe event name)
- `invoice.payment_succeeded` — Successful payment
- `invoice.payment_failed` — Start grace period

### Grace Period Flow
1. Payment fails → status = 'past_due', start 30-day grace
2. Send emails: Day 1, 7, 14, 25, 30
3. Day 30: Revoke access, status = 'canceled'

## Email Templates Needed

1. Magic link (15 min expiry)
2. Password reset (1 hour expiry)
3. Welcome (membership confirmed)
4. Payment failed (grace period notice)
5. Grace period warnings (Day 7, 14, 25)
6. Membership canceled
7. Payment succeeded (receipt)

## Security Requirements

### Rate Limits
- Login: 5/minute per email
- Magic link: 3/10 minutes per email
- Password reset: 3/hour per email
- API (auth): 100/minute per user
- API (unauth): 20/minute per IP

### Input Validation
- Email: proper format validation
- Password: 12+ chars, mixed case, no common passwords
- All queries: parameterized (sqlx)

### Security Headers (via Traefik)
- HSTS with preload
- X-Frame-Options: DENY
- X-Content-Type-Options: nosniff
- CSP configured for Stripe

## Frontend Structure

```
frontend/
├── src/
│   ├── api/           # API client functions
│   ├── components/
│   │   ├── ui/        # shadcn/ui components
│   │   ├── layout/    # Header, Footer, Sidebar
│   │   ├── auth/      # Login, Register forms
│   │   ├── dashboard/ # App cards, membership status
│   │   └── admin/     # Admin components
│   ├── pages/
│   │   ├── public/    # Landing, Pricing, Auth pages
│   │   ├── dashboard/ # Protected user pages
│   │   ├── admin/     # Admin pages
│   │   └── errors/    # 404, 500, 403, MembershipRequired
│   ├── hooks/         # useAuth, useMembership, etc.
│   ├── stores/        # Zustand stores
│   ├── lib/           # Utilities
│   └── types/         # TypeScript types
```

## Color Theme

```css
--primary-500: #f97316;  /* Main orange */
--rust: #b7410e;         /* Rust accent */
```

## Development Commands

```bash
# Start development environment
docker-compose -f docker-compose.dev.yml up -d

# Run database migrations
cd api && cargo sqlx migrate run

# Run backend tests
cd api && cargo test

# Run frontend tests
cd frontend && npm test

# Generate JWT keys
openssl genpkey -algorithm Ed25519 -out secrets/jwt_private.pem
openssl pkey -in secrets/jwt_private.pem -pubout -out secrets/jwt_public.pem
chmod 600 secrets/jwt_private.pem

# Deploy
docker-compose pull && docker-compose up -d

# View logs
docker-compose logs -f api
```

## Project Structure

```
a8n-tools/
├── api/                    # Rust backend (Actix-Web)
│   ├── src/
│   │   ├── main.rs
│   │   ├── routes/         # Route handlers
│   │   ├── models/         # Database models
│   │   ├── services/       # Business logic
│   │   ├── middleware/     # Auth, rate limiting
│   │   └── utils/          # Helpers
│   ├── migrations/         # SQL migrations
│   ├── Cargo.toml
│   └── Dockerfile
├── frontend/               # React SPA
│   ├── src/
│   ├── package.json
│   └── Dockerfile
├── apps/
│   ├── shared/
│   │   └── a8n-auth/      # Shared JWT auth library for child apps
│   ├── rus/               # RUS application
│   └── rustylinks/        # Rusty Links application
├── monitoring/
│   ├── prometheus.yml
│   └── grafana/
├── secrets/
│   ├── jwt_private.pem
│   └── jwt_public.pem
├── docker-compose.yml
├── docker-compose.dev.yml
├── .env.example
├── CLAUDE.md              # This file
└── a8n-tools-specification.md  # Full spec
```

## Phase 1 MVP Checklist

### P0 (Launch Blockers)
- [ ] Landing page
- [ ] User registration (email/password)
- [ ] Magic link authentication
- [ ] JWT auth system with refresh tokens
- [ ] Password reset flow
- [ ] Stripe checkout integration (both tiers)
- [ ] Membership management (view, cancel, reactivate)
- [ ] Fixed price for life tracking
- [ ] Documentation paywall
- [ ] User dashboard
- [ ] Application listing with status
- [ ] RUS integration
- [ ] Rusty Links integration
- [ ] Subdomain routing (Traefik)
- [ ] Basic admin panel
- [ ] Terms of Service page
- [ ] Privacy Policy page

### P1 (Should Have)
- [ ] Grace period handling (30 days)
- [ ] All email templates
- [ ] Full admin user management
- [ ] Admin membership management
- [ ] Audit logging
- [ ] Admin notifications dashboard
- [ ] Rate limiting
- [ ] Prometheus metrics
- [ ] Grafana dashboards
- [ ] GlitchTip error tracking

### P2 (Nice to Have)
- [ ] Admin user impersonation
- [ ] App maintenance mode
- [ ] Cookie Policy page
- [ ] Dark/light mode toggle

## Key Crates (Rust)

```toml
[dependencies]
actix-web = "4"
actix-cors = "0.7"
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "uuid", "chrono"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
jsonwebtoken = "9"
argon2 = "0.5"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
validator = { version = "0.16", features = ["derive"] }
lettre = "0.11"
tera = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json"] }
stripe-rust = "0.25"
thiserror = "1"
anyhow = "1"
```

## Key NPM Packages (Frontend)

```json
{
  "dependencies": {
    "react": "^18",
    "react-dom": "^18",
    "react-router-dom": "^6",
    "@tanstack/react-query": "^5",
    "zustand": "^4",
    "react-hook-form": "^7",
    "zod": "^3",
    "@hookform/resolvers": "^3",
    "tailwindcss": "^3",
    "class-variance-authority": "^0.7",
    "clsx": "^2",
    "tailwind-merge": "^2",
    "lucide-react": "^0.300"
  },
  "devDependencies": {
    "typescript": "^5",
    "vite": "^5",
    "@vitejs/plugin-react": "^4",
    "eslint": "^8",
    "prettier": "^3"
  }
}
```

## Environment Variables

```bash
# Database (Main Platform)
DATABASE_URL=postgres://a8n:password@localhost:5432/a8n_platform

# Child App Databases
RUS_DB_PASSWORD=rus_password
RUSTYLINKS_DB_PASSWORD=rustylinks_password

# JWT (shared across platform and all child apps)
JWT_SECRET=your-secret-key-at-least-32-characters

# Stripe
STRIPE_SECRET_KEY=sk_live_...
STRIPE_WEBHOOK_SECRET=whsec_...
STRIPE_PRICE_ID=price_a8n_personal_v1
STRIPE_BUSINESS_PRICE_ID=price_a8n_business_v1

# Email
SMTP_HOST=stalwart
SMTP_PORT=25
SMTP_FROM=noreply@a8n.tools

# App
RUST_LOG=info
ENVIRONMENT=production
```

## Notes for Development

1. **Always use parameterized queries** — sqlx handles this automatically
2. **Hash tokens before storing** — never store raw magic link or reset tokens
3. **Log all auth events** — audit_logs table for security tracking
4. **Test Stripe webhooks locally** — use Stripe CLI: `stripe listen --forward-to localhost:8080/v1/webhooks/stripe`
5. **JWT public key shared with apps** — mount as read-only volume
6. **Cookie domain is `.a8n.tools`** — enables SSO across subdomains
7. **Validate membership status on every app request** — check JWT claims
8. **Admin actions require extra logging** — set `is_admin_action = true` in audit logs

## Tagline Options

1. "Developer tools, automated."
2. "Your tools, our servers."
3. "Open source. Managed for you."
4. "Tools that just work."
5. "Build more. Manage less."

---

*Last updated: February 3, 2026*
