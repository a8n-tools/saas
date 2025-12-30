# a8n.tools - Project Overview & Blueprint

## Executive Summary

**a8n.tools** is a SaaS platform hosting developer and productivity tools. It provides managed, hosted versions of open-source applications, selling convenience, reliability, and support.

### Value Proposition
- **Convenience:** No server setup, maintenance, or updates required
- **Reliability:** Managed infrastructure with monitoring and backups
- **Support:** Dedicated support for subscribers
- **Cost-effective:** $3/month for access to all current and future tools
- **Early Adopter Reward:** Fixed price for life

### Initial Applications
1. **RUS (Rust URL Shortener):** URL shortening with QR code generation
2. **Rusty Links:** Bookmark management application

---

## Technology Stack

| Layer | Technology |
|-------|------------|
| Backend API | Rust, Actix-Web |
| Frontend | React 18+, Vite, TypeScript |
| Styling | Tailwind CSS, shadcn/ui |
| Database | PostgreSQL 16+ |
| Containerization | Docker, Docker Compose |
| Reverse Proxy | Traefik |
| Email | Stalwart (self-hosted) |
| Monitoring | Prometheus, Grafana |
| Error Tracking | GlitchTip (self-hosted) |

---

## Domain Structure

| Subdomain | Purpose |
|-----------|---------|
| `a8n.tools` | Marketing/landing page |
| `app.a8n.tools` | User dashboard |
| `api.a8n.tools` | Backend API |
| `admin.a8n.tools` | Admin panel |
| `rus.a8n.tools` | RUS application |
| `rustylinks.a8n.tools` | Rusty Links application |

---

## Implementation Blueprint

### Phase 1: Foundation (Prompts 01-02)
- Project scaffolding
- Docker development environment
- Database schema and migrations

### Phase 2: Authentication (Prompt 03)
- JWT with Ed25519
- Email/password registration
- Magic link authentication
- Password reset flow
- Session management

### Phase 3: Core API (Prompt 04)
- User endpoints
- Application listing
- Standard response formats
- Error handling

### Phase 4: Payments (Prompt 05)
- Stripe integration
- Checkout flow
- Webhook handling
- Subscription management
- Grace period logic

### Phase 5: Frontend Foundation (Prompts 06-08)
- React/Vite setup
- Authentication UI
- User dashboard
- Application access

### Phase 6: Admin (Prompt 09)
- Admin panel
- User management
- Subscription management
- Audit logs

### Phase 7: Email (Prompt 10)
- Stalwart configuration
- Email templates
- Transactional emails

### Phase 8: Infrastructure (Prompts 11-12)
- Production Docker setup
- Traefik routing
- Prometheus/Grafana
- GlitchTip error tracking

### Phase 9: Hardening (Prompts 13-14)
- Security headers
- Rate limiting
- Comprehensive testing

---

## Build Order Rationale

Each step builds on the previous:

1. **Setup first** - Can't build without scaffolding
2. **Database second** - Everything needs data persistence
3. **Auth third** - Most endpoints require authentication
4. **Core API fourth** - Foundation for frontend
5. **Stripe fifth** - Core business logic
6. **Frontend sixth** - Consumes the API
7. **Admin seventh** - Extends existing patterns
8. **Email eighth** - Enhances existing flows
9. **Infrastructure ninth** - Productionizes existing code
10. **Security last** - Hardens the complete system

---

## Key Technical Decisions

### JWT Configuration
- Algorithm: EdDSA (Ed25519)
- Access Token: 15 minutes
- Refresh Token: 30 days
- Cookie Domain: `.a8n.tools`
- Flags: HttpOnly, Secure, SameSite=Lax

### Subscription Model
- Price: $3.00 USD/month
- No free tier
- No trial period
- Price locked at signup forever

### User Roles
- **Subscriber:** Paying customer with app access
- **Admin:** Platform operators with full access

---

## File Structure (Target)

```
a8n-tools/
├── api/                    # Rust backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── config/
│   │   ├── handlers/
│   │   ├── models/
│   │   ├── services/
│   │   ├── middleware/
│   │   └── errors/
│   ├── migrations/
│   ├── Cargo.toml
│   └── Dockerfile
├── frontend/               # React frontend
│   ├── src/
│   │   ├── api/
│   │   ├── components/
│   │   ├── pages/
│   │   ├── hooks/
│   │   ├── stores/
│   │   ├── lib/
│   │   └── types/
│   ├── package.json
│   └── Dockerfile
├── apps/                   # Hosted applications
│   ├── rus/
│   └── rustylinks/
├── docker-compose.yml
├── docker-compose.dev.yml
└── docs/
```

---

## Next Steps

Proceed to **[01-project-setup.md](./01-project-setup.md)** to begin implementation.
