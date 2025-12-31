//! User repository

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{CreateUser, SubscriptionStatus, User, UserRole};

pub struct UserRepository;

impl UserRepository {
    /// Create a new user
    pub async fn create(
        pool: &PgPool,
        data: CreateUser,
    ) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, password_hash, role)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(&data.email)
        .bind(&data.password_hash)
        .bind(data.role.as_str())
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    /// Find user by ID
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Find user by email
    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users WHERE LOWER(email) = LOWER($1) AND deleted_at IS NULL
            "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Find user by Stripe customer ID
    pub async fn find_by_stripe_customer_id(
        pool: &PgPool,
        customer_id: &str,
    ) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users WHERE stripe_customer_id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(customer_id)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Update user's password hash
    pub async fn update_password(
        pool: &PgPool,
        user_id: Uuid,
        password_hash: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE users
            SET password_hash = $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(password_hash)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update email verified status
    pub async fn set_email_verified(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE users
            SET email_verified = TRUE, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update subscription status
    pub async fn update_subscription_status(
        pool: &PgPool,
        user_id: Uuid,
        status: SubscriptionStatus,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE users
            SET subscription_status = $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(status.as_str())
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update Stripe customer ID
    pub async fn update_stripe_customer_id(
        pool: &PgPool,
        user_id: Uuid,
        customer_id: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE users
            SET stripe_customer_id = $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(customer_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Lock price for user
    pub async fn lock_price(
        pool: &PgPool,
        user_id: Uuid,
        price_id: &str,
        amount: i32,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE users
            SET price_locked = TRUE, locked_price_id = $1, locked_price_amount = $2, updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(price_id)
        .bind(amount)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Set grace period
    pub async fn set_grace_period(
        pool: &PgPool,
        user_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE users
            SET grace_period_start = $1, grace_period_end = $2, updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(start)
        .bind(end)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Clear grace period
    pub async fn clear_grace_period(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE users
            SET grace_period_start = NULL, grace_period_end = NULL, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update last login timestamp
    pub async fn update_last_login(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE users
            SET last_login_at = NOW(), updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Soft delete user
    pub async fn soft_delete(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE users
            SET deleted_at = NOW(), updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// List users with pagination
    pub async fn list_paginated(
        pool: &PgPool,
        page: i32,
        per_page: i32,
        search: Option<&str>,
        status_filter: Option<SubscriptionStatus>,
    ) -> Result<(Vec<User>, i64), AppError> {
        let offset = (page - 1) * per_page;

        // Build dynamic query based on filters
        let mut conditions = vec!["deleted_at IS NULL".to_string()];

        if search.is_some() {
            conditions.push("LOWER(email) LIKE LOWER($3)".to_string());
        }

        if let Some(status) = &status_filter {
            let idx = if search.is_some() { 4 } else { 3 };
            conditions.push(format!("subscription_status = ${}", idx));
        }

        let where_clause = conditions.join(" AND ");
        let query = format!(
            "SELECT * FROM users WHERE {} ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            where_clause
        );
        let count_query = format!("SELECT COUNT(*) FROM users WHERE {}", where_clause);

        // Execute queries based on filters
        let (users, total): (Vec<User>, i64) = match (search, &status_filter) {
            (Some(s), Some(status)) => {
                let search_pattern = format!("%{}%", s);
                let users = sqlx::query_as::<_, User>(&query)
                    .bind(per_page)
                    .bind(offset)
                    .bind(&search_pattern)
                    .bind(status.as_str())
                    .fetch_all(pool)
                    .await?;

                let total: (i64,) = sqlx::query_as(&count_query)
                    .bind(&search_pattern)
                    .bind(status.as_str())
                    .fetch_one(pool)
                    .await?;

                (users, total.0)
            }
            (Some(s), None) => {
                let search_pattern = format!("%{}%", s);
                let users = sqlx::query_as::<_, User>(&query)
                    .bind(per_page)
                    .bind(offset)
                    .bind(&search_pattern)
                    .fetch_all(pool)
                    .await?;

                let total: (i64,) = sqlx::query_as(&count_query)
                    .bind(&search_pattern)
                    .fetch_one(pool)
                    .await?;

                (users, total.0)
            }
            (None, Some(status)) => {
                let users = sqlx::query_as::<_, User>(&query)
                    .bind(per_page)
                    .bind(offset)
                    .bind(status.as_str())
                    .fetch_all(pool)
                    .await?;

                let total: (i64,) = sqlx::query_as(&count_query)
                    .bind(status.as_str())
                    .fetch_one(pool)
                    .await?;

                (users, total.0)
            }
            (None, None) => {
                let users = sqlx::query_as::<_, User>(&query)
                    .bind(per_page)
                    .bind(offset)
                    .fetch_all(pool)
                    .await?;

                let total: (i64,) = sqlx::query_as(&count_query)
                    .fetch_one(pool)
                    .await?;

                (users, total.0)
            }
        };

        Ok((users, total))
    }

    /// Find users in grace period
    pub async fn find_in_grace_period(pool: &PgPool) -> Result<Vec<User>, AppError> {
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT * FROM users
            WHERE subscription_status = 'grace_period'
            AND grace_period_end IS NOT NULL
            AND deleted_at IS NULL
            ORDER BY grace_period_end ASC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(users)
    }
}
