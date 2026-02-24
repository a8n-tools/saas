# 04 - Core API Endpoints

## Overview

This document contains prompts for implementing core API functionality including applications listing, health checks, and request validation.

## Prerequisites
- Completed 01-03 documents
- Authentication system working

---

## Prompt 4.1: Request Validation Infrastructure

```text
Create a comprehensive request validation system.

Add dependency:
- validator = { version = "0.16", features = ["derive"] }

Create src/validation/mod.rs:

1. Custom validators:
   ```rust
   pub fn validate_email(email: &str) -> Result<(), ValidationError>;
   pub fn validate_password_strength(password: &str) -> Result<(), ValidationError>;
   pub fn validate_uuid(id: &str) -> Result<(), ValidationError>;
   pub fn validate_slug(slug: &str) -> Result<(), ValidationError>;
   ```

2. Validation error conversion:
   ```rust
   impl From<validator::ValidationErrors> for AppError {
       fn from(errors: validator::ValidationErrors) -> Self {
           // Convert to VALIDATION_ERROR with field details
       }
   }
   ```

3. Create a validated JSON extractor:
   ```rust
   pub struct ValidatedJson<T>(pub T);

   impl<T: DeserializeOwned + Validate> FromRequest for ValidatedJson<T> {
       // Parse JSON
       // Run validation
       // Return ValidationError if invalid
   }
   ```

4. Create validated query extractor:
   ```rust
   pub struct ValidatedQuery<T>(pub T);
   ```

5. Common validation rules struct:
   ```rust
   pub struct ValidationRules;

   impl ValidationRules {
       pub const EMAIL_MAX_LENGTH: usize = 255;
       pub const PASSWORD_MIN_LENGTH: usize = 12;
       pub const PASSWORD_MAX_LENGTH: usize = 128;
       pub const SLUG_PATTERN: &'static str = r"^[a-z0-9-]+$";
   }
   ```

Write unit tests for each validator.
```

---

## Prompt 4.2: Applications API

```text
Implement the applications listing endpoint.

Create src/handlers/application.rs:

1. GET /v1/applications
   ```rust
   pub async fn list_applications(
       user: OptionalUser,
       pool: web::Data<PgPool>,
   ) -> Result<HttpResponse, AppError>;
   ```
   - Return all active applications
   - If user is authenticated, include access status
   - If user has active subscription, show full details
   - Response format:
     ```json
     {
       "success": true,
       "data": {
         "applications": [
           {
             "id": "uuid",
             "slug": "rus",
             "display_name": "RUS - URL Shortener",
             "description": "...",
             "icon_url": "...",
             "version": "1.0.0",
             "source_code_url": "...",
             "is_accessible": true,
             "maintenance_mode": false,
             "maintenance_message": null
           }
         ]
       }
     }
     ```

2. GET /v1/applications/:slug
   ```rust
   pub async fn get_application(
       user: OptionalUser,
       slug: web::Path<String>,
       pool: web::Data<PgPool>,
   ) -> Result<HttpResponse, AppError>;
   ```
   - Return single application details
   - 404 if not found or not active

3. ApplicationResponse struct:
   ```rust
   #[derive(Serialize)]
   pub struct ApplicationResponse {
       pub id: Uuid,
       pub slug: String,
       pub display_name: String,
       pub description: Option<String>,
       pub icon_url: Option<String>,
       pub version: Option<String>,
       pub source_code_url: Option<String>,
       pub is_accessible: bool,
       pub maintenance_mode: bool,
       pub maintenance_message: Option<String>,
   }
   ```

Create src/routes/application.rs and integrate.

Write integration tests.
```

---

## Prompt 4.3: Health Check Endpoints

