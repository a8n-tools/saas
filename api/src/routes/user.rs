//! User routes

use actix_web::web;

use crate::handlers;

/// Configure user routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("/me", web::get().to(handlers::get_current_user))
            .route("/me/password", web::put().to(handlers::change_password))
            .route("/me/sessions", web::get().to(handlers::list_sessions))
            .route("/me/sessions/{session_id}", web::delete().to(handlers::revoke_session)),
    );
}
