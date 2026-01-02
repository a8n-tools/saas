//! Authentication routes

use actix_web::web;

use crate::handlers;

/// Configure authentication routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(handlers::register))
            .route("/login", web::post().to(handlers::login))
            .route("/logout", web::post().to(handlers::logout))
            .route("/logout-all", web::post().to(handlers::logout_all))
            .route("/refresh", web::post().to(handlers::refresh_token))
            .route("/magic-link", web::post().to(handlers::request_magic_link))
            .route("/magic-link/verify", web::post().to(handlers::verify_magic_link))
            .route("/password-reset", web::post().to(handlers::request_password_reset))
            .route("/password-reset/verify", web::get().to(handlers::verify_password_reset_token))
            .route("/password-reset/confirm", web::post().to(handlers::confirm_password_reset)),
    );
}
