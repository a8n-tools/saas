# Configuration

Every environment variable the platform reads, plus the health-check contract a deployer has to wire up.

`api/src/config.rs` is authoritative. `Config::from_env()` is the single place the API reads its environment, so when this page and that file disagree, the file wins.

## API

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `DATABASE_URL` | PostgreSQL connection string | - | Yes |
| `HOST_IP` | API server bind address | `0.0.0.0` | No |
| `APP_PORT` | API server port | `4000` | No |
| `RUST_LOG` | Log level | `info` | No |
| `ENVIRONMENT` | `production` or `development` | `production` | No |
| `APP_NAME` | App name used in email subjects and templates | `localhost` | No |
| `APP_URL` | Front-end base URL for email links | Falls back to `CORS_ORIGIN`, then `http://localhost:5173` | No |
| `CORS_ORIGIN` | Allowed CORS origin (the front-end URL) | `http://localhost:5173` | No |
| `COOKIE_DOMAIN` | Cookie domain for cross-subdomain auth (for example `.example.com`) | None, meaning the exact hostname | Yes in production |
| `JWT_SECRET` | Shared JWT signing secret | - | Yes in production |
| `TOTP_ENCRYPTION_KEY` | Hex-encoded 32-byte key for encrypting TOTP secrets | Zero bytes in development; startup panics in production if unset | Yes in production |
| `TOTP_ENCRYPTION_KEY_PREV` | Previous TOTP key, read during rotation | - | No |
| `TOTP_KEY_VERSION` | Key version stamped onto newly encrypted TOTP secrets | `1` | No |
| `STRIPE_ENCRYPTION_KEY` | Hex-encoded 32-byte key for encrypting Stripe secrets | Zero bytes in development; startup panics in production if unset | Yes in production |
| `STRIPE_ENCRYPTION_KEY_PREV` | Previous Stripe key, read during rotation | - | No |
| `STRIPE_KEY_VERSION` | Key version stamped onto newly encrypted Stripe secrets | `1` | No |
| `STRIPE_SECRET_KEY` | Stripe API secret key | - | Yes in production |
| `STRIPE_WEBHOOK_SECRET` | Stripe webhook signing secret | - | Yes in production |
| `STRIPE_PRICE_ID` | Stripe price id for the subscription | - | Yes in production |
| `STRIPE_BUSINESS_PRICE_ID` | Stripe price id for the business tier | - | No |
| `STRIPE_FREE_PRICE_ID` | Stripe price id for the free tier | - | No |
| `SMTP_HOST` | SMTP server hostname | `localhost` | No |
| `SMTP_PORT` | SMTP server port | Derived from `SMTP_TLS` | No |
| `SMTP_TLS` | TLS mode for the SMTP connection | Implicit TLS | No |
| `SMTP_FROM` | Sender address, as `Name <email>` or `email` | `noreply@localhost` | No |
| `SMTP_USERNAME` | SMTP auth username | - | No |
| `SMTP_PASSWORD` | SMTP auth password | - | No |
| `EMAIL_ENABLED` | Force email sending on in development | `false` | No |
| `ADMIN_NOTIFICATION_EMAILS` | Comma-separated recipients for admin notifications | Empty | No |

Generate the two encryption keys with `openssl rand -hex 32`. Rotating them is covered in [encryption-key-rotation.md](encryption-key-rotation.md).

### Membership tiers

| Variable | Description | Default |
|----------|-------------|---------|
| `TIER_LIFETIME_SLOTS` | Lifetime-tier slots available | `5` |
| `TIER_EARLY_ADOPTER_SLOTS` | Early-adopter slots available | `5` |
| `TIER_EARLY_ADOPTER_TRIAL_DAYS` | Trial length for the early-adopter tier | `90` |
| `TIER_STANDARD_TRIAL_DAYS` | Trial length for the standard tier | `30` |

Values stored in the database override these; the environment supplies the fallback.

### Automatic IP banning

| Variable | Description | Default |
|----------|-------------|---------|
| `AUTO_BAN_ENABLED` | Turn automatic banning on or off | `true` |
| `AUTO_BAN_THRESHOLD` | Failures inside the window before a ban | `5` |
| `AUTO_BAN_WINDOW_SECS` | Length of the counting window | `3600` |
| `AUTO_BAN_DURATION_SECS` | How long a ban lasts | `86400` |

### OIDC provider

| Variable | Description | Default |
|----------|-------------|---------|
| `OIDC_ISSUER` | Issuer URL. Unset or empty disables the provider. | - |
| `OIDC_JWT_PRIVATE_KEY_PATH` | Signing key path | `secrets/jwt_private.pem` |
| `OIDC_JWT_ACTIVE_KID` | Key id stamped into issued tokens | `dev-key` |
| `OIDC_JWT_PUBLIC_KEYS_DIR` | Directory of published public keys | `secrets` |
| `OIDC_ACCESS_TOKEN_TTL_SECONDS` | Access token lifetime | `600` |
| `OIDC_REFRESH_TOKEN_TTL_SECONDS` | Absolute refresh token lifetime | `2592000` |
| `OIDC_REFRESH_IDLE_TTL_SECONDS` | Idle refresh token lifetime | `1209600` |
| `OIDC_CODE_TTL_SECONDS` | Authorization code lifetime | `60` |