```text
Implement comprehensive health check endpoints.

Create src/handlers/health.rs:

1. GET /health (simple liveness):
   ```rust
   pub async fn health() -> HttpResponse {
       HttpResponse::Ok().json(json!({"status": "ok"}))
   }
   ```

2. GET /health/ready (readiness with dependencies):
   ```rust
   #[derive(Serialize)]
   pub struct ReadinessResponse {
       pub status: String,
       pub checks: HashMap<String, CheckResult>,
       pub version: String,
       pub uptime_seconds: u64,
   }

   #[derive(Serialize)]
   pub struct CheckResult {
       pub status: String,  // "ok" | "degraded" | "error"
       pub latency_ms: Option<u64>,
       pub message: Option<String>,
   }

   pub async fn readiness(
       pool: web::Data<PgPool>,
       start_time: web::Data<Instant>,
   ) -> Result<HttpResponse, AppError>;
   ```
   - Check database connectivity
   - Check Redis if configured
   - Return 200 if all OK, 503 if any failed

3. GET /health/detailed (admin only):
   ```rust
   pub async fn detailed_health(
       user: AuthenticatedUser,
       pool: web::Data<PgPool>,
   ) -> Result<HttpResponse, AppError>;
   ```
   - Require admin role
   - Include database pool stats
   - Include memory usage
   - Include active connections

4. Store start time in app data for uptime calculation.

Create src/routes/health.rs.

Write tests for each endpoint.
```

---

## Prompt 4.4: Request Logging Middleware

```text
Create comprehensive request logging middleware.

Create src/middleware/logging.rs:

1. RequestLogger middleware:
   ```rust
   pub struct RequestLogger;
   ```
   Logs for each request:
   - Method
   - Path
   - Query string (sanitized)
   - Request ID
   - User ID (if authenticated)
   - Response status
   - Duration in ms
   - Response body size

2. Use tracing spans:
   ```rust
   let span = tracing::info_span!(
       "http_request",
       method = %req.method(),
       path = %req.path(),
       request_id = %request_id,
       user_id = tracing::field::Empty,
   );
   ```

3. Sensitive data filtering:
   - Never log passwords
   - Never log tokens
   - Mask email addresses in logs
   - Mask credit card numbers

4. Different log levels:
   - 2xx: INFO
   - 4xx: WARN
   - 5xx: ERROR

5. Structured JSON output for production:
   ```json
   {
     "timestamp": "2024-12-30T10:00:00Z",
     "level": "INFO",
     "target": "a8n_api::middleware::logging",
     "message": "request completed",
     "method": "POST",
     "path": "/v1/auth/login",
     "status": 200,
     "duration_ms": 45,
     "request_id": "req_abc123"
   }
   ```

Add to middleware chain in main.rs.

Write tests verifying log output format.
```

---

## Prompt 4.5: CORS Configuration

```text
Implement proper CORS configuration for the platform.

Update CORS setup in main.rs:

1. CorsConfig struct:
   ```rust
   pub struct CorsConfig {
       pub allowed_origins: Vec<String>,
       pub allowed_methods: Vec<Method>,
       pub allowed_headers: Vec<HeaderName>,
       pub expose_headers: Vec<HeaderName>,
       pub max_age: usize,
       pub supports_credentials: bool,
   }
   ```

2. Default configuration:
   ```rust
   impl Default for CorsConfig {
       fn default() -> Self {
           Self {
               allowed_origins: vec![
                   "https://example.com".to_string(),
                   "https://app.example.com".to_string(),
                   "https://admin.example.com".to_string(),
               ],
               allowed_methods: vec![
                   Method::GET,
                   Method::POST,
                   Method::PUT,
                   Method::DELETE,
                   Method::OPTIONS,
               ],
               allowed_headers: vec![
                   header::CONTENT_TYPE,
                   header::AUTHORIZATION,
                   header::ACCEPT,
                   HeaderName::from_static("x-request-id"),
               ],
               expose_headers: vec![
                   HeaderName::from_static("x-request-id"),
               ],
               max_age: 3600,
               supports_credentials: true,
           }
       }
   }
   ```

3. Development override:
   - Add localhost origins in development mode
   - Read from CORS_ORIGINS environment variable

4. Apply with actix-cors:
   ```rust
   Cors::default()
       .allowed_origin_fn(|origin, _req_head| {
           // Check against config
       })
       .allowed_methods(config.allowed_methods)
       .allowed_headers(config.allowed_headers)
       .expose_headers(config.expose_headers)
       .max_age(config.max_age)
       .supports_credentials()
   ```

Test CORS preflight requests work correctly.
```

---

## Prompt 4.6: Request ID Propagation

