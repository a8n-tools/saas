//! Invoice model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Invoice database model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Invoice {
    pub id: Uuid,
    pub user_id: Uuid,
    pub invoice_number: String,
    pub amount_cents: i32,
    pub currency: String,
    pub description: String,
    pub pdf_storage_path: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Data for creating a new invoice record
#[derive(Debug, Clone)]
pub struct CreateInvoice {
    pub user_id: Uuid,
    pub invoice_number: String,
    pub amount_cents: i32,
    pub currency: String,
    pub description: String,
    pub pdf_storage_path: Option<String>,
}

/// Public invoice response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceResponse {
    pub id: Uuid,
    pub invoice_number: String,
    pub amount_cents: i32,
    pub currency: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

impl From<Invoice> for InvoiceResponse {
    fn from(inv: Invoice) -> Self {
        Self {
            id: inv.id,
            invoice_number: inv.invoice_number,
            amount_cents: inv.amount_cents,
            currency: inv.currency,
            description: inv.description,
            created_at: inv.created_at,
        }
    }
}
