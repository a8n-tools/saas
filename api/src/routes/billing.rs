//! Billing route configuration

use actix_web::web;

use crate::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/billing")
            .route("/invoices", web::get().to(handlers::list_invoices))
            .route(
                "/invoices/{invoice_id}/download",
                web::get().to(handlers::download_invoice),
            ),
    );
}
