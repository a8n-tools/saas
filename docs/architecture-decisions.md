# Architecture decisions

Why the platform is built the way it is. The shape of the running system, the auth and subscription flows, and the data-isolation model are in [ARCHITECTURE.md](ARCHITECTURE.md); this page is the reasoning behind the choices.

## Why Actix Web

- Strong performance and mature async support
- A broad ecosystem for web services
- Type-safe request handling, which the extractor-based auth model leans on directly
- Battle-tested in production

## Why SQLx over an ORM

- Compile-time query verification, with the checked queries committed under `api/.sqlx/`
- No runtime ORM overhead
- Direct SQL with type safety
- Async-first design

## JWT strategy

- **Algorithm**: HS256 over the shared `JWT_SECRET` (`api/src/services/jwt.rs`). A symmetric algorithm is what lets a child application validate a platform token without a key-distribution step; the cost is that every child app holds a secret that can also mint tokens.
- **Access token**: 15 minutes, short-lived by design
- **Refresh token**: 30 days, stored in the database so it can be revoked
- **Transport**: HTTP-only cookies, set and cleared through the `AuthCookies` helper in `api/src/middleware/auth.rs`. The API reads the `access_token` cookie first and falls back to an `Authorization: Bearer` header.
- **Cookie domain**: set through `COOKIE_DOMAIN`, which is what makes single sign-on work across subdomains

Child applications validate the JWT locally using the same `JWT_SECRET` as the platform API. A child app deployed with a different secret cannot participate in single sign-on.

The OIDC provider is a separate surface with its own key material and lifetimes, configured through the `OIDC_*` variables in [configuration.md](configuration.md#oidc-provider). It signs with EdDSA (Ed25519) from a PEM private key rather than the shared secret, and issues RFC 9068 `at+jwt` access tokens.

## Subdomain routing

Traefik routes by hostname. In production, on the platform's own domain:

- the apex domain serves the marketing site
- `app.` serves the user dashboard
- `api.` serves the backend API
- `admin.` serves the admin panel
- every other subdomain serves an individual hosted application

Development reuses the same shape rather than simulating it: `compose.dev.yml` publishes each developer's stack at `${USER}-app.a8n.run` and `${USER}-api.a8n.run` on the shared `network-traefik-public` Traefik network, with real DNS and real certificates. That is what makes cross-subdomain cookie behavior testable at all, since `localhost` cannot carry a shared cookie domain.

## Development versus production

| Concern | Development | Production |
|---------|-------------|------------|
| Routing | `compose.dev.yml` behind the shared Traefik network at `${USER}-app.a8n.run`, or `compose.yml` on plain localhost ports | Real DNS records per subdomain |
| TLS | Cloudflare-resolved certificates under `compose.dev.yml`; none under `compose.yml` | Let's Encrypt via Traefik |
| Cookie domain | `.a8n.run` under `compose.dev.yml`; unset, meaning the exact hostname, under `compose.yml` | Set through `COOKIE_DOMAIN` |
| Cookie `Secure` flag | `false` | `true`, when `ENVIRONMENT=production` |
| CORS | The developer's own front-end URL, through `CORS_ORIGIN` | The deployed front-end URL, through `CORS_ORIGIN` |
| Front end | Vite dev server with HMR | Static files served by Caddy from the `oci-build` image |
| Email links | Derived from `APP_URL`, falling back to `CORS_ORIGIN` | Derived from `APP_URL`, falling back to `CORS_ORIGIN` |
| Encryption keys | Zero bytes if unset | `TOTP_ENCRYPTION_KEY` and `STRIPE_ENCRYPTION_KEY` are mandatory; startup panics without them |
