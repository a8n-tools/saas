//! User repository

use chrono::{self, DateTime, Utc};
use sqlx::postgres::Postgres;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{CreateUser, MembershipStatus, SubscriptionTier, User};

pub struct UserRepository;

impl UserRepository {
    /// Create a new user
    pub async fn create(pool: &PgPool, data: CreateUser) -> Result<User, AppError> {
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

        // DEV-525: an install whose very first admin arrives here (rather than
        // through a role change) still needs a super admin, or the gated
        // actions are unreachable forever.
        if user.role == "admin" {
            Self::ensure_super_admin(pool).await?;
            return Self::find_by_id(pool, user.id)
                .await?
                .ok_or_else(|| AppError::not_found("User"));
        }

        Ok(user)
    }

    /// DEV-525: hold the "at least one super admin exists, once any admin
    /// does" invariant by promoting the earliest-created admin when nobody
    /// holds the flag.
    ///
    /// Deliberately a single statement rather than a read-then-write: two
    /// concurrent invite acceptances would both see "no super admin" and race,
    /// and the `NOT EXISTS` guard inside the same statement is what makes the
    /// second one a no-op. It is also byte-for-byte the backfill in
    /// `20260804000048_add_super_admin.sql`, so migration and runtime cannot
    /// drift into two different rules.
    ///
    /// Never demotes: once an operator holds the flag it stays with them, and
    /// promoting someone else is a future admin action rather than a side
    /// effect of creating an account.
    pub async fn ensure_super_admin<'e, E>(executor: E) -> Result<(), AppError>
    where
        E: sqlx::Executor<'e, Database = Postgres>,
    {
        sqlx::query(
            r#"
            UPDATE users
            SET is_super_admin = TRUE
            WHERE id = (
                SELECT id FROM users
                WHERE role = 'admin' AND deleted_at IS NULL
                ORDER BY created_at ASC
                LIMIT 1
            )
            AND NOT EXISTS (
                SELECT 1 FROM users WHERE is_super_admin AND deleted_at IS NULL
            )
            "#,
        )
        .execute(executor)
        .await?;

        Ok(())
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

    /// DEV-525: replace the optional profile fields.
    ///
    /// Every field is set on every call, `None` included, so clearing a name in
    /// the settings form actually clears it. A partial-update helper would need
    /// the caller to distinguish "absent" from "null" in the request body, which
    /// the settings form has no way to express.
    pub async fn update_profile<'e, E>(
        executor: E,
        user_id: Uuid,
        first_name: Option<&str>,
        last_name: Option<&str>,
        phone: Option<&str>,
    ) -> Result<User, AppError>
    where
        E: sqlx::Executor<'e, Database = Postgres>,
    {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET first_name = $2, last_name = $3, phone = $4, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(first_name)
        .bind(last_name)
        .bind(phone)
        .fetch_optional(executor)
        .await?
        .ok_or_else(|| AppError::not_found("User"))?;

        Ok(user)
    }

