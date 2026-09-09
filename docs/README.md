# a8n.tools documentation

Every page in this directory, grouped by what you are trying to do.

## Getting started

- [quickstart.md](quickstart.md) - prerequisites, the two compose files, the task-runner recipes, tests, and the repository layout
- [configuration.md](configuration.md) - every environment variable, the health-check contract, and a deployment compose example

## Architecture

- [ARCHITECTURE.md](ARCHITECTURE.md) - service responsibilities, auth and subscription flows, data isolation, security layers, and scaling
- [architecture-decisions.md](architecture-decisions.md) - why Actix Web and SQLx, the JWT strategy, subdomain routing, and how development differs from production
- [a8n-tools-specification.md](a8n-tools-specification.md) - the full product and technical specification

## Features

- [forgejo-download-proxy.md](forgejo-download-proxy.md) - member downloads of release binaries proxied from private Forgejo repositories
- [oci-registry.md](oci-registry.md) - the read-only OCI registry members can `docker pull` from
- [stripe-setup.md](stripe-setup.md) - wiring up the Stripe account, products, and webhooks
- [user-domains.md](user-domains.md) - custom domains for hosted applications

## Operations

- [releasing.md](releasing.md) - how a version bump becomes a tag and a Forgejo release
- [encryption-key-rotation.md](encryption-key-rotation.md) - rotating `TOTP_ENCRYPTION_KEY` and `STRIPE_ENCRYPTION_KEY`

## Build notes

The numbered `00` through `14` pages are the original build-out notes, written as prompts and blueprints while the platform was scaffolded. They record intent and design at the time of writing and are not maintained against the current code, so treat the pages above and the code itself as authoritative where they disagree.

- [00-project-overview.md](00-project-overview.md)
- [01-project-setup.md](01-project-setup.md)
- [02-database-schema.md](02-database-schema.md)
- [03-authentication.md](03-authentication.md)
- [04-api-core.md](04-api-core.md)
- [05-stripe-integration.md](05-stripe-integration.md)
- [06-frontend-foundation.md](06-frontend-foundation.md)
- [07-frontend-auth.md](07-frontend-auth.md)
- [08-frontend-dashboard.md](08-frontend-dashboard.md)
- [09-admin-panel.md](09-admin-panel.md)
- [10-email-system.md](10-email-system.md)
- [11-infrastructure.md](11-infrastructure.md)
- [12-monitoring.md](12-monitoring.md)
- [13-security.md](13-security.md)
- [14-testing-strategy.md](14-testing-strategy.md)

## Design records

Per-feature designs and their implementation plans live under [`superpowers/`](superpowers/):

- [specs/2026-03-31-automated-releases-design.md](superpowers/specs/2026-03-31-automated-releases-design.md) and [plans/2026-03-31-automated-releases.md](superpowers/plans/2026-03-31-automated-releases.md)
- [specs/2026-04-15-forgejo-download-proxy-design.md](superpowers/specs/2026-04-15-forgejo-download-proxy-design.md) and [plans/2026-04-15-forgejo-download-proxy.md](superpowers/plans/2026-04-15-forgejo-download-proxy.md)
- [specs/2026-04-16-oci-registry-design.md](superpowers/specs/2026-04-16-oci-registry-design.md) and [plans/2026-04-16-oci-registry.md](superpowers/plans/2026-04-16-oci-registry.md)