### Forgejo download proxy

Gated behind `FORGEJO_BASE_URL` and `FORGEJO_API_TOKEN`: with either unset the download routes and their UI are hidden. Full documentation is in [forgejo-download-proxy.md](forgejo-download-proxy.md).

| Variable | Description | Default |
|----------|-------------|---------|
| `FORGEJO_BASE_URL` | Forgejo instance root, for example `https://git.example.com` | - |
| `FORGEJO_API_TOKEN` | Forgejo token with read access to the release repos | - |
| `DOWNLOAD_CACHE_DIR` | On-disk SHA-256 blob cache directory | `/var/cache/a8n-downloads` |
| `DOWNLOAD_CACHE_MAX_BYTES` | Soft cap on the blob cache; exceeding it triggers async LRU eviction | `10737418240` (10 GiB) |
| `DOWNLOAD_CONCURRENCY_PER_USER` | Simultaneous in-flight downloads per user, counted in-process and therefore single-instance only | `2` |
| `DOWNLOAD_DAILY_LIMIT_PER_USER` | Downloads per UTC day per user | `50` |
| `FORGEJO_RELEASE_CACHE_TTL_SECS` | TTL for cached Forgejo release metadata | `300` |

### OCI registry

Gated behind `OCI_REGISTRY_ENABLED` plus the two Forgejo variables above. Full documentation is in [oci-registry.md](oci-registry.md).

| Variable | Description | Default |
|----------|-------------|---------|
| `OCI_REGISTRY_ENABLED` | Start the second HTTP server exposing the read-only registry | `false` |
| `OCI_REGISTRY_PORT` | Port for that second server | `18081` |
| `OCI_REGISTRY_SERVICE` | Registry service name presented to clients | `oci.example.com` |
| `OCI_BLOB_CACHE_DIR` | On-disk blob cache directory | `/var/cache/a8n-oci` |
| `OCI_BLOB_CACHE_MAX_BYTES` | Soft cap on the blob cache | `53687091200` (50 GiB) |
| `OCI_MANIFEST_CACHE_TTL_SECS` | TTL for cached manifests | `300` |
| `OCI_CONCURRENT_MANIFESTS_PER_USER` | Simultaneous manifest fetches per user | `2` |
| `OCI_PULLS_PER_USER_PER_DAY` | Pulls per user per day | `50` |
| `OCI_TOKEN_TTL_SECS` | Lifetime of an issued registry token | `900` |

## Front end

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `VITE_API_URL` | API base URL | `http://localhost:18080` | Yes in production |
| `VITE_APP_DOMAIN` | Application domain | `localhost` | No |
| `VITE_SHOW_BUSINESS_PRICING` | Show the business pricing tier | `false` | No |
| `VITE_STRIPE_PUBLISHABLE_KEY` | Stripe publishable key | - | Yes in production |

Front-end variables are injected at runtime through a Caddy template endpoint on an obfuscated path, not baked into the build. The same image can therefore be deployed to different environments by changing container environment variables.

## Health checks

Both the API and front-end images expose a `/health` endpoint but deliberately carry no `HEALTHCHECK` instruction. Configuring health checks is the deployer's job, in their compose file or orchestrator.

| Service | Endpoint | Port | Healthy response |
|---------|----------|------|------------------|
| `api` | `/health` | `APP_PORT`, default `4000` | `200 OK` |
| `frontend` | `/health` | `PORT`, default `8080` | `200 OK`, body `healthy` |

The two images differ. [`oci-build/api/Dockerfile`](../oci-build/api/Dockerfile) sets no port variable and declares `EXPOSE 4000`, matching the `APP_PORT` default. [`oci-build/frontend/Dockerfile`](../oci-build/frontend/Dockerfile) sets `PORT=8080` and declares `EXPOSE 8080`. A deployment therefore reaches the API on `4000` and the front end on `8080` unless it sets `APP_PORT`.

The reference deployment in [`examples/compose.yml`](../examples/compose.yml) sets `HOST` and `PORT` on the API service. The API reads `HOST_IP` and `APP_PORT`, so those two values have no effect and the API listens on `0.0.0.0:4000` behind an `expose: 8080` that does not match it.

### Compose example

```yaml
services:
  api:
    image: your-registry/saas-api:latest
    healthcheck:
      test: ["CMD", "wget", "-q", "--spider", "http://localhost:4000/health"]
      interval: 30s
      timeout: 3s
      start_period: 5s
      retries: 3

  frontend:
    image: your-registry/saas-frontend:latest
    healthcheck:
      test: ["CMD", "wget", "-q", "--spider", "http://localhost:8080/health"]
      interval: 30s
      timeout: 3s
      start_period: 5s
      retries: 3
```
