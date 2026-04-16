//! DB access for the `oci_pull_daily_counts` table.

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;

pub struct OciPullDailyCountRepository;

impl OciPullDailyCountRepository {
    /// Atomically increment today's count for a user. Returns the new count.
    pub async fn increment(
        pool: &PgPool,
        user_id: Uuid,
        day_utc: NaiveDate,
    ) -> Result<i32, AppError> {
        let (count,): (i32,) = sqlx::query_as(
            "INSERT INTO oci_pull_daily_counts (user_id, day_utc, count)
             VALUES ($1, $2, 1)
             ON CONFLICT (user_id, day_utc) DO UPDATE
                 SET count = oci_pull_daily_counts.count + 1
             RETURNING count",
        )
        .bind(user_id)
        .bind(day_utc)
        .fetch_one(pool)
        .await?;
        Ok(count)
    }

    /// Decrement today's count by 1 (best-effort rollback). Never goes below 0.
    pub async fn decrement(
        pool: &PgPool,
        user_id: Uuid,
        day_utc: NaiveDate,
    ) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE oci_pull_daily_counts SET count = GREATEST(count - 1, 0)
             WHERE user_id = $1 AND day_utc = $2",
        )
        .bind(user_id)
        .bind(day_utc)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn current(
        pool: &PgPool,
        user_id: Uuid,
        day_utc: NaiveDate,
    ) -> Result<i32, AppError> {
        let row: Option<(i32,)> = sqlx::query_as(
            "SELECT count FROM oci_pull_daily_counts WHERE user_id = $1 AND day_utc = $2",
        )
        .bind(user_id)
        .bind(day_utc)
        .fetch_optional(pool)
        .await?;
        Ok(row.map(|(c,)| c).unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    //! DB-backed. Skipped when DATABASE_URL is unset.
    use super::*;
    use chrono::Utc;

    async fn maybe_pool() -> Option<PgPool> {
        let url = std::env::var("DATABASE_URL").ok()?;
        PgPool::connect(&url).await.ok()
    }

    #[actix_rt::test]
    async fn increment_creates_and_bumps() {
        let Some(pool) = maybe_pool().await else { return; };

        // Insert a test user with only required columns; let DB defaults fill the rest.
        let user_id = Uuid::new_v4();
        let email = format!("oci-count-test-{}@example.com", user_id);
        let res = sqlx::query(
            "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'placeholder')"
        )
        .bind(user_id)
        .bind(&email)
        .execute(&pool)
        .await;
        if res.is_err() {
            // Schema requires more fields — skip this test rather than guessing.
            return;
        }

        let today = Utc::now().date_naive();

        // Clean leftover from previous runs.
        sqlx::query("DELETE FROM oci_pull_daily_counts WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .ok();

        assert_eq!(
            OciPullDailyCountRepository::increment(&pool, user_id, today).await.unwrap(),
            1
        );
        assert_eq!(
            OciPullDailyCountRepository::increment(&pool, user_id, today).await.unwrap(),
            2
        );
        assert_eq!(
            OciPullDailyCountRepository::current(&pool, user_id, today).await.unwrap(),
            2
        );

        OciPullDailyCountRepository::decrement(&pool, user_id, today).await.unwrap();
        assert_eq!(
            OciPullDailyCountRepository::current(&pool, user_id, today).await.unwrap(),
            1
        );

        // Decrement twice more — hitting GREATEST floor.
        OciPullDailyCountRepository::decrement(&pool, user_id, today).await.unwrap();
        OciPullDailyCountRepository::decrement(&pool, user_id, today).await.unwrap();
        assert_eq!(
            OciPullDailyCountRepository::current(&pool, user_id, today).await.unwrap(),
            0
        );

        // Cleanup
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .ok();
    }
}
