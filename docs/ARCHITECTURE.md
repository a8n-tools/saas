# a8n.tools Architecture

This document describes the system architecture of the a8n.tools platform.

## System Overview

```
                                    ┌─────────────────────────────────────────────────────────┐
                                    │                        Internet                         │
                                    └─────────────────────────────────────────────────────────┘
                                                              │
                                                              ▼
                                    ┌─────────────────────────────────────────────────────────┐
                                    │                    Traefik (Reverse Proxy)              │
                                    │         Wildcard SSL (*.a8n.tools)                      │
                                    │         Rate Limiting | Security Headers               │
                                    └─────────────────────────────────────────────────────────┘
                                                              │
                    ┌─────────────────┬───────────────────────┼────────────────────┬──────────────────┐
                    │                 │                       │                    │                  │
                    ▼                 ▼                       ▼                    ▼                  ▼
            ┌───────────────┐ ┌───────────────┐ ┌─────────────────────┐ ┌───────────────┐ ┌───────────────┐
            │  a8n.tools    │ │ app.a8n.tools │ │   api.a8n.tools     │ │ rus.a8n.tools │ │rustylinks...  │
            │   Landing     │ │   Dashboard   │ │      API            │ │     RUS       │ │ Rusty Links   │
            │    (React)    │ │    (React)    │ │   (Actix-Web)       │ │   (Rust)      │ │    (Rust)     │
            └───────────────┘ └───────────────┘ └─────────────────────┘ └───────────────┘ └───────────────┘
                                                              │                    │                  │
                                                              │                    │                  │
                                                              ▼                    │                  │
                                    ┌─────────────────────────────────────────────────────────┐
                                    │                    PostgreSQL 16                        │
                                    │                                                         │
                                    │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
                                    │  │   Platform   │  │     RUS      │  │ Rusty Links  │  │
                                    │  │   Database   │  │   Database   │  │   Database   │  │
                                    │  └──────────────┘  └──────────────┘  └──────────────┘  │
                                    └─────────────────────────────────────────────────────────┘
```

## Service Responsibilities

### Traefik (Reverse Proxy)

- Routes requests based on subdomain
- Handles SSL termination with wildcard certificate
- Applies rate limiting rules
- Adds security headers (HSTS, CSP, etc.)
- Load balancing for horizontal scaling

### Platform API (api.a8n.tools)

The central API that handles:

| Responsibility | Description |
|----------------|-------------|
| Authentication | JWT tokens, magic links, password reset |
| User Management | Registration, profile, preferences |
| Subscriptions | Stripe integration, billing, access control |
| Application Registry | Available apps, user permissions |
| Admin Functions | User/subscription management, audit logs |

### Frontend Applications

| App | Purpose |
|-----|---------|
| Landing (a8n.tools) | Marketing, pricing, public info |
| Dashboard (app.a8n.tools) | User portal, app access, subscription |
| Admin (admin.a8n.tools) | Administration panel |

### Individual Applications

Each hosted application:
- Has its own isolated database
- Validates JWTs using shared public key
- Receives user context from JWT claims
- Manages its own data independently

## Authentication Flow

```
┌────────┐          ┌─────────┐          ┌─────────┐          ┌─────────┐
│ Client │          │ Frontend│          │   API   │          │   DB    │
└───┬────┘          └────┬────┘          └────┬────┘          └────┬────┘
    │                    │                    │                    │
    │  1. Login Request  │                    │                    │
    ├───────────────────►│                    │                    │
    │                    │  2. POST /auth/login                    │
    │                    ├───────────────────►│                    │
    │                    │                    │  3. Verify Password│
    │                    │                    ├───────────────────►│
    │                    │                    │◄───────────────────┤
    │                    │                    │                    │
    │                    │  4. JWT + Refresh  │                    │
    │                    │◄───────────────────┤                    │
    │                    │                    │  5. Store Refresh  │
    │                    │                    ├───────────────────►│
    │  6. Set Cookies    │                    │                    │
    │◄───────────────────┤                    │                    │
    │                    │                    │                    │
```

### JWT Token Structure

