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

    /// Updates only the fields that are `Some`. `None` leaves the existing DB value unchanged.
    pub async fn update(
        pool: &PgPool,
        secret_key: Option<&str>,
        webhook_secret: Option<&str>,
        price_id_personal: Option<&str>,
        price_id_business: Option<&str>,
        updated_by: Uuid,
    ) -> Result<StripeConfig, AppError> {
        let config = sqlx::query_as::<_, StripeConfig>(
            r#"
            UPDATE stripe_config
            SET
                secret_key        = CASE WHEN $1::TEXT IS NOT NULL THEN $1 ELSE secret_key END,
                webhook_secret    = CASE WHEN $2::TEXT IS NOT NULL THEN $2 ELSE webhook_secret END,
                price_id_personal = CASE WHEN $3::TEXT IS NOT NULL THEN $3 ELSE price_id_personal END,
                price_id_business = CASE WHEN $4::TEXT IS NOT NULL THEN $4 ELSE price_id_business END,
                updated_at        = NOW(),
                updated_by        = $5
            WHERE id = 1
            RETURNING *
            "#,
        )
        .bind(secret_key)
        .bind(webhook_secret)
        .bind(price_id_personal)
        .bind(price_id_business)
        .bind(updated_by)
        .fetch_one(pool)
        .await?;

        Ok(config)
    }
}
