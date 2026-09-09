# a8n.tools

A SaaS platform hosting developer and productivity tools: a Rust and Actix Web API with a React single-page front end, selling managed hosting for open-source applications behind one subscription and single sign-on across every hosted app.

<!--
BUNYIP-631 tracks the cross-repo README sweep this file belongs to. No
walkthrough GIF has been recorded for this repository yet. When one lands,
commit it at docs/assets/a8n-tools-walkthrough.gif (a cross-repo relative path
does not render on the mirrors, and hot-linking a raw asset URL is fragile) and
replace this comment with:
![a8n.tools walkthrough](docs/assets/a8n-tools-walkthrough.gif)
-->

## Documentation

Everything else is in [`docs/`](docs/README.md), indexed there in full:

- [quickstart.md](docs/quickstart.md) - get a fresh clone running, the task-runner recipes, and the tests
- [configuration.md](docs/configuration.md) - every environment variable, the health-check endpoints, and a deployment compose example
- [ARCHITECTURE.md](docs/ARCHITECTURE.md) - services, auth and subscription flows, data isolation, and security layers
- [architecture-decisions.md](docs/architecture-decisions.md) - why Actix Web and SQLx, the JWT strategy, subdomain routing, and how dev differs from production
- [releasing.md](docs/releasing.md) - how a version bump becomes a tag and a Forgejo release
- [forgejo-download-proxy.md](docs/forgejo-download-proxy.md) and [oci-registry.md](docs/oci-registry.md) - the two gated member-download features

Contributor conventions are in [CONTRIBUTING.md](CONTRIBUTING.md), and conventions for AI agents working in this repository are in [CLAUDE.md](CLAUDE.md).

## Development happens on Forgejo

The development home for this repository is <https://dev.a8n.run/a8n-tools/saas>. The [GitHub](https://github.com/a8n-tools/saas) and [Codeberg](https://codeberg.org/a8n-tools/saas) copies are read-only mirrors that exist for visibility only: issues and pull requests are disabled there, and no community support runs on the mirrors. File issues and open pull requests on Forgejo.

## Security

Please do not report a suspected vulnerability through the public issue tracker, on Forgejo or on either mirror: filing it there publishes it. Contact a maintainer privately instead. A published disclosure address and a `SECURITY.md` are being set up and this section will link to them.

## License

MIT. See [LICENSE](LICENSE).

## Authors and credits

a8n.tools is built by a8n Tools, and `api/Cargo.toml` carries the authoritative author field.

Built on [Rust](https://www.rust-lang.org/), [Actix Web](https://actix.rs/), [SQLx](https://github.com/launchbadge/sqlx) and [PostgreSQL](https://www.postgresql.org/) on the back end, and [React](https://react.dev/), [TypeScript](https://www.typescriptlang.org/), [Vite](https://vite.dev/), [Tailwind CSS](https://tailwindcss.com/) and [shadcn/ui](https://ui.shadcn.com/) on the front end, built and tested with [Bun](https://bun.sh/) and [Vitest](https://vitest.dev/). Driven by [just](https://github.com/casey/just) and [Nushell](https://www.nushell.sh/), containerized with [Docker](https://www.docker.com/), served by [Caddy](https://caddyserver.com/) and deployed behind [Traefik](https://traefik.io/). Billing runs on [Stripe](https://stripe.com/).
