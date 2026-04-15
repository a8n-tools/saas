//! Routes for the global downloads endpoint.

use actix_web::web;
use crate::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/downloads", web::get().to(handlers::list_all_downloads));
}
