//! Subscription repository

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{CreateSubscription, Subscription};

pub struct SubscriptionRepository;

impl SubscriptionRepository {
    /// Create a new subscription
    pub async fn create(pool: &PgPool, data: CreateSubscription) -> Result<Subscription, AppError> {
        let subscription = sqlx::query_as::<_, Subscription>(
            r#"
            INSERT INTO subscriptions (
                user_id, stripe_subscription_id, stripe_price_id, status,
                current_period_start, current_period_end, amount, currency
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#,
        )
        .bind(data.user_id)
        .bind(&data.stripe_subscription_id)
        .bind(&data.stripe_price_id)
        .bind(&data.status)
        .bind(data.current_period_start)
        .bind(data.current_period_end)
        .bind(data.amount)
        .bind(&data.currency)
        .fetch_one(pool)
        .await?;

        Ok(subscription)
    }

    /// Find subscription by ID
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Subscription>, AppError> {
        let subscription = sqlx::query_as::<_, Subscription>(
            r#"
            SELECT * FROM subscriptions WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(subscription)
    }

    /// Find subscription by user ID
    pub async fn find_by_user_id(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Option<Subscription>, AppError> {
        let subscription = sqlx::query_as::<_, Subscription>(
            r#"
            SELECT * FROM subscriptions WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(subscription)
    }

    /// Find subscription by Stripe subscription ID
    pub async fn find_by_stripe_subscription_id(
        pool: &PgPool,
        stripe_subscription_id: &str,
    ) -> Result<Option<Subscription>, AppError> {
        let subscription = sqlx::query_as::<_, Subscription>(
            r#"
            SELECT * FROM subscriptions WHERE stripe_subscription_id = $1
            "#,
        )
        .bind(stripe_subscription_id)
        .fetch_optional(pool)
        .await?;

        Ok(subscription)
    }

    /// Update subscription status
    pub async fn update_status(
        pool: &PgPool,
        subscription_id: Uuid,
        status: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE subscriptions
            SET status = $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(status)
        .bind(subscription_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update subscription period
    pub async fn update_period(
        pool: &PgPool,
        subscription_id: Uuid,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE subscriptions
            SET current_period_start = $1, current_period_end = $2, updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(period_start)
        .bind(period_end)
        .bind(subscription_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Set cancel at period end
    pub async fn set_cancel_at_period_end(
        pool: &PgPool,
        subscription_id: Uuid,
        cancel: bool,
    ) -> Result<(), AppError> {
        let canceled_at = if cancel { Some(Utc::now()) } else { None };

        sqlx::query(
            r#"
            UPDATE subscriptions
            SET cancel_at_period_end = $1, canceled_at = $2, updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(cancel)
        .bind(canceled_at)
        .bind(subscription_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// List subscriptions with pagination
    pub async fn list_paginated(
        pool: &PgPool,
        page: i32,
        per_page: i32,
        status_filter: Option<&str>,
    ) -> Result<(Vec<Subscription>, i64), AppError> {
        let offset = (page - 1) * per_page;

        let (subscriptions, total): (Vec<Subscription>, i64) = if let Some(status) = status_filter {
            let subscriptions = sqlx::query_as::<_, Subscription>(
                r#"
                SELECT * FROM subscriptions
                WHERE status = $3
                ORDER BY created_at DESC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(per_page)
            .bind(offset)
            .bind(status)
            .fetch_all(pool)
            .await?;

            let total: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM subscriptions WHERE status = $1",
            )
            .bind(status)
            .fetch_one(pool)
            .await?;

            (subscriptions, total.0)
        } else {
            let subscriptions = sqlx::query_as::<_, Subscription>(
                r#"
                SELECT * FROM subscriptions
                ORDER BY created_at DESC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool)
            .await?;

            let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM subscriptions")
                .fetch_one(pool)
                .await?;

            (subscriptions, total.0)
        };

        Ok((subscriptions, total))
    }
}
