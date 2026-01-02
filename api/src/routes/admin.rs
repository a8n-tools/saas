//! Admin routes

use actix_web::web;

use crate::handlers;

/// Configure admin routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            // Dashboard stats
            .route("/stats", web::get().to(handlers::get_dashboard_stats))
            // User management
            .route("/users", web::get().to(handlers::list_users))
            .route("/users/{user_id}", web::get().to(handlers::get_user))
            .route("/users/{user_id}/status", web::put().to(handlers::update_user_status))
            // Subscription management
            .route("/subscriptions", web::get().to(handlers::list_subscriptions))
            .route("/subscriptions/grant", web::post().to(handlers::grant_subscription))
            .route("/subscriptions/revoke", web::post().to(handlers::revoke_subscription))
            // Application management
            .route("/applications", web::get().to(handlers::list_all_applications))
            .route("/applications/{app_id}", web::put().to(handlers::update_application))
            // Audit logs
            .route("/audit-logs", web::get().to(handlers::list_audit_logs)),
    );
}
