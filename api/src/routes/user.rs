//! User routes

use actix_web::web;

use crate::handlers;

/// Configure user routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("/me", web::get().to(handlers::get_current_user))
            .route("/me/profile", web::put().to(handlers::update_profile))
            .route(
                "/me/login-alerts",
                web::put().to(handlers::update_login_alerts),
            )
            // DEV-525: avatar upload / fetch / removal. The 3 MiB body limit
            // leaves room for the 2 MiB image plus multipart overhead; the
            // authoritative cap is `ImagePolicy::avatar().max_bytes`, enforced
            // while streaming.
            .service(
                web::resource("/me/avatar")
                    .app_data(actix_web::web::PayloadConfig::new(3 * 1024 * 1024))
                    .route(web::post().to(handlers::upload_avatar))
                    .route(web::get().to(handlers::get_avatar))
                    .route(web::delete().to(handlers::delete_avatar)),
            )
            .route("/me/password", web::put().to(handlers::change_password))
            .route("/me/email", web::post().to(handlers::request_email_change))
            .route(
                "/me/email/confirm",
                web::post().to(handlers::confirm_email_change),
            )
            .route(
                "/me/email/verify",
                web::post().to(handlers::request_email_verification),
            )
            .route(
                "/me/email/verify/confirm",
                web::post().to(handlers::confirm_email_verification),
            )
            .route("/me/sessions", web::get().to(handlers::list_sessions))
            .route("/me", web::delete().to(handlers::delete_account))
            .route(
                "/me/sessions/{session_id}",
                web::delete().to(handlers::revoke_session),
            ),
    );
}
