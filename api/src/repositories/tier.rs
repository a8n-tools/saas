//! Tier configuration repository (singleton, id=1)

use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::tier::TierConfigRow;

pub struct TierConfigRepository;

impl TierConfigRepository {
    pub async fn get(pool: &PgPool) -> Result<TierConfigRow, AppError> {
        let row = sqlx::query_as::<_, TierConfigRow>("SELECT * FROM tier_config WHERE id = 1")
            .fetch_one(pool)
            .await?;
        Ok(row)
    }

    /// Updates only the fields that are `Some`. `None` leaves the existing DB value unchanged.
    pub async fn update(
        pool: &PgPool,
        lifetime_slots: Option<i64>,
        early_adopter_slots: Option<i64>,
        early_adopter_trial_days: Option<i64>,
        standard_trial_days: Option<i64>,
        updated_by: Uuid,
    ) -> Result<TierConfigRow, AppError> {
        let row = sqlx::query_as::<_, TierConfigRow>(
            r#"
            UPDATE tier_config
            SET
                lifetime_slots           = COALESCE($1, lifetime_slots),
                early_adopter_slots      = COALESCE($2, early_adopter_slots),
                early_adopter_trial_days = COALESCE($3, early_adopter_trial_days),
                standard_trial_days      = COALESCE($4, standard_trial_days),
                updated_at               = NOW(),
                updated_by               = $5
            WHERE id = 1
            RETURNING *
            "#,
        )
        .bind(lifetime_slots)
        .bind(early_adopter_slots)
        .bind(early_adopter_trial_days)
        .bind(standard_trial_days)
        .bind(updated_by)
        .fetch_one(pool)
        .await?;

        Ok(row)
    }
}
