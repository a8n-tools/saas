# 12 - Monitoring & Observability

## Overview

This document contains prompts for setting up Prometheus metrics, Grafana dashboards, and GlitchTip error tracking.

## Prerequisites
- Infrastructure setup complete (11-infrastructure.md)
- All services running

---

## Prompt 12.1: Prometheus Metrics in API

```text
Add Prometheus metrics to the Rust API.

Add dependencies to Cargo.toml:
- actix-web-prom = "0.7"
- prometheus = "0.13"

Create src/metrics.rs:
```rust
use prometheus::{
    Counter, CounterVec, Histogram, HistogramVec, Gauge, Opts,
    Registry, TextEncoder, Encoder,
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    // HTTP metrics
    pub static ref HTTP_REQUESTS_TOTAL: CounterVec = CounterVec::new(
        Opts::new("http_requests_total", "Total HTTP requests"),
        &["method", "path", "status"]
    ).unwrap();

    pub static ref HTTP_REQUEST_DURATION: HistogramVec = HistogramVec::new(
        prometheus::HistogramOpts::new(
            "http_request_duration_seconds",
            "HTTP request duration"
        ).buckets(vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
        &["method", "path"]
    ).unwrap();

    // Auth metrics
    pub static ref LOGIN_ATTEMPTS_TOTAL: CounterVec = CounterVec::new(
        Opts::new("login_attempts_total", "Total login attempts"),
        &["result"]  // success, invalid_credentials, rate_limited
    ).unwrap();

    pub static ref ACTIVE_SESSIONS: Gauge = Gauge::new(
        "active_sessions",
        "Number of active refresh tokens"
    ).unwrap();

    // Subscription metrics
    pub static ref ACTIVE_SUBSCRIPTIONS: Gauge = Gauge::new(
        "active_subscriptions",
        "Number of active subscriptions"
    ).unwrap();

    pub static ref SUBSCRIPTION_EVENTS: CounterVec = CounterVec::new(
        Opts::new("subscription_events_total", "Subscription lifecycle events"),
        &["event"]  // created, canceled, reactivated
    ).unwrap();

    // Payment metrics
    pub static ref PAYMENTS_TOTAL: CounterVec = CounterVec::new(
        Opts::new("payments_total", "Total payment attempts"),
        &["status"]  // succeeded, failed
    ).unwrap();

    pub static ref REVENUE_CENTS: Counter = Counter::new(
        "revenue_cents_total",
        "Total revenue in cents"
    ).unwrap();

    // Database metrics
    pub static ref DB_CONNECTIONS_ACTIVE: Gauge = Gauge::new(
        "db_connections_active",
        "Active database connections"
    ).unwrap();

    pub static ref DB_QUERY_DURATION: HistogramVec = HistogramVec::new(
        prometheus::HistogramOpts::new(
            "db_query_duration_seconds",
            "Database query duration"
        ),
        &["query_type"]
    ).unwrap();
}

pub fn register_metrics() {
    REGISTRY.register(Box::new(HTTP_REQUESTS_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(HTTP_REQUEST_DURATION.clone())).unwrap();
    REGISTRY.register(Box::new(LOGIN_ATTEMPTS_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(ACTIVE_SESSIONS.clone())).unwrap();
    REGISTRY.register(Box::new(ACTIVE_SUBSCRIPTIONS.clone())).unwrap();
    REGISTRY.register(Box::new(SUBSCRIPTION_EVENTS.clone())).unwrap();
    REGISTRY.register(Box::new(PAYMENTS_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(REVENUE_CENTS.clone())).unwrap();
    REGISTRY.register(Box::new(DB_CONNECTIONS_ACTIVE.clone())).unwrap();
    REGISTRY.register(Box::new(DB_QUERY_DURATION.clone())).unwrap();
}

pub fn gather_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
```

Add metrics endpoint:
```rust
pub async fn metrics_handler() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(gather_metrics())
}
```

Use middleware to record HTTP metrics automatically.
```

---

## Prompt 12.2: Prometheus Server Configuration

```text
Configure Prometheus server in Docker.

Add to docker-compose.yml:
```yaml
prometheus:
  image: prom/prometheus:v2.48.0
  container_name: a8n-prometheus
  restart: unless-stopped
  volumes:
    - ./prometheus/prometheus.yml:/etc/prometheus/prometheus.yml:ro
    - ./prometheus/rules:/etc/prometheus/rules:ro
    - prometheus-data:/prometheus
  command:
    - '--config.file=/etc/prometheus/prometheus.yml'
    - '--storage.tsdb.path=/prometheus'
    - '--storage.tsdb.retention.time=30d'
    - '--web.enable-lifecycle'
  networks:
    - a8n-network
  labels:
    - "traefik.enable=true"
    - "traefik.http.routers.prometheus.rule=Host(`prometheus.a8n.tools`)"
    - "traefik.http.routers.prometheus.middlewares=auth"
```

Create prometheus/prometheus.yml:
```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

alerting:
  alertmanagers:
    - static_configs:
        - targets: []

rule_files:
  - /etc/prometheus/rules/*.yml

scrape_configs:
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'api'
    static_configs:
      - targets: ['api:8080']
    metrics_path: /metrics

  - job_name: 'traefik'
    static_configs:
      - targets: ['traefik:8082']

  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']

  - job_name: 'node'
    static_configs:
      - targets: ['node-exporter:9100']
```

Create prometheus/rules/alerts.yml:
```yaml
groups:
  - name: a8n-alerts
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.1
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: High error rate detected
          description: "Error rate is {{ $value }} errors/sec"

      - alert: APIDown
        expr: up{job="api"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: API is down

      - alert: HighResponseTime
        expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: High API response time

      - alert: PaymentFailureSpike
        expr: rate(payments_total{status="failed"}[1h]) > rate(payments_total{status="failed"}[24h]) * 2
        for: 15m
        labels:
          severity: warning
        annotations:
          summary: Unusual spike in payment failures

      - alert: LowDiskSpace
        expr: node_filesystem_avail_bytes{mountpoint="/"} / node_filesystem_size_bytes{mountpoint="/"} < 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: Low disk space remaining
```
```

---

## Prompt 12.3: Grafana Dashboard Setup

```text
Configure Grafana with dashboards.

Add to docker-compose.yml:
```yaml
grafana:
  image: grafana/grafana:10.2.0
  container_name: a8n-grafana
  restart: unless-stopped
  environment:
    - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD}
    - GF_USERS_ALLOW_SIGN_UP=false
    - GF_SERVER_ROOT_URL=https://grafana.a8n.tools
  volumes:
    - grafana-data:/var/lib/grafana
    - ./grafana/provisioning:/etc/grafana/provisioning:ro
    - ./grafana/dashboards:/var/lib/grafana/dashboards:ro
  networks:
    - a8n-network
  labels:
    - "traefik.enable=true"
    - "traefik.http.routers.grafana.rule=Host(`grafana.a8n.tools`)"
    - "traefik.http.routers.grafana.tls=true"
```

Create grafana/provisioning/datasources/prometheus.yml:
```yaml
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
```

Create grafana/provisioning/dashboards/dashboards.yml:
```yaml
apiVersion: 1

providers:
  - name: 'default'
    orgId: 1
    folder: ''
    type: file
    disableDeletion: false
    editable: true
    options:
      path: /var/lib/grafana/dashboards
```

Create grafana/dashboards/api-overview.json:
```json
{
  "title": "API Overview",
  "panels": [
    {
      "title": "Request Rate",
      "type": "graph",
      "targets": [
        {
          "expr": "rate(http_requests_total[5m])",
          "legendFormat": "{{method}} {{path}}"
        }
      ]
    },
    {
      "title": "Response Time (p95)",
      "type": "graph",
      "targets": [
        {
          "expr": "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))",
          "legendFormat": "p95"
        }
      ]
    },
    {
      "title": "Error Rate",
      "type": "graph",
      "targets": [
        {
          "expr": "rate(http_requests_total{status=~\"5..\"}[5m])",
          "legendFormat": "5xx errors"
        }
      ]
    },
    {
      "title": "Active Sessions",
      "type": "stat",
      "targets": [
        {
          "expr": "active_sessions"
        }
      ]
    }
  ]
}
```

Create additional dashboards:
- Business metrics (subscriptions, revenue, users)
- Application health (container status, database connections)
- Infrastructure (CPU, memory, disk)
```

---

## Prompt 12.4: GlitchTip Error Tracking

```text
Set up GlitchTip for error tracking.

Add to docker-compose.yml:
```yaml
glitchtip:
  image: glitchtip/glitchtip:latest
  container_name: a8n-glitchtip
  restart: unless-stopped
  environment:
    - DATABASE_URL=postgres://glitchtip:${GLITCHTIP_DB_PASSWORD}@glitchtip-db:5432/glitchtip
    - SECRET_KEY=${GLITCHTIP_SECRET_KEY}
    - EMAIL_URL=smtp://stalwart:587
    - DEFAULT_FROM_EMAIL=glitchtip@a8n.tools
    - GLITCHTIP_DOMAIN=https://glitchtip.a8n.tools
  depends_on:
    - glitchtip-db
  networks:
    - a8n-network
  labels:
    - "traefik.enable=true"
    - "traefik.http.routers.glitchtip.rule=Host(`glitchtip.a8n.tools`)"
    - "traefik.http.routers.glitchtip.tls=true"

glitchtip-db:
  image: postgres:16-alpine
  container_name: a8n-glitchtip-db
  restart: unless-stopped
  environment:
    - POSTGRES_USER=glitchtip
    - POSTGRES_PASSWORD=${GLITCHTIP_DB_PASSWORD}
    - POSTGRES_DB=glitchtip
  volumes:
    - glitchtip-data:/var/lib/postgresql/data
  networks:
    - a8n-network

glitchtip-worker:
  image: glitchtip/glitchtip:latest
  container_name: a8n-glitchtip-worker
  restart: unless-stopped
  command: ./bin/run-celery-with-beat.sh
  environment:
    - DATABASE_URL=postgres://glitchtip:${GLITCHTIP_DB_PASSWORD}@glitchtip-db:5432/glitchtip
    - SECRET_KEY=${GLITCHTIP_SECRET_KEY}
  depends_on:
    - glitchtip-db
    - redis
  networks:
    - a8n-network
```

Add Sentry SDK to API (Cargo.toml):
```toml
sentry = { version = "0.32", features = ["tracing"] }
sentry-actix = "0.32"
```

Configure in main.rs:
```rust
let _guard = sentry::init((
    std::env::var("GLITCHTIP_DSN").ok(),
    sentry::ClientOptions {
        release: sentry::release_name!(),
        environment: Some(std::env::var("ENVIRONMENT").unwrap_or("development".to_string()).into()),
        traces_sample_rate: 0.1,
        ..Default::default()
    },
));

// Add to Actix app
App::new()
    .wrap(sentry_actix::Sentry::new())
```

Add to frontend (package.json):
```json
"@sentry/react": "^7.0.0"
```

Configure in main.tsx:
```typescript
import * as Sentry from "@sentry/react";

Sentry.init({
  dsn: import.meta.env.VITE_GLITCHTIP_DSN,
  environment: import.meta.env.MODE,
  tracesSampleRate: 0.1,
});
```
```

---

## Prompt 12.5: Structured Logging

```text
Implement structured JSON logging for production.

Update logging configuration in main.rs:
```rust
use tracing_subscriber::{
    fmt::{self, format::JsonFields},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

fn init_logging() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let format = if std::env::var("RUST_LOG_FORMAT").unwrap_or_default() == "json" {
        fmt::layer()
            .json()
            .with_current_span(true)
            .with_span_list(true)
            .with_file(true)
            .with_line_number(true)
            .boxed()
    } else {
        fmt::layer()
            .with_target(true)
            .with_file(true)
            .with_line_number(true)
            .boxed()
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(format)
        .init();
}
```

Create log aggregation with Docker logging driver:
```yaml
# In docker-compose.yml, add to services:
x-logging: &logging
  driver: "json-file"
  options:
    max-size: "10m"
    max-file: "3"
    labels: "service"

services:
  api:
    logging: *logging
    labels:
      - service=api
```

Log format example:
```json
{
  "timestamp": "2024-12-30T10:00:00.000Z",
  "level": "INFO",
  "target": "a8n_api::handlers::auth",
  "message": "user logged in",
  "span": {
    "request_id": "req_abc123",
    "user_id": "uuid"
  },
  "file": "src/handlers/auth.rs",
  "line": 42
}
```
```

---

## Validation Checklist

After completing all prompts in this section, verify:

- [ ] Prometheus scrapes all targets
- [ ] Metrics endpoint returns data
- [ ] Grafana dashboards load
- [ ] Alerts trigger correctly
- [ ] GlitchTip receives errors
- [ ] Source maps work in GlitchTip
- [ ] JSON logs in production
- [ ] Log aggregation works

---

## Next Steps

Proceed to **[13-security.md](./13-security.md)** to implement security hardening.
