//! Subscription routes

use actix_web::web;

use crate::handlers;

/// Configure subscription routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/subscriptions")
            .route("/me", web::get().to(handlers::get_subscription))
            .route("/checkout", web::post().to(handlers::create_checkout))
            .route("/cancel", web::post().to(handlers::cancel_subscription))
            .route("/reactivate", web::post().to(handlers::reactivate_subscription))
            .route("/billing-portal", web::post().to(handlers::billing_portal))
            .route("/payments", web::get().to(handlers::get_payment_history)),
    );
}