```text
Implement request ID generation and propagation.

Create src/middleware/request_id.rs:

1. RequestId middleware:
   - Check for incoming X-Request-ID header
   - If not present, generate new UUID with "req_" prefix
   - Store in request extensions
   - Add to response headers

2. RequestId extractor:
   ```rust
   pub struct RequestId(pub String);

   impl FromRequest for RequestId {
       // Extract from extensions
   }
   ```

3. Update all handlers to use RequestId:
   - Pass to service layer
   - Include in error responses
   - Include in audit logs

4. Configure tracing to include request_id:
   ```rust
   tracing_subscriber::fmt()
       .json()
       .with_current_span(true)
       .init();
   ```

5. Propagate to external services:
   - Add X-Request-ID when calling app health checks
   - Add to outgoing webhook calls

Update response helpers to require request_id parameter.

Write tests verifying:
- ID generated when not provided
- ID preserved when provided
- ID appears in response header
- ID appears in logs
```

---

## Prompt 4.7: API Versioning

```text
Implement API versioning strategy.

Create src/routes/mod.rs:

1. Version-scoped route configuration:
   ```rust
   pub fn configure_v1(cfg: &mut web::ServiceConfig) {
       cfg.service(
           web::scope("/v1")
               .configure(auth::configure)
               .configure(user::configure)
               .configure(subscription::configure)
               .configure(application::configure)
               .configure(admin::configure)
       );
   }
   ```

2. Root routes (unversioned):
   ```rust
   pub fn configure_root(cfg: &mut web::ServiceConfig) {
       cfg.service(
           web::resource("/health").route(web::get().to(health::health))
       );
       cfg.service(
           web::resource("/health/ready").route(web::get().to(health::readiness))
       );
   }
   ```

3. Update main.rs:
   ```rust
   App::new()
       .configure(routes::configure_root)
       .configure(routes::configure_v1)
   ```

4. Create version header middleware (optional):
   - Add X-API-Version: v1 to all responses under /v1

5. Document versioning strategy:
   - Breaking changes require new version
   - Old versions supported for 6 months after deprecation
   - Deprecation warnings in response headers

Test all routes are correctly mounted.
```

---

## Prompt 4.8: Graceful Shutdown

```text
Implement graceful shutdown handling.

Update main.rs:

1. Create shutdown signal handler:
   ```rust
   async fn shutdown_signal() {
       let ctrl_c = async {
           tokio::signal::ctrl_c()
               .await
               .expect("failed to install Ctrl+C handler");
       };

       #[cfg(unix)]
       let terminate = async {
           tokio::signal::unix::signal(
               tokio::signal::unix::SignalKind::terminate()
           )
           .expect("failed to install signal handler")
           .recv()
           .await;
       };

       #[cfg(not(unix))]
       let terminate = std::future::pending::<()>();

       tokio::select! {
           _ = ctrl_c => {},
           _ = terminate => {},
       }

       tracing::info!("shutdown signal received");
   }
   ```

2. Configure server with shutdown:
   ```rust
   HttpServer::new(|| App::new())
       .bind("0.0.0.0:8080")?
       .shutdown_timeout(30)  // 30 seconds to finish requests
       .run()
       .await
   ```

3. Cleanup tasks on shutdown:
   - Close database connections gracefully
   - Flush any pending logs
   - Complete in-flight requests

4. Health check updates during shutdown:
   - Return 503 on /health/ready during shutdown
   - Allow /health to continue returning 200

5. Add startup logging:
   ```rust
   tracing::info!(
       version = env!("CARGO_PKG_VERSION"),
       host = %config.host,
       port = %config.port,
       "server starting"
   );
   ```

Test graceful shutdown with Docker stop.
```

---

## Validation Checklist

After completing all prompts in this section, verify:

- [ ] Request validation returns proper error format
- [ ] Applications endpoint returns correct data
- [ ] Health endpoints work correctly
- [ ] Request logging shows all expected fields
- [ ] CORS allows frontend origins
- [ ] Request IDs propagate through entire request
- [ ] All routes under /v1 prefix
- [ ] Graceful shutdown completes in-flight requests
- [ ] Structured JSON logs in production

---

## Next Steps

Proceed to **[05-stripe-integration.md](./05-stripe-integration.md)** to implement payment processing.
