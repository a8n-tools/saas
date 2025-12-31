//! Application repository

use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::Application;

pub struct ApplicationRepository;

impl ApplicationRepository {
    /// List all active applications
    pub async fn list_active(pool: &PgPool) -> Result<Vec<Application>, AppError> {
        let apps = sqlx::query_as::<_, Application>(
            r#"
            SELECT * FROM applications
            WHERE is_active = TRUE
            ORDER BY display_name ASC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(apps)
    }

    /// Find application by ID
    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Application>, AppError> {
        let app = sqlx::query_as::<_, Application>(
            r#"
            SELECT * FROM applications WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(app)
    }

    /// Find application by slug
    pub async fn find_by_slug(pool: &PgPool, slug: &str) -> Result<Option<Application>, AppError> {
        let app = sqlx::query_as::<_, Application>(
            r#"
            SELECT * FROM applications WHERE slug = $1
            "#,
        )
        .bind(slug)
        .fetch_optional(pool)
        .await?;

        Ok(app)
    }

    /// Find active application by slug
    pub async fn find_active_by_slug(
        pool: &PgPool,
        slug: &str,
    ) -> Result<Option<Application>, AppError> {
        let app = sqlx::query_as::<_, Application>(
            r#"
            SELECT * FROM applications WHERE slug = $1 AND is_active = TRUE
            "#,
        )
        .bind(slug)
        .fetch_optional(pool)
        .await?;

        Ok(app)
    }

    /// Toggle maintenance mode
    pub async fn set_maintenance_mode(
        pool: &PgPool,
        app_id: Uuid,
        maintenance: bool,
        message: Option<&str>,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE applications
            SET maintenance_mode = $1, maintenance_message = $2, updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(maintenance)
        .bind(message)
        .bind(app_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Toggle active status
    pub async fn set_active(pool: &PgPool, app_id: Uuid, active: bool) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE applications
            SET is_active = $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(active)
        .bind(app_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update application version
    pub async fn update_version(
        pool: &PgPool,
        app_id: Uuid,
        version: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE applications
            SET version = $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(version)
        .bind(app_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// List all applications (admin)
    pub async fn list_all(pool: &PgPool) -> Result<Vec<Application>, AppError> {
        let apps = sqlx::query_as::<_, Application>(
            r#"
            SELECT * FROM applications ORDER BY display_name ASC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(apps)
    }
}