    /// DEV-525: record the country of the user's most recent geolocatable
    /// login. Called only after a successful authentication, and only for an
    /// address that resolved to a real country.
    pub async fn set_last_login_country<'e, E>(
        executor: E,
        user_id: Uuid,
        country: Option<&str>,
    ) -> Result<(), AppError>
    where
        E: sqlx::Executor<'e, Database = Postgres>,
    {
        sqlx::query("UPDATE users SET last_login_country = $2, updated_at = NOW() WHERE id = $1")
            .bind(user_id)
            .bind(country)
            .execute(executor)
            .await?;

        Ok(())
    }

    /// DEV-525: set the per-user opt-out for new-login-location alerts.
    pub async fn set_login_location_alerts(
        pool: &PgPool,
        user_id: Uuid,
        enabled: bool,
    ) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET login_location_alerts = $2, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(enabled)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::not_found("User"))?;

        Ok(user)
    }

    /// DEV-525: store (or replace) the user's avatar.
    ///
    /// The `user_avatars` row and the `users.avatar_updated_at` marker are
    /// written in one transaction. Split across two statements they could drift
    /// into "the marker says an avatar exists, the table says it does not",
    /// which renders as a broken image for every viewer until someone notices.
    pub async fn set_avatar(
        pool: &PgPool,
        user_id: Uuid,
        mime_type: &str,
        data: &[u8],
    ) -> Result<User, AppError> {
        let mut tx = pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO user_avatars (user_id, mime_type, size_bytes, data, updated_at)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (user_id) DO UPDATE
            SET mime_type = EXCLUDED.mime_type,
                size_bytes = EXCLUDED.size_bytes,
                data = EXCLUDED.data,
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(mime_type)
        .bind(data.len() as i32)
        .bind(data)
        .execute(&mut *tx)
        .await?;

        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET avatar_updated_at = NOW(), updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::not_found("User"))?;

        tx.commit().await?;
        Ok(user)
    }

    /// DEV-525: remove the user's avatar. Idempotent: deleting an avatar that
    /// is already gone succeeds, because a client retrying a delete should not
    /// see an error for having got what it asked for.
    pub async fn clear_avatar(pool: &PgPool, user_id: Uuid) -> Result<User, AppError> {
        let mut tx = pool.begin().await?;

        sqlx::query("DELETE FROM user_avatars WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET avatar_updated_at = NULL, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::not_found("User"))?;

        tx.commit().await?;
        Ok(user)
    }

    /// DEV-525: fetch the stored avatar bytes and their MIME type.
    ///
    /// Kept off every other read path on purpose: this is the only query that
    /// touches the BYTEA, so `find_by_id` and the users list never carry image
    /// data they will not use.
    pub async fn get_avatar(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Option<(String, Vec<u8>)>, AppError> {
        let row: Option<(String, Vec<u8>)> =
            sqlx::query_as("SELECT mime_type, data FROM user_avatars WHERE user_id = $1")
                .bind(user_id)
                .fetch_optional(pool)
                .await?;

        Ok(row)
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

    /// Update membership status
    pub async fn update_membership_status<'e, E>(
        executor: E,
        user_id: Uuid,
        status: MembershipStatus,
    ) -> Result<(), AppError>
    where
        E: sqlx::Executor<'e, Database = Postgres>,
    {
        sqlx::query(
            r#"
            UPDATE users
            SET subscription_status = $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(status.as_str())
        .bind(user_id)
        .execute(executor)
        .await?;

        Ok(())
    }

    /// Activate membership (set subscription_status to 'active')
    pub async fn activate_membership(pool: &PgPool, user_id: Uuid) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET subscription_status = 'active',
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::not_found("User"))?;

        Ok(user)
    }

    /// Update Stripe customer ID
    pub async fn update_stripe_customer_id<'e, E>(
        executor: E,
        user_id: Uuid,
        customer_id: &str,
    ) -> Result<(), AppError>
    where
        E: sqlx::Executor<'e, Database = Postgres>,
    {
        sqlx::query(
            r#"
            UPDATE users
            SET stripe_customer_id = $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(customer_id)
        .bind(user_id)
        .execute(executor)
        .await?;

        Ok(())
    }

    /// Store the Stripe customer ID and authorized payment method ID captured at signup.
    pub async fn update_stripe_registration_info(
        pool: &PgPool,
        user_id: Uuid,
        customer_id: &str,
        payment_method_id: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE users
            SET stripe_customer_id = $1, stripe_payment_method_id = $2, updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(customer_id)
        .bind(payment_method_id)
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
    pub async fn set_grace_period<'e, E>(
        executor: E,
        user_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<(), AppError>
    where
        E: sqlx::Executor<'e, Database = Postgres>,
    {
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
        .execute(executor)
        .await?;

        Ok(())
    }

    /// Reset subscription tier to standard when a membership is revoked/canceled.
    /// This frees the lifetime or early_adopter slot so it can be assigned to the next user.
    pub async fn reset_subscription_tier<'e, E>(executor: E, user_id: Uuid) -> Result<(), AppError>
    where
        E: sqlx::Executor<'e, Database = Postgres>,
    {
        sqlx::query(
            r#"
            UPDATE users
            SET subscription_tier = 'standard',
                lifetime_member = FALSE,
                trial_ends_at = NULL,
                subscription_override_by = NULL,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(executor)
        .await?;

        Ok(())
    }

    /// Set subscription tier from a Stripe subscription event.
    /// Clears trial_ends_at (user is now a paid subscriber) but does not touch subscription_status.
    pub async fn upgrade_subscription_tier<'e, E>(
        executor: E,
        user_id: Uuid,
        tier: &SubscriptionTier,
    ) -> Result<(), AppError>
    where
        E: sqlx::Executor<'e, Database = Postgres>,
    {
        let lifetime_member = matches!(tier, SubscriptionTier::Lifetime | SubscriptionTier::Free);
        sqlx::query(
            r#"
            UPDATE users
            SET subscription_tier = $1,
                lifetime_member   = $2,
                trial_ends_at     = NULL,
                updated_at        = NOW()
            WHERE id = $3
            "#,
        )
        .bind(tier.as_str())
        .bind(lifetime_member)
        .bind(user_id)
        .execute(executor)
        .await?;

        Ok(())
    }

    /// Clear grace period
    pub async fn clear_grace_period<'e, E>(executor: E, user_id: Uuid) -> Result<(), AppError>
    where
        E: sqlx::Executor<'e, Database = Postgres>,
    {
        sqlx::query(
            r#"
            UPDATE users
            SET grace_period_start = NULL, grace_period_end = NULL, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .execute(executor)
        .await?;

        Ok(())
    }

    /// Update user's email address
    pub async fn update_email(
        pool: &PgPool,
        user_id: Uuid,
        new_email: &str,
        set_verified: bool,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE users
            SET email = $1, email_verified = $2, updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(new_email)
        .bind(set_verified)
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

    /// Set two_factor_enabled flag on a user
    pub async fn set_two_factor_enabled(
        pool: &PgPool,
        user_id: Uuid,
        enabled: bool,
    ) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE users SET two_factor_enabled = $2, updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(user_id)
        .bind(enabled)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update user role
    pub async fn update_role(pool: &PgPool, user_id: Uuid, role: &str) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET role = $2, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(role)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::not_found("User"))?;

        // DEV-525: promoting someone to admin on an install that has none yet
        // (the invite-accept path for an existing subscriber) has to seed the
        // super admin, or the gated actions stay unreachable.
        if user.role == "admin" {
            Self::ensure_super_admin(pool).await?;
            return Self::find_by_id(pool, user.id)
                .await?
                .ok_or_else(|| AppError::not_found("User"));
        }

        Ok(user)
    }

    /// List users with pagination
    pub async fn list_paginated(
        pool: &PgPool,
        page: i32,
        per_page: i32,
        search: Option<&str>,
        status_filter: Option<MembershipStatus>,
    ) -> Result<(Vec<User>, i64), AppError> {
        let offset = (page - 1) * per_page;

        // Build dynamic query based on filters
        let mut conditions = vec!["deleted_at IS NULL".to_string()];

        if search.is_some() {
            conditions.push("LOWER(email) LIKE LOWER($3)".to_string());
        }

        if let Some(_status) = &status_filter {
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

                let total: (i64,) = sqlx::query_as(&count_query).fetch_one(pool).await?;

                (users, total.0)
            }
        };

        Ok((users, total))
    }

    /// Atomically assign a subscription tier to a user.
    ///
    /// Must be called inside a transaction that holds a `pg_advisory_xact_lock`
    /// to prevent concurrent verifications from racing on the same slot count.
    /// Returns the tier that was assigned.
    pub async fn assign_subscription_tier<'e, E>(
        executor: E,
        user_id: Uuid,
        tier: &SubscriptionTier,
        early_adopter_trial_days: i64,
        standard_trial_days: i64,
    ) -> Result<(), AppError>
    where
        E: sqlx::Executor<'e, Database = Postgres>,
    {
        let (lifetime_member, trial_ends_at) = match tier {
            SubscriptionTier::Lifetime | SubscriptionTier::Free => (true, None),
            SubscriptionTier::EarlyAdopter => {
                let ends = chrono::Utc::now() + chrono::Duration::days(early_adopter_trial_days);
                (false, Some(ends))
            }
            SubscriptionTier::Standard => {
                let ends = chrono::Utc::now() + chrono::Duration::days(standard_trial_days);
                (false, Some(ends))
            }
        };

        sqlx::query(
            r#"
            UPDATE users
            SET subscription_tier = $1,
                lifetime_member = $2,
                trial_ends_at = $3,
                subscription_status = 'active',
                updated_at = NOW()
            WHERE id = $4
            "#,
        )
        .bind(tier.as_str())
        .bind(lifetime_member)
        .bind(trial_ends_at)
        .bind(user_id)
        .execute(executor)
        .await?;

        Ok(())
    }

    /// Count users assigned to each tier — used inside a transaction with an advisory lock
    /// to atomically determine which tier the next verified user should receive.
    ///
    /// Counts are based on how many users have actually been assigned each tier,
    /// not total verified users. This ensures tier slots are filled correctly even
    /// if users existed before the tier system was introduced.
    pub async fn count_tier_assignments<'e, E>(executor: E) -> Result<(i64, i64), AppError>
    where
        E: sqlx::Executor<'e, Database = Postgres>,
    {
        let row: (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE subscription_tier = 'lifetime' AND subscription_override_by IS NULL) AS lifetime_count,
                COUNT(*) FILTER (WHERE subscription_tier = 'early_adopter') AS early_adopter_count
            FROM users
            WHERE email_verified = true AND deleted_at IS NULL
            "#,
        )
        .fetch_one(executor)
        .await?;
        Ok(row)
    }

    /// Grant lifetime membership to a user (admin override).
    pub async fn grant_lifetime_membership(
        pool: &PgPool,
        user_id: Uuid,
        granted_by: Uuid,
    ) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET subscription_tier = 'lifetime',
                lifetime_member = TRUE,
                trial_ends_at = NULL,
                subscription_override_by = $2,
                subscription_status = 'active',
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(granted_by)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::not_found("User"))?;

        Ok(user)
    }

    /// Grant free membership to a user (admin override, not tied to signup count).
    pub async fn grant_free_membership(
        pool: &PgPool,
        user_id: Uuid,
        granted_by: Uuid,
    ) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET subscription_tier = 'free',
                lifetime_member = TRUE,
                trial_ends_at = NULL,
                subscription_override_by = $2,
                subscription_status = 'active',
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(granted_by)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::not_found("User"))?;

        Ok(user)
    }

    /// Get email addresses of all active admin users for system notifications
    pub async fn find_admin_emails(pool: &PgPool) -> Result<Vec<String>, AppError> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT email FROM users WHERE role = 'admin' AND deleted_at IS NULL ORDER BY created_at ASC",
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|(email,)| email).collect())
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

