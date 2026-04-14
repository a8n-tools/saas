//! a8n-api - Main entry point
//!
//! This is the entry point for the backend API server.

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use a8n_api::{
    config::{Config, TierConfig},
    middleware::{
        auto_ban::{self, AutoBanService},
        request_id::RequestIdMiddleware,
        AutoBanMiddleware, SecurityHeaders,
    },
    models::{CreateUser, UserRole},
    repositories::{FeedbackRepository, RateLimitRepository, UserRepository},
    routes,
    services::{AuthService, EmailService, EncryptionKeySet, JwtConfig, JwtService, PasswordService, StripeConfig, StripeService, TotpService, WebhookService},
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

    // Seed default admin if SETUP_DEFAULT_ADMIN is set and no admin exists
    if let Ok(setup_admin) = std::env::var("SETUP_DEFAULT_ADMIN") {
        let admin_emails = UserRepository::find_admin_emails(&pool).await?;
        if admin_emails.is_empty() {
            let (email, password) = setup_admin.split_once(':').unwrap_or_else(|| {
                panic!("SETUP_DEFAULT_ADMIN must be in format 'email:password'");
            });

            let email = email.trim();
            let password = password.trim();

            let password_service = PasswordService::new();
            let password_hash = password_service.hash(password)?;

            let user = UserRepository::create(
                &pool,
                CreateUser {
                    email: email.to_string(),
                    password_hash: Some(password_hash),
                    role: UserRole::Admin,
                },
            )
            .await?;

            info!(email = %user.email, "Default admin user created from SETUP_DEFAULT_ADMIN");
        } else {
            info!("Admin user(s) already exist, skipping SETUP_DEFAULT_ADMIN");
        }
    }

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
    let jwt_config = JwtConfig::from_secret(&jwt_secret, &config.app_name);
    let jwt_service = Arc::new(JwtService::new(jwt_config));

    info!("JWT service initialized");

    // Initialize tier config — prefer DB overrides, fall back to env vars
    let tier_config = {
        use a8n_api::repositories::TierConfigRepository;
        match TierConfigRepository::get(&pool).await {
            Ok(row) if TierConfig::has_db_overrides(&row) => {
                info!("Tier config initialized from database");
                TierConfig::from_db_row(&row)
            }
            _ => {
                info!("Tier config initialized from environment variables");
                config.tier.clone()
            }
        }
    };
    let tier_config = Arc::new(std::sync::RwLock::new(tier_config));

    // Initialize Auth service
    let auth_service = Arc::new(AuthService::new(pool.clone(), (*jwt_service).clone(), tier_config.clone()));

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

    // Build encryption key sets for key rotation support
    let totp_key_set = EncryptionKeySet {
        current: config.totp_encryption_key,
        current_version: config.totp_key_version,
        previous: config.totp_encryption_key_prev,
    };
    let stripe_key_set = EncryptionKeySet {
        current: config.stripe_encryption_key,
        current_version: config.stripe_key_version,
        previous: config.stripe_encryption_key_prev,
    };

    // Initialize Stripe service — prefer DB config (set via admin UI), fall back to env vars
    let stripe_config = {
        use a8n_api::repositories::StripeConfigRepository;
        match StripeConfigRepository::get(&pool).await {
            Ok(db_config) if db_config.secret_key.is_some() => {
                match StripeConfig::from_db_model(&db_config, &stripe_key_set) {
                    Ok(cfg) => {
                        info!("Stripe service initialized from database config");
                        cfg
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "Failed to decrypt DB Stripe config, falling back to env vars");
                        StripeConfig::from_env()?
                    }
                }
            }
            _ => StripeConfig::from_env()?,
        }
    };
    let stripe_service = Arc::new(StripeService::new(stripe_config));

    info!("Stripe service initialized");

    // Initialize TOTP service
    let totp_service = Arc::new(TotpService::new(
        totp_key_set,
        config.app_name.clone(),
        pool.clone(),
    ));

    info!("TOTP service initialized");

    // Initialize webhook service
    let webhook_service = Arc::new(WebhookService::new(jwt_secret.clone()));

    info!("Webhook service initialized");


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

    // Extract the domain from CORS_ORIGIN for subdomain matching
    // e.g. "https://pugtsurani.net" → ".pugtsurani.net"
    let cors_domain = cors_origin
        .split("://")
        .nth(1)
        .unwrap_or("")
        .split('/')
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .map(|host| format!(".{host}"))
        .unwrap_or_default();

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

    // Spawn feedback archive+purge background task (every 24h)
    // Archives closed feedback older than 90 days into feedback_archive, then hard-deletes it
    let feedback_purge_pool = pool.clone();
    tokio::spawn(async move {
        info!("Feedback archive/purge task started");
        let mut interval = tokio::time::interval(Duration::from_secs(86400));
        loop {
            interval.tick().await;
            match FeedbackRepository::archive_and_purge_closed(&feedback_purge_pool).await {
                Ok(purged) => {
                    if purged > 0 {
                        info!(purged, "Archived and purged closed feedback records");
                    }
                }
                Err(e) => {
                    error!(error = %e, "Failed to archive/purge closed feedback");
                }
            }
        }
    });

    info!(address = %server_addr, "Starting HTTP server");

    // Start HTTP server
    HttpServer::new(move || {
        // Configure CORS
        let domain = cors_domain.clone();
        let cors = Cors::default()
            .allowed_origin(&cors_origin)
            .allowed_origin_fn(move |origin, _req_head| {
                let origin = origin.as_bytes();
                // Allow localhost (development)
                if origin.starts_with(b"http://localhost") {
                    return true;
                }
                // Allow the configured domain and its subdomains
                if !domain.is_empty() {
                    return origin.ends_with(domain.as_bytes());
                }
                false
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
            .app_data(web::Data::new(totp_service.clone()))
            .app_data(web::Data::new(webhook_service.clone()))
            .app_data(web::Data::new(stripe_key_set.clone()))
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

/// Initialize tracing subscriber with compact human-readable output
fn init_tracing(log_level: &str) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().compact())
        .init();
}
