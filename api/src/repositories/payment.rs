//! Payment repository

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{CreatePayment, PaymentHistory};

pub struct PaymentRepository;

impl PaymentRepository {
    /// Create a new payment record
    pub async fn create(pool: &PgPool, data: CreatePayment) -> Result<PaymentHistory, AppError> {
        let payment = sqlx::query_as::<_, PaymentHistory>(
            r#"
            INSERT INTO payment_history (
                user_id, subscription_id, stripe_payment_intent_id, stripe_invoice_id,
                amount, currency, status, failure_reason
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(data.user_id)
        .bind(data.subscription_id)
        .bind(&data.stripe_payment_intent_id)
        .bind(&data.stripe_invoice_id)
        .bind(data.amount)
        .bind(&data.currency)
        .bind(data.status.as_str())
        .bind(&data.failure_reason)
        .fetch_one(pool)
        .await?;

        Ok(payment)
    }

    /// Find payment by ID
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<PaymentHistory>, AppError> {
        let payment = sqlx::query_as::<_, PaymentHistory>(
            r#"
            SELECT * FROM payment_history WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(payment)
    }

    /// Find payment by Stripe payment intent ID
    pub async fn find_by_payment_intent_id(
        pool: &PgPool,
        payment_intent_id: &str,
    ) -> Result<Option<PaymentHistory>, AppError> {
        let payment = sqlx::query_as::<_, PaymentHistory>(
            r#"
            SELECT * FROM payment_history WHERE stripe_payment_intent_id = $1
            "#,
        )
        .bind(payment_intent_id)
        .fetch_optional(pool)
        .await?;

        Ok(payment)
    }

    /// List payments for a user with pagination
    pub async fn list_by_user(
        pool: &PgPool,
        user_id: Uuid,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<PaymentHistory>, i64), AppError> {
        let offset = (page - 1) * per_page;

        let payments = sqlx::query_as::<_, PaymentHistory>(
            r#"
            SELECT * FROM payment_history
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM payment_history WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok((payments, total.0))
    }

    /// List payments with date range filter
    pub async fn list_by_date_range(
        pool: &PgPool,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<PaymentHistory>, i64), AppError> {
        let offset = (page - 1) * per_page;

        let payments = sqlx::query_as::<_, PaymentHistory>(
            r#"
            SELECT * FROM payment_history
            WHERE created_at >= $1 AND created_at <= $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(start_date)
        .bind(end_date)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM payment_history WHERE created_at >= $1 AND created_at <= $2",
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_one(pool)
        .await?;

        Ok((payments, total.0))
    }

    /// Update payment status
    pub async fn update_status(
        pool: &PgPool,
        payment_id: Uuid,
        status: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE payment_history SET status = $1 WHERE id = $2
            "#,
        )
        .bind(status)
        .bind(payment_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Mark payment as refunded
    pub async fn mark_refunded(
        pool: &PgPool,
        payment_id: Uuid,
        refund_amount: i32,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE payment_history
            SET status = 'refunded', refunded_at = NOW(), refund_amount = $1
            WHERE id = $2
            "#,
        )
        .bind(refund_amount)
        .bind(payment_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
