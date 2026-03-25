//! Invoice service — PDF generation and invoice management

use chrono::Utc;
use printpdf::*;
use sqlx::PgPool;
use std::io::BufWriter;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{CreateInvoice, Invoice, SubscriptionTier};
use crate::repositories::InvoiceRepository;

pub struct InvoiceService {
    storage_base_path: String,
}

impl InvoiceService {
    pub fn new(storage_base_path: String) -> Self {
        Self { storage_base_path }
    }

    /// Generate an invoice for a user after email verification.
    /// Saves a PDF to disk and creates the invoice record in the database.
    /// Returns the created Invoice.
    pub async fn generate_invoice(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        email: &str,
        tier: &SubscriptionTier,
    ) -> Result<Invoice, AppError> {
        let invoice_number = InvoiceRepository::next_invoice_number(pool).await?;
        let (amount_cents, description) = tier_invoice_details(tier);

        let pdf_bytes = generate_pdf_bytes(&invoice_number, email, &description, amount_cents)
            .map_err(|e| AppError::InternalError { message: format!("PDF generation failed: {e}") })?;

        let relative_path = format!("{}/{}.pdf", user_id, invoice_number);
        let full_dir = format!("{}/{}", self.storage_base_path, user_id);
        let full_path = format!("{}/{}.pdf", full_dir, invoice_number);

        tokio::fs::create_dir_all(&full_dir)
            .await
            .map_err(|e| AppError::InternalError { message: format!("Failed to create invoice directory: {e}") })?;

        tokio::fs::write(&full_path, &pdf_bytes)
            .await
            .map_err(|e| AppError::InternalError { message: format!("Failed to write invoice PDF: {e}") })?;

        let invoice = InvoiceRepository::create(
            pool,
            CreateInvoice {
                user_id,
                invoice_number,
                amount_cents,
                currency: "usd".to_string(),
                description,
                pdf_storage_path: Some(relative_path),
            },
        )
        .await?;

        Ok(invoice)
    }

    /// Generate an invoice for a Stripe payment (membership purchase or renewal).
    /// Accepts amount and description directly rather than deriving from tier.
    pub async fn generate_payment_invoice(
        &self,
        pool: &PgPool,
        user_id: Uuid,
        email: &str,
        amount_cents: i32,
        description: &str,
    ) -> Result<Invoice, AppError> {
        let invoice_number = InvoiceRepository::next_invoice_number(pool).await?;

        let pdf_bytes = generate_pdf_bytes(&invoice_number, email, description, amount_cents)
            .map_err(|e| AppError::InternalError { message: format!("PDF generation failed: {e}") })?;

        let relative_path = format!("{}/{}.pdf", user_id, invoice_number);
        let full_dir = format!("{}/{}", self.storage_base_path, user_id);
        let full_path = format!("{}/{}.pdf", full_dir, invoice_number);

        tokio::fs::create_dir_all(&full_dir)
            .await
            .map_err(|e| AppError::InternalError { message: format!("Failed to create invoice directory: {e}") })?;

        tokio::fs::write(&full_path, &pdf_bytes)
            .await
            .map_err(|e| AppError::InternalError { message: format!("Failed to write invoice PDF: {e}") })?;

        let invoice = InvoiceRepository::create(
            pool,
            CreateInvoice {
                user_id,
                invoice_number,
                amount_cents,
                currency: "usd".to_string(),
                description: description.to_string(),
                pdf_storage_path: Some(relative_path),
            },
        )
        .await?;

        Ok(invoice)
    }

    /// Read the PDF bytes for an invoice from disk.
    pub async fn read_invoice_pdf(
        &self,
        pdf_storage_path: &str,
    ) -> Result<Vec<u8>, AppError> {
        let full_path = format!("{}/{}", self.storage_base_path, pdf_storage_path);
        tokio::fs::read(&full_path)
            .await
            .map_err(|e| AppError::InternalError { message: format!("Failed to read invoice PDF: {e}") })
    }
}

