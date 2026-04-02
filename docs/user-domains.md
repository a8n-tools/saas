# User Domains — Application Subdomain Configuration

## Overview

Applications hosted on the platform are accessed via subdomains (e.g., `go.example.com`, `links.example.com`). The **user domain** feature lets admins configure a custom subdomain for each application, decoupling the user-facing URL from the internal application slug.

Without this feature, an application's URL is derived directly from its slug (`rus.example.com`). With it, admins can assign a friendlier subdomain (`go.example.com`) while keeping the slug unchanged for internal use.

---

## How It Works

### URL Resolution

The frontend constructs application URLs at runtime:

```
subdomain = app.subdomain || app.slug   // fallback to slug if no subdomain set
url       = https://{subdomain}.{APP_DOMAIN}
```

- **`VITE_APP_DOMAIN`** — sets the base domain (e.g., `example.com`)
- If no subdomain is configured, the slug is used as-is

**Example mappings:**

| Application | Slug | Subdomain | Resulting URL |
|------------|------|-----------|--------------|
| RUS | `rus` | `go` | `go.example.com` |
| Rusty Links | `rustylinks` | `links` | `links.example.com` |
| New App | `newapp` | *(none)* | `newapp.example.com` |

### Cross-Subdomain SSO

All subdomains share authentication via a single cookie domain:

- **`COOKIE_DOMAIN`** env var (e.g., `.example.com`) scopes auth cookies to the parent domain
- Access and refresh tokens are set as HTTP-only cookies on this domain
- Any `*.example.com` subdomain can read the cookies, enabling SSO

The API's CORS configuration extracts the domain from `CORS_ORIGIN` and allows all matching subdomains via suffix checking (`origin.ends_with(domain)`).

---

## Database

**Migration:** `api/migrations/20241230000017_add_application_subdomain.sql`

```sql
ALTER TABLE applications ADD COLUMN subdomain VARCHAR(100);

-- Seed existing applications
UPDATE applications SET subdomain = 'go' WHERE slug = 'rus';
UPDATE applications SET subdomain = 'links' WHERE slug = 'rustylinks';
```

The column is nullable — `NULL` means "use the slug as the subdomain."

---

## API

The subdomain field flows through the standard application CRUD endpoints:

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/v1/applications` | GET | Lists active apps (includes subdomain) |
| `/v1/applications/{slug}` | GET | Single app detail |
| `/v1/admin/applications` | GET | Admin list (all apps) |
| `/v1/admin/applications` | POST | Create app (accepts optional `subdomain`) |
| `/v1/admin/applications/{id}` | PUT | Update app (accepts optional `subdomain`) |

**Key files:**
- Model: `api/src/models/application.rs` — `Application`, `CreateApplication`, `UpdateApplication` structs all include `subdomain: Option<String>`
- Repository: `api/src/repositories/application.rs` — uses `COALESCE($6, subdomain)` on update to preserve existing value when not provided
- Handlers: `api/src/handlers/admin.rs` — admin CRUD, `api/src/handlers/application.rs` — public read endpoints

---

## Frontend

### Dashboard (user-facing)

**`frontend/src/pages/dashboard/ApplicationsPage.tsx`**

Each application card displays its URL and links to it:

```tsx
const baseDomain = config.appDomain || 'localhost'
const subdomain = app.subdomain || app.slug
const appUrl = `${subdomain}.${baseDomain}`
```

The "Launch" button opens `https://{appUrl}` in a new tab.

### Admin Panel

**`frontend/src/pages/admin/AdminApplicationsPage.tsx`**

- **Create dialog:** optional subdomain input; placeholder shows the slug; helper text says "Leave empty to use the slug as the subdomain."
- **Edit dialog:** editable subdomain field with the same fallback behavior
- **Application cards:** display the resolved URL as `{subdomain || slug}.{appDomain}`

### Configuration

**`frontend/src/config.ts`**

```ts
appDomain: rc?.appDomain || import.meta.env.VITE_APP_DOMAIN || ''
```

Set via `VITE_APP_DOMAIN` environment variable or runtime config injection.

---

## Environment Variables

| Variable | Where | Purpose | Example |
|----------|-------|---------|---------|
| `COOKIE_DOMAIN` | API | Scopes auth cookies for cross-subdomain SSO | `.example.com` |
| `CORS_ORIGIN` | API | Primary allowed origin; subdomains auto-allowed | `https://app.example.com` |
| `VITE_APP_DOMAIN` | Frontend | Base domain for constructing app URLs | `example.com` |

---

## Development — Per-User Named Domains

**PR #93** (`feat/dev-named-domains`, commit `f052596`) introduced per-user domain isolation for development environments.

### Problem

Multiple developers running the stack on a shared Traefik instance would conflict on `app.a8n.run` and `api.a8n.run`.

### Solution

The `compose.dev.yml` uses the `${USER}` environment variable to namespace everything:

| Resource | Pattern |
|----------|---------|
| Compose project name | `saas-${USER}` |
| Container names | `saas-api-${USER}`, `saas-frontend-${USER}`, `saas-postgres-${USER}` |
| API URL | `https://${USER}-api.a8n.run` |
| Frontend URL | `https://${USER}-app.a8n.run` |
| Traefik router names | `api-saas-${USER}`, `frontend-saas-${USER}` |
| Docker volumes | `saas-data-${USER}`, `saas-api-target-${USER}` |

Each developer gets their own URLs (e.g., `nate-app.a8n.run`, `nate-api.a8n.run`) while the cookie domain (`.a8n.run`) is shared across all developer instances.

**Traefik routing** accepts both the shared and user-prefixed domains:

```yaml
# API
Host(`api.a8n.run`) || Host(`${USER}-api.a8n.run`)

# Frontend
Host(`app.a8n.run`) || Host(`${USER}-app.a8n.run`)
```

The frontend's Vite dev server allows all `*.a8n.run` subdomains via `allowedHosts: ['.a8n.run']`.
