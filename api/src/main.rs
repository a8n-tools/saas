//! a8n-api - Main entry point
//!
//! This is the entry point for the a8n.tools backend API server.

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tracing::{error, info};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use a8n_api::{config::Config, middleware::request_id::RequestIdMiddleware, routes};

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

    // Test database connection
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|e| {
            error!(error = %e, "Database health check failed");
            e
        })?;

    info!("Database health check passed");

    let server_addr = config.server_addr();
    let cors_origin = config.cors_origin.clone();

    info!(address = %server_addr, "Starting HTTP server");

    // Start HTTP server
    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allowed_origin(&cors_origin)
            .allowed_origin_fn(|origin, _req_head| {
                // Allow all subdomains of a8n.tools
                origin
                    .as_bytes()
                    .ends_with(b".a8n.tools")
                    || origin.as_bytes() == b"https://a8n.tools"
            })
            .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::CONTENT_TYPE,
            ])
            .supports_credentials()
            .max_age(3600);

        App::new()
            // Add middleware
            .wrap(TracingLogger::default())
            .wrap(Logger::default())
            .wrap(RequestIdMiddleware)
            .wrap(cors)
            // Add database pool to app state
            .app_data(web::Data::new(pool.clone()))
            // Configure routes
            .configure(routes::configure)
    })
    .bind(&server_addr)?
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
