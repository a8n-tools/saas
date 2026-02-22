//! a8n-api - Main entry point
//!
//! This is the entry point for the a8n.tools backend API server.

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use a8n_api::{
    config::Config,
    middleware::{
        auto_ban::{self, AutoBanService},
        request_id::RequestIdMiddleware,
        AutoBanMiddleware, SecurityHeaders,
    },
    repositories::RateLimitRepository,
    routes,
    services::{AuthService, EmailService, JwtConfig, JwtService, StripeConfig, StripeService},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = Config::from_env()?;

    // Initialize tracing/logging
    init_tracing(&config.log_level);

    info!(
        version = env!("CARGO_PKG_VERSION"),
        environment = %config.environment,
        "Starting a8n-api"
    );

    // Create database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&config.database_url)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to connect to database");
            e
        })?;

    info!("Database connection pool established");

    // Run database migrations
    info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to run database migrations");
            e
        })?;

    info!("Database migrations completed successfully");

    // Test database connection
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|e| {
            error!(error = %e, "Database health check failed");
            e
        })?;

    info!("Database health check passed");

    // Initialize JWT service
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        if config.is_production() {
            panic!("JWT_SECRET must be set in production");
        }
        "development-secret-key-min-32-chars-long!".to_string()
    });
    let jwt_config = JwtConfig::from_secret(&jwt_secret);
    let jwt_service = Arc::new(JwtService::new(jwt_config));

    info!("JWT service initialized");

    // Initialize Auth service
    let auth_service = Arc::new(AuthService::new(pool.clone(), (*jwt_service).clone()));

    info!("Auth service initialized");

    // Initialize Email service
    let email_service = Arc::new(
        EmailService::new(config.email.clone())
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "Failed to initialize email service, using dev mode");
                EmailService::new_dev()
            })
    );

    info!(enabled = config.email.enabled, "Email service initialized");

    // Initialize Stripe service
    let stripe_config = StripeConfig::from_env()?;
    let stripe_service = Arc::new(StripeService::new(stripe_config));

    info!("Stripe service initialized");

    // Initialize auto-ban service
    let auto_ban_service = Arc::new(AutoBanService::new(config.auto_ban.clone(), pool.clone()));

    // Load existing bans from DB
    match auto_ban::load_active_bans(&pool).await {
        Ok(bans) => {
            auto_ban_service.load_bans(bans).await;
        }
        Err(e) => {
            error!(error = %e, "Failed to load IP bans from database");
        }
    }

    info!(
        enabled = config.auto_ban.enabled,
        threshold = config.auto_ban.threshold,
        "Auto-ban service initialized"
    );

    let server_addr = config.server_addr();
    let cors_origin = config.cors_origin.clone();
    let config_data = config.clone();

    // Spawn rate limit cleanup background task
    let cleanup_pool = pool.clone();
    tokio::spawn(async move {
        info!("Rate limit cleanup task started");
        let mut interval = tokio::time::interval(Duration::from_secs(3600));
        loop {
            interval.tick().await;
            match RateLimitRepository::cleanup_expired(&cleanup_pool).await {
                Ok(deleted) => {
                    if deleted > 0 {
                        info!(deleted, "Cleaned up expired rate limit entries");
                    }
                }
                Err(e) => {
                    error!(error = %e, "Failed to cleanup expired rate limit entries");
                }
            }
        }
    });

    // Spawn auto-ban cleanup background task (every 5 minutes)
    let ban_cleanup_pool = pool.clone();
    let ban_cleanup_service = auto_ban_service.clone();
    tokio::spawn(async move {
        info!("Auto-ban cleanup task started");
        let mut interval = tokio::time::interval(Duration::from_secs(300));
        loop {
            interval.tick().await;
            // Clean in-memory state
            ban_cleanup_service.cleanup_expired().await;
            // Clean database
            match auto_ban::cleanup_expired_bans(&ban_cleanup_pool).await {
                Ok(deleted) => {
                    if deleted > 0 {
                        info!(deleted, "Cleaned up expired IP bans");
                    }
                }
                Err(e) => {
                    error!(error = %e, "Failed to cleanup expired IP bans");
                }
            }
        }
    });

    info!(address = %server_addr, "Starting HTTP server");

    // Start HTTP server
    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allowed_origin(&cors_origin)
            .allowed_origin_fn(|origin, _req_head| {
                // Allow all subdomains of a8n.tools and a8n.test (dev)
                origin
                    .as_bytes()
                    .ends_with(b".a8n.tools")
                    || origin.as_bytes() == b"https://a8n.tools"
                    || origin.as_bytes().ends_with(b".a8n.test")
                    || origin.as_bytes() == b"http://a8n.test"
                    || origin.as_bytes().starts_with(b"http://localhost")
            })
            .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::CONTENT_TYPE,
                actix_web::http::header::COOKIE,
            ])
            .expose_headers(vec![
                actix_web::http::header::SET_COOKIE,
            ])
            .supports_credentials()
            .max_age(3600);

        App::new()
            // Add middleware (order matters - executed in reverse order)
            .wrap(TracingLogger::default())
            .wrap(Logger::default())
            .wrap(SecurityHeaders)
            .wrap(RequestIdMiddleware)
            .wrap(cors)
            // Auto-ban runs outermost — rejects banned IPs before CORS processing
            .wrap(AutoBanMiddleware::new(auto_ban_service.clone()))
            // Explicit JSON body size limit (32 KB)
            .app_data(web::JsonConfig::default().limit(32_768))
            // Add database pool to app state
            .app_data(web::Data::new(pool.clone()))
            // Add services to app state
            .app_data(jwt_service.clone())
            .app_data(web::Data::new(auth_service.clone()))
            .app_data(web::Data::new(email_service.clone()))
            .app_data(web::Data::new(stripe_service.clone()))
            .app_data(web::Data::new(config_data.clone()))
            // Configure routes
            .configure(routes::configure)
    })
    .bind(&server_addr)?
    .shutdown_timeout(30)
    .run()
    .await?;

    Ok(())
}

/// Initialize tracing subscriber with JSON output for production
fn init_tracing(log_level: &str) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}
