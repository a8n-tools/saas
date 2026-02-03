//! Membership repository

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{CreateMembership, Membership};

pub struct MembershipRepository;

impl MembershipRepository {
    /// Create a new membership
    pub async fn create(pool: &PgPool, data: CreateMembership) -> Result<Membership, AppError> {
        let membership = sqlx::query_as::<_, Membership>(
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

        Ok(membership)
    }

    /// Find membership by ID
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Membership>, AppError> {
        let membership = sqlx::query_as::<_, Membership>(
            r#"
            SELECT * FROM subscriptions WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(membership)
    }

    /// Find membership by user ID
    pub async fn find_by_user_id(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Option<Membership>, AppError> {
        let membership = sqlx::query_as::<_, Membership>(
            r#"
            SELECT * FROM subscriptions WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(membership)
    }

    /// Find membership by Stripe subscription ID
    pub async fn find_by_stripe_subscription_id(
        pool: &PgPool,
        stripe_subscription_id: &str,
    ) -> Result<Option<Membership>, AppError> {
        let membership = sqlx::query_as::<_, Membership>(
            r#"
            SELECT * FROM subscriptions WHERE stripe_subscription_id = $1
            "#,
        )
        .bind(stripe_subscription_id)
        .fetch_optional(pool)
        .await?;

        Ok(membership)
    }

    /// Update membership status
    pub async fn update_status(
        pool: &PgPool,
        membership_id: Uuid,
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
        .bind(membership_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update membership period
    pub async fn update_period(
        pool: &PgPool,
        membership_id: Uuid,
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
        .bind(membership_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Set cancel at period end
    pub async fn set_cancel_at_period_end(
        pool: &PgPool,
        membership_id: Uuid,
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
        .bind(membership_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// List memberships with pagination
    pub async fn list_paginated(
        pool: &PgPool,
        page: i32,
        per_page: i32,
        status_filter: Option<&str>,
    ) -> Result<(Vec<Membership>, i64), AppError> {
        let offset = (page - 1) * per_page;

        let (memberships, total): (Vec<Membership>, i64) = if let Some(status) = status_filter {
            let memberships = sqlx::query_as::<_, Membership>(
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

            (memberships, total.0)
        } else {
            let memberships = sqlx::query_as::<_, Membership>(
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

            (memberships, total.0)
        };

        Ok((memberships, total))
    }
}
