//! Invoice repository — database access for invoices

use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{CreateInvoice, Invoice};

pub struct InvoiceRepository;

impl InvoiceRepository {
    /// Get the next invoice number using the database sequence.
    /// Format: INV-00001
    pub async fn next_invoice_number(pool: &PgPool) -> Result<String, AppError> {
        let row: (i64,) = sqlx::query_as("SELECT nextval('invoice_number_seq')")
            .fetch_one(pool)
            .await?;
        Ok(format!("INV-{:05}", row.0))
    }

    /// Create a new invoice record
    pub async fn create(pool: &PgPool, data: CreateInvoice) -> Result<Invoice, AppError> {
        let invoice = sqlx::query_as::<_, Invoice>(
            r#"
            INSERT INTO invoices (user_id, invoice_number, amount_cents, currency, description, pdf_storage_path)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(data.user_id)
        .bind(&data.invoice_number)
        .bind(data.amount_cents)
        .bind(&data.currency)
        .bind(&data.description)
        .bind(&data.pdf_storage_path)
        .fetch_one(pool)
        .await?;

        Ok(invoice)
    }

    /// List all invoices for a user, newest first
    pub async fn find_by_user_id(pool: &PgPool, user_id: Uuid) -> Result<Vec<Invoice>, AppError> {
        let invoices = sqlx::query_as::<_, Invoice>(
            "SELECT * FROM invoices WHERE user_id = $1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(invoices)
    }

    /// Find a single invoice belonging to a specific user
    pub async fn find_by_id_and_user(
        pool: &PgPool,
        invoice_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Invoice>, AppError> {
        let invoice = sqlx::query_as::<_, Invoice>(
            "SELECT * FROM invoices WHERE id = $1 AND user_id = $2",
        )
        .bind(invoice_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(invoice)
    }
}
