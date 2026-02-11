//! Admin routes

use actix_web::web;

use crate::handlers;

/// Configure admin routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            // Dashboard stats
            .route("/stats", web::get().to(handlers::get_dashboard_stats))
            // System health
            .route("/health", web::get().to(handlers::get_system_health))
            // User management
            .route("/users", web::get().to(handlers::list_users))
            .route("/users/{user_id}", web::get().to(handlers::get_user))
            .route("/users/{user_id}", web::delete().to(handlers::delete_user))
            .route("/users/{user_id}/status", web::put().to(handlers::update_user_status))
            .route("/users/{user_id}/role", web::put().to(handlers::update_user_role))
            .route("/users/{user_id}/reset-password", web::post().to(handlers::admin_reset_password))
            .route("/users/{user_id}/impersonate", web::post().to(handlers::impersonate_user))
            // Membership management
            .route("/memberships", web::get().to(handlers::list_memberships))
            .route("/memberships/grant", web::post().to(handlers::grant_membership))
            .route("/memberships/revoke", web::post().to(handlers::revoke_membership))
            // Application management
            .route("/applications", web::get().to(handlers::list_all_applications))
            .route("/applications/{app_id}", web::put().to(handlers::update_application))
            // Audit logs
            .route("/audit-logs", web::get().to(handlers::list_audit_logs))
            // Notifications
            .route("/notifications", web::get().to(handlers::list_notifications))
            .route("/notifications/{notification_id}/read", web::post().to(handlers::mark_notification_read))
            .route("/notifications/read-all", web::post().to(handlers::mark_all_notifications_read)),
    );
}