#[cfg(test)]
mod super_admin_tests {
    //! DEV-525: the super-admin invariant is enforced entirely in SQL, so these
    //! need a real Postgres. Skipped automatically when DATABASE_URL is unset,
    //! matching the `handlers::oci_registry::integration` pattern.
    //!
    //! Everything runs inside a transaction that is rolled back, so the rows
    //! these insert (and the flag they clear) never outlive the test, and a
    //! developer database with real admins in it is left untouched.

    use super::*;
    use sqlx::Row;

    async fn maybe_pool() -> Option<PgPool> {
        let url = std::env::var("DATABASE_URL").ok()?;
        PgPool::connect(&url).await.ok()
    }

    /// Insert an admin with an explicit `created_at`, so "earliest" is a fact
    /// the test controls rather than a race against clock resolution.
    async fn seed_admin(tx: &mut sqlx::PgConnection, minutes_ago: i64) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, role, subscription_status, created_at)
             VALUES ($1, $2, 'x', 'admin', 'none', NOW() - ($3 || ' minutes')::interval)",
        )
        .bind(id)
        .bind(format!("super-admin-test-{}@example.com", id))
        .bind(minutes_ago.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
        id
    }

    async fn is_super(tx: &mut sqlx::PgConnection, id: Uuid) -> bool {
        sqlx::query("SELECT is_super_admin FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .unwrap()
            .get::<bool, _>(0)
    }

    #[tokio::test]
    async fn promotes_the_earliest_admin_when_nobody_holds_the_flag() {
        let Some(pool) = maybe_pool().await else {
            return;
        };
        let mut tx = pool.begin().await.unwrap();

        // Simulate a fresh install: no super admin anywhere.
        sqlx::query("UPDATE users SET is_super_admin = FALSE")
            .execute(&mut *tx)
            .await
            .unwrap();
        // Older than anything a real deployment would already hold.
        let first = seed_admin(&mut tx, 100_000).await;
        let second = seed_admin(&mut tx, 99_999).await;

        UserRepository::ensure_super_admin(&mut *tx).await.unwrap();

        assert!(is_super(&mut tx, first).await, "earliest admin is promoted");
        assert!(!is_super(&mut tx, second).await, "only one is promoted");

        tx.rollback().await.unwrap();
    }

    #[tokio::test]
    async fn never_moves_the_flag_off_an_existing_super_admin() {
        let Some(pool) = maybe_pool().await else {
            return;
        };
        let mut tx = pool.begin().await.unwrap();

        sqlx::query("UPDATE users SET is_super_admin = FALSE")
            .execute(&mut *tx)
            .await
            .unwrap();
        let earliest = seed_admin(&mut tx, 100_000).await;
        let holder = seed_admin(&mut tx, 1).await;
        sqlx::query("UPDATE users SET is_super_admin = TRUE WHERE id = $1")
            .bind(holder)
            .execute(&mut *tx)
            .await
            .unwrap();

        // Twice, because idempotence is the property that makes it safe to call
        // on every admin creation.
        UserRepository::ensure_super_admin(&mut *tx).await.unwrap();
        UserRepository::ensure_super_admin(&mut *tx).await.unwrap();

        assert!(is_super(&mut tx, holder).await, "the holder keeps the flag");
        assert!(
            !is_super(&mut tx, earliest).await,
            "an operator's choice is not overridden by creation order"
        );

        tx.rollback().await.unwrap();
    }

    #[tokio::test]
    async fn ignores_soft_deleted_admins() {
        let Some(pool) = maybe_pool().await else {
            return;
        };
        let mut tx = pool.begin().await.unwrap();

        sqlx::query("UPDATE users SET is_super_admin = FALSE")
            .execute(&mut *tx)
            .await
            .unwrap();
        let deleted = seed_admin(&mut tx, 100_000).await;
        sqlx::query("UPDATE users SET deleted_at = NOW() WHERE id = $1")
            .bind(deleted)
            .execute(&mut *tx)
            .await
            .unwrap();
        let live = seed_admin(&mut tx, 99_999).await;

        UserRepository::ensure_super_admin(&mut *tx).await.unwrap();

        assert!(
            !is_super(&mut tx, deleted).await,
            "a deleted admin is skipped"
        );
        assert!(
            is_super(&mut tx, live).await,
            "the earliest live admin wins"
        );

        tx.rollback().await.unwrap();
    }
}

