//! Route configuration for the API
//!
//! This module organizes all API routes and their handlers.

pub mod health;

use actix_web::web;

/// Configure all application routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1")
            .configure(health::configure)
    );

    // Health check at root level too
    cfg.service(health::health_check);
}
