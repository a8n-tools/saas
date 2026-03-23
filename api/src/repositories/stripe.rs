use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::stripe::StripeConfig;

pub struct StripeConfigRepository;

impl StripeConfigRepository {
    pub async fn get(pool: &PgPool) -> Result<StripeConfig, AppError> {
        let config = sqlx::query_as::<_, StripeConfig>("SELECT * FROM stripe_config WHERE id = 1")
            .fetch_one(pool)
            .await?;
        Ok(config)
    }

    /// Clears encrypted secrets (secret_key, webhook_secret and their nonces) from the DB.
    /// Called when decryption fails (e.g. encryption key was rotated).
    pub async fn clear_secrets(pool: &PgPool) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE stripe_config
            SET secret_key = NULL,
                secret_key_nonce = NULL,
                webhook_secret = NULL,
                webhook_secret_nonce = NULL,
                updated_at = NOW()
            WHERE id = 1
            "#,
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Updates only the fields that are `Some`. `None` leaves the existing DB value unchanged.
    /// Secrets are passed as pre-encrypted (ciphertext, nonce) pairs.
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        pool: &PgPool,
        secret_key: Option<Vec<u8>>,
        secret_key_nonce: Option<Vec<u8>>,
        webhook_secret: Option<Vec<u8>>,
        webhook_secret_nonce: Option<Vec<u8>>,
        price_id_personal: Option<&str>,
        price_id_business: Option<&str>,
        updated_by: Uuid,
    ) -> Result<StripeConfig, AppError> {
        let config = sqlx::query_as::<_, StripeConfig>(
            r#"
            UPDATE stripe_config
            SET
                secret_key            = CASE WHEN $1::BYTEA IS NOT NULL THEN $1 ELSE secret_key END,
                secret_key_nonce      = CASE WHEN $1::BYTEA IS NOT NULL THEN $2 ELSE secret_key_nonce END,
                webhook_secret        = CASE WHEN $3::BYTEA IS NOT NULL THEN $3 ELSE webhook_secret END,
                webhook_secret_nonce  = CASE WHEN $3::BYTEA IS NOT NULL THEN $4 ELSE webhook_secret_nonce END,
                price_id_personal     = CASE WHEN $5::TEXT IS NOT NULL THEN $5 ELSE price_id_personal END,
                price_id_business     = CASE WHEN $6::TEXT IS NOT NULL THEN $6 ELSE price_id_business END,
                updated_at            = NOW(),
                updated_by            = $7
            WHERE id = 1
            RETURNING *
            "#,
        )
        .bind(secret_key)
        .bind(secret_key_nonce)
        .bind(webhook_secret)
        .bind(webhook_secret_nonce)
        .bind(price_id_personal)
        .bind(price_id_business)
        .bind(updated_by)
        .fetch_one(pool)
        .await?;

        Ok(config)
    }
}