/// Returns `(amount_cents, description)` for a given subscription tier.
fn tier_invoice_details(tier: &SubscriptionTier) -> (i32, String) {
    match tier {
        SubscriptionTier::Lifetime => (0, "Lifetime Access — Founding Member".to_string()),
        SubscriptionTier::Trial3m => (0, "3-Month Free Trial Access".to_string()),
        SubscriptionTier::Trial1m => (0, "1-Month Free Trial Access".to_string()),
    }
}

/// Generate a simple PDF invoice and return the raw bytes.
fn generate_pdf_bytes(
    invoice_number: &str,
    email: &str,
    description: &str,
    amount_cents: i32,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let (doc, page1, layer1) =
        PdfDocument::new("Invoice", Mm(210.0), Mm(297.0), "Layer 1");
    let layer = doc.get_page(page1).get_layer(layer1);

    let regular = doc.add_builtin_font(BuiltinFont::Helvetica)?;
    let bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;

    // Title
    layer.use_text("INVOICE", 28.0, Mm(20.0), Mm(265.0), &bold);

    // Invoice number
    layer.use_text(
        &format!("Invoice #: {invoice_number}"),
        11.0,
        Mm(20.0),
        Mm(250.0),
        &regular,
    );

    // Date
    let today = Utc::now().format("%Y-%m-%d").to_string();
    layer.use_text(
        &format!("Date: {today}"),
        11.0,
        Mm(20.0),
        Mm(243.0),
        &regular,
    );

    // Billed to
    layer.use_text("Billed to:", 11.0, Mm(20.0), Mm(228.0), &bold);
    layer.use_text(email, 11.0, Mm(20.0), Mm(221.0), &regular);

    // Divider label
    layer.use_text("Description", 10.0, Mm(20.0), Mm(205.0), &bold);
    layer.use_text("Amount", 10.0, Mm(160.0), Mm(205.0), &bold);

    // Line item
    layer.use_text(description, 11.0, Mm(20.0), Mm(196.0), &regular);
    let amount_str = if amount_cents == 0 {
        "Free".to_string()
    } else {
        format!("${:.2}", amount_cents as f64 / 100.0)
    };
    layer.use_text(&amount_str, 11.0, Mm(160.0), Mm(196.0), &regular);

    // Total
    layer.use_text("Total:", 12.0, Mm(140.0), Mm(180.0), &bold);
    layer.use_text(&amount_str, 12.0, Mm(160.0), Mm(180.0), &bold);

    // Footer
    layer.use_text(
        "Thank you for being part of the community.",
        9.0,
        Mm(20.0),
        Mm(20.0),
        &regular,
    );

    let mut bytes = Vec::new();
    doc.save(&mut BufWriter::new(&mut bytes))?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invoice_number_format_lifetime() {
        let (amount, desc) = tier_invoice_details(&SubscriptionTier::Lifetime);
        assert_eq!(amount, 0);
        assert!(desc.contains("Lifetime"));
    }

    #[test]
    fn invoice_number_format_trial3m() {
        let (amount, desc) = tier_invoice_details(&SubscriptionTier::Trial3m);
        assert_eq!(amount, 0);
        assert!(desc.contains("3-Month"));
    }

    #[test]
    fn invoice_number_format_trial1m() {
        let (amount, desc) = tier_invoice_details(&SubscriptionTier::Trial1m);
        assert_eq!(amount, 0);
        assert!(desc.contains("1-Month"));
    }

    #[test]
    fn pdf_generation_produces_bytes() {
        let bytes = generate_pdf_bytes("INV-00001", "user@example.com", "Test Description", 0)
            .expect("PDF generation should succeed");
        assert!(!bytes.is_empty());
        // PDF files start with %PDF
        assert!(bytes.starts_with(b"%PDF"));
    }
}
