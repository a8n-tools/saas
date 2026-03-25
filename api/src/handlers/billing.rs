//! Billing handlers
//!
//! This module contains HTTP handlers for invoice/billing endpoints.

use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppError;
use crate::middleware::AuthenticatedUser;
use crate::models::{InvoiceResponse, RateLimitConfig};
use crate::repositories::{InvoiceRepository, RateLimitRepository};
use crate::responses::{get_request_id, success};
use crate::services::{InvoiceService, StripeService};
use crate::middleware::extract_client_ip;


/// Request body for SetupIntent creation
#[derive(Debug, Deserialize)]
pub struct CreateSetupIntentRequest {
    pub email: String,
}

/// Response for SetupIntent creation
#[derive(Debug, Serialize)]
pub struct CreateSetupIntentResponse {
    pub client_secret: String,
    pub customer_id: String,
}

/// POST /v1/billing/setup-intent
/// Create a Stripe Customer and SetupIntent for $0 card authorization at signup.
/// Unauthenticated — the user does not exist yet at this point.
pub async fn create_setup_intent(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    stripe: web::Data<Arc<StripeService>>,
    body: web::Json<CreateSetupIntentRequest>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let ip_address = extract_client_ip(&req);

    // Rate-limit by IP using the same budget as registration
    let ip_key = ip_address.map(|ip| ip.to_string()).unwrap_or_default();
    let (_count, exceeded) =
        RateLimitRepository::check_and_increment(&pool, &ip_key, &RateLimitConfig::REGISTRATION)
            .await?;
    if exceeded {
        let retry_after =
            RateLimitRepository::get_retry_after(&pool, &ip_key, &RateLimitConfig::REGISTRATION)
                .await?;
        return Err(AppError::RateLimited { retry_after });
    }

    crate::validation::validate_email(&body.email)?;

    let (customer_id, client_secret) = stripe.create_setup_intent(&body.email).await?;

    Ok(success(
        CreateSetupIntentResponse {
            client_secret,
            customer_id,
        },
        request_id,
    ))
}

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
