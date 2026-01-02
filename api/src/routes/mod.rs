//! Route configuration for the API
//!
//! This module organizes all API routes and their handlers.

pub mod admin;
pub mod application;
pub mod auth;
pub mod health;
pub mod subscription;
pub mod user;
pub mod webhook;

use actix_web::web;

/// Configure all application routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    // V1 API routes
    cfg.service(
        web::scope("/v1")
            .configure(health::configure)
            .configure(auth::configure)
            .configure(user::configure)
            .configure(application::configure)
            .configure(subscription::configure)
            .configure(webhook::configure)
            .configure(admin::configure),
    );

    // Health check at root level too
    cfg.service(health::health_check);
}