```json
{
  "sub": "user_id",
  "email": "user@example.com",
  "role": "subscriber",
  "subscription_status": "active",
  "exp": 1234567890,
  "iat": 1234567890
}
```

### Cookie Configuration

| Cookie | Purpose | Expiry | Flags |
|--------|---------|--------|-------|
| `access_token` | JWT access token | 15 min | HttpOnly, Secure, SameSite=Lax |
| `refresh_token` | Refresh token ID | 30 days | HttpOnly, Secure, SameSite=Lax |

## Subscription Flow

```
┌────────┐          ┌─────────┐          ┌─────────┐          ┌─────────┐
│ Client │          │   API   │          │  Stripe │          │   DB    │
└───┬────┘          └────┬────┘          └────┬────┘          └────┬────┘
    │                    │                    │                    │
    │ 1. Create Checkout │                    │                    │
    ├───────────────────►│                    │                    │
    │                    │ 2. Create Session  │                    │
    │                    ├───────────────────►│                    │
    │                    │◄───────────────────┤                    │
    │ 3. Redirect to Stripe                   │                    │
    │◄───────────────────┤                    │                    │
    ├─────────────────────────────────────────►                    │
    │                    │                    │                    │
    │ 4. Payment         │                    │                    │
    ├─────────────────────────────────────────►                    │
    │◄─────────────────────────────────────────                    │
    │                    │                    │                    │
    │                    │ 5. Webhook Event   │                    │
    │                    │◄───────────────────┤                    │
    │                    │                    │                    │
    │                    │ 6. Update Subscription                  │
    │                    ├─────────────────────────────────────────►
    │                    │                    │                    │
    │ 7. Redirect to Success                  │                    │
    │◄─────────────────────────────────────────                    │
```

## Data Isolation

Each application has complete data isolation:

```
Platform Database (a8n_platform)
├── users
├── subscriptions
├── refresh_tokens
├── audit_logs
└── applications

RUS Database (rus_db)
├── urls
├── url_stats
└── user_settings

Rusty Links Database (rustylinks_db)
├── bookmarks
├── collections
└── user_settings
```

Applications receive user identity from JWT claims and manage their own data schemas independently.

## Security Layers

### Layer 1: Network (Traefik)

- TLS 1.3 with strong cipher suites
- Rate limiting per IP/user
- Security headers injection
- Request size limits

### Layer 2: Application (API)

- Input validation on all endpoints
- Parameterized SQL queries (SQLx)
- JWT verification on protected routes
- CORS configuration

### Layer 3: Data

- Argon2id password hashing
- Token hashing before storage
- Database connection encryption
- Separate databases per app

## Monitoring Stack

```
┌─────────────────────────────────────────────────────────────────┐
│                         Grafana                                  │
│                    (Visualization)                               │
└─────────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┴───────────────┐
              ▼                               ▼
┌─────────────────────────┐     ┌─────────────────────────┐
│       Prometheus        │     │       GlitchTip         │
│    (Metrics Storage)    │     │    (Error Tracking)     │
└─────────────────────────┘     └─────────────────────────┘
              │                               │
              ▼                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Application Services                        │
│               (API, Frontend, Applications)                      │
└─────────────────────────────────────────────────────────────────┘
```

## Deployment Architecture

### Development

```
docker-compose.dev.yml
├── postgres (local data)
├── api (with hot reload)
├── frontend (with HMR)
└── traefik (local routing)
```

### Production

```
docker-compose.yml
├── postgres (persistent volume)
├── api (replicated)
├── frontend (nginx)
├── traefik (Let's Encrypt)
├── prometheus
├── grafana
├── glitchtip
└── stalwart (email)
```

## Scaling Considerations

### Horizontal Scaling

- API is stateless - can run multiple instances
- Traefik handles load balancing
- Session data in database, not memory
- Consider read replicas for DB

### Performance Optimizations

- Connection pooling (SQLx)
- JWT validation without DB lookup
- CDN for static assets
- Database query optimization

## Future Considerations

- **Caching Layer**: Redis for session/rate limit data
- **Message Queue**: For async email sending
- **Search**: Meilisearch for app-specific search
- **CDN**: CloudFlare for global distribution
