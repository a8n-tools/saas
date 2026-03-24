//! Billing handlers
//!
//! This module contains HTTP handlers for invoice/billing endpoints.

use actix_web::{web, HttpRequest, HttpResponse};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppError;
use crate::middleware::AuthenticatedUser;
use crate::models::InvoiceResponse;
use crate::repositories::InvoiceRepository;
use crate::responses::{get_request_id, success};
use crate::services::InvoiceService;


/// GET /v1/billing/invoices
/// List all invoices for the authenticated user
pub async fn list_invoices(
    req: HttpRequest,
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    let invoices = InvoiceRepository::find_by_user_id(&pool, user.0.sub).await?;
    let response: Vec<InvoiceResponse> = invoices.into_iter().map(InvoiceResponse::from).collect();

    Ok(success(response, request_id))
}

/// GET /v1/billing/invoices/{invoice_id}/download
/// Download a PDF invoice for the authenticated user
pub async fn download_invoice(
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
    invoice_service: web::Data<Arc<InvoiceService>>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let invoice_id = path.into_inner();

    let invoice = InvoiceRepository::find_by_id_and_user(&pool, invoice_id, user.0.sub)
        .await?
        .ok_or_else(|| AppError::not_found("Invoice"))?;

    let pdf_path = invoice
        .pdf_storage_path
        .ok_or_else(|| AppError::not_found("Invoice PDF"))?;

    let bytes = invoice_service.read_invoice_pdf(&pdf_path).await?;

    let filename = format!("{}.pdf", invoice.invoice_number);

    Ok(HttpResponse::Ok()
        .content_type("application/pdf")
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{filename}\""),
        ))
        .body(bytes))
}