#[cfg(test)]
mod profile_tests {
    //! DEV-525: `update_profile` writes every field on every call, including
    //! `None`, so clearing a name in the settings form actually clears it.
    //! Needs a real Postgres; skipped when DATABASE_URL is unset, and wrapped in
    //! a rolled-back transaction so a developer database is left untouched.

    use super::*;
    use sqlx::Row;

    async fn maybe_pool() -> Option<PgPool> {
        let url = std::env::var("DATABASE_URL").ok()?;
        PgPool::connect(&url).await.ok()
    }

    #[tokio::test]
    async fn writes_then_clears_every_field() {
        let Some(pool) = maybe_pool().await else {
            return;
        };
        let mut tx = pool.begin().await.unwrap();

        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, role, subscription_status)
             VALUES ($1, $2, 'x', 'subscriber', 'none')",
        )
        .bind(id)
        .bind(format!("profile-test-{}@example.com", id))
        .execute(&mut *tx)
        .await
        .unwrap();

        let set = UserRepository::update_profile(
            &mut *tx,
            id,
            Some("Ada"),
            Some("Lovelace"),
            Some("+61 400 000 000"),
        )
        .await
        .unwrap();
        assert_eq!(set.first_name.as_deref(), Some("Ada"));
        assert_eq!(set.last_name.as_deref(), Some("Lovelace"));
        assert_eq!(set.phone.as_deref(), Some("+61 400 000 000"));

        // Omitting a field clears it: the settings form has no way to say
        // "leave this one alone", so a partial write would strand old values.
        let cleared = UserRepository::update_profile(&mut *tx, id, None, None, None)
            .await
            .unwrap();
        assert_eq!(cleared.first_name, None);
        assert_eq!(cleared.last_name, None);
        assert_eq!(cleared.phone, None);

        let row = sqlx::query("SELECT first_name, last_name, phone FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
        assert_eq!(row.get::<Option<String>, _>(0), None);
        assert_eq!(row.get::<Option<String>, _>(1), None);
        assert_eq!(row.get::<Option<String>, _>(2), None);

        tx.rollback().await.unwrap();
    }

    #[tokio::test]
    async fn login_alerts_default_on_and_can_be_turned_off() {
        let Some(pool) = maybe_pool().await else {
            return;
        };
        let mut tx = pool.begin().await.unwrap();

        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, role, subscription_status)
             VALUES ($1, $2, 'x', 'subscriber', 'none')",
        )
        .bind(id)
        .bind(format!("alerts-test-{}@example.com", id))
        .execute(&mut *tx)
        .await
        .unwrap();

        // DEV-525: a security alert nobody opted into is still worth sending,
        // so the column defaults on rather than off.
        let row = sqlx::query("SELECT login_location_alerts FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
        assert!(row.get::<bool, _>(0), "alerts must default on");

        sqlx::query("UPDATE users SET login_location_alerts = FALSE WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .unwrap();
        let row = sqlx::query("SELECT login_location_alerts FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
        assert!(!row.get::<bool, _>(0));

        tx.rollback().await.unwrap();
    }

    #[tokio::test]
    async fn records_the_login_country() {
        let Some(pool) = maybe_pool().await else {
            return;
        };
        let mut tx = pool.begin().await.unwrap();

        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, role, subscription_status)
             VALUES ($1, $2, 'x', 'subscriber', 'none')",
        )
        .bind(id)
        .bind(format!("country-test-{}@example.com", id))
        .execute(&mut *tx)
        .await
        .unwrap();

        UserRepository::set_last_login_country(&mut *tx, id, Some("AU"))
            .await
            .unwrap();
        let row = sqlx::query("SELECT last_login_country FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
        assert_eq!(row.get::<Option<String>, _>(0).as_deref(), Some("AU"));

        tx.rollback().await.unwrap();
    }

    /// 1x1 transparent PNG, the smallest valid payload to store.
    const PNG_1X1: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    async fn seed_user(tx: &mut sqlx::PgConnection, tag: &str) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, role, subscription_status)
             VALUES ($1, $2, 'x', 'subscriber', 'none')",
        )
        .bind(id)
        .bind(format!("{tag}-{id}@example.com"))
        .execute(&mut *tx)
        .await
        .unwrap();
        id
    }

    #[tokio::test]
    async fn avatar_marker_and_bytes_move_together() {
        let Some(pool) = maybe_pool().await else {
            return;
        };
        let id = {
            let mut tx = pool.begin().await.unwrap();
            let id = seed_user(&mut tx, "avatar").await;
            tx.commit().await.unwrap();
            id
        };

        // `set_avatar` and `clear_avatar` own their own transactions, so this
        // test cannot wrap them in one; it cleans up after itself instead.
        let before = UserRepository::find_by_id(&pool, id)
            .await
            .unwrap()
            .unwrap();
        assert!(before.avatar_updated_at.is_none(), "no avatar to start");

        let stored = UserRepository::set_avatar(&pool, id, "image/png", PNG_1X1)
            .await
            .unwrap();
        assert!(
            stored.avatar_updated_at.is_some(),
            "the marker must be set in the same transaction as the bytes"
        );
        let (mime, data) = UserRepository::get_avatar(&pool, id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(mime, "image/png");
        assert_eq!(data, PNG_1X1);

        let cleared = UserRepository::clear_avatar(&pool, id).await.unwrap();
        assert!(
            cleared.avatar_updated_at.is_none(),
            "clearing the bytes must clear the marker, or the client renders a broken image"
        );
        assert!(UserRepository::get_avatar(&pool, id)
            .await
            .unwrap()
            .is_none());

        // Idempotent: deleting what is already gone is not an error.
        UserRepository::clear_avatar(&pool, id).await.unwrap();

        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn re_upload_replaces_rather_than_accumulates() {
        let Some(pool) = maybe_pool().await else {
            return;
        };
        let id = {
            let mut tx = pool.begin().await.unwrap();
            let id = seed_user(&mut tx, "avatar-replace").await;
            tx.commit().await.unwrap();
            id
        };

        UserRepository::set_avatar(&pool, id, "image/png", PNG_1X1)
            .await
            .unwrap();
        UserRepository::set_avatar(&pool, id, "image/gif", b"GIF89a-second-upload")
            .await
            .unwrap();

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM user_avatars WHERE user_id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 1, "one row per user, replaced on re-upload");

        let (mime, data) = UserRepository::get_avatar(&pool, id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(mime, "image/gif");
        assert_eq!(data, b"GIF89a-second-upload");

        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn the_database_refuses_an_avatar_over_the_cap() {
        let Some(pool) = maybe_pool().await else {
            return;
        };
        let mut tx = pool.begin().await.unwrap();
        let id = seed_user(&mut tx, "avatar-cap").await;

        // The handler enforces this too, but the handler is not the only
        // possible writer, so the CHECK has to hold on its own.
        let oversized = vec![0u8; 2 * 1024 * 1024 + 1];
        let err = sqlx::query(
            "INSERT INTO user_avatars (user_id, mime_type, size_bytes, data)
             VALUES ($1, 'image/png', $2, $3)",
        )
        .bind(id)
        .bind(oversized.len() as i32)
        .bind(&oversized)
        .execute(&mut *tx)
        .await
        .expect_err("the size CHECK must reject 2 MiB + 1");
        assert!(
            err.to_string().contains("user_avatars_size_bytes_check"),
            "expected the size CHECK to fire, got: {err}"
        );

        tx.rollback().await.unwrap();
    }

    #[tokio::test]
    async fn the_database_refuses_a_lying_size() {
        let Some(pool) = maybe_pool().await else {
            return;
        };
        let mut tx = pool.begin().await.unwrap();
        let id = seed_user(&mut tx, "avatar-lie").await;

        // A row claiming 1 KB while carrying more is what would let the size
        // cap be sidestepped; `user_avatars_data_size` makes it unrepresentable.
        let err = sqlx::query(
            "INSERT INTO user_avatars (user_id, mime_type, size_bytes, data)
             VALUES ($1, 'image/png', 1024, $2)",
        )
        .bind(id)
        .bind(vec![0u8; 4096])
        .execute(&mut *tx)
        .await
        .expect_err("size_bytes must match the payload");
        assert!(
            err.to_string().contains("user_avatars_data_size"),
            "expected the payload-size CHECK to fire, got: {err}"
        );

        tx.rollback().await.unwrap();
    }

    #[tokio::test]
    async fn the_database_refuses_an_over_length_value() {
        let Some(pool) = maybe_pool().await else {
            return;
        };
        let mut tx = pool.begin().await.unwrap();

        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, role, subscription_status)
             VALUES ($1, $2, 'x', 'subscriber', 'none')",
        )
        .bind(id)
        .bind(format!("profile-len-{}@example.com", id))
        .execute(&mut *tx)
        .await
        .unwrap();

        // The handler guards this too, but the handler is not the only writer,
        // so the CHECK constraint has to hold on its own.
        let err = sqlx::query("UPDATE users SET first_name = $2 WHERE id = $1")
            .bind(id)
            .bind("a".repeat(65))
            .execute(&mut *tx)
            .await
            .expect_err("the CHECK constraint must reject 65 characters");
        assert!(
            err.to_string().contains("users_first_name_check"),
            "expected the first_name CHECK to fire, got: {err}"
        );

        tx.rollback().await.unwrap();
    }
}
