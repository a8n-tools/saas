//! Application repository

use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{Application, CreateApplication, UpdateApplication};

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

    /// Update application fields (admin)
    pub async fn update(
        pool: &PgPool,
        app_id: Uuid,
        data: &UpdateApplication,
    ) -> Result<Application, AppError> {
        let app = sqlx::query_as::<_, Application>(
            r#"
            UPDATE applications
            SET display_name     = COALESCE($1, display_name),
                description      = COALESCE($2, description),
                icon_url         = COALESCE($3, icon_url),
                source_code_url  = COALESCE($4, source_code_url),
                version          = COALESCE($5, version),
                subdomain        = COALESCE($6, subdomain),
                container_name   = COALESCE($7, container_name),
                health_check_url = COALESCE($8, health_check_url),
                is_active        = COALESCE($9, is_active),
                maintenance_mode = COALESCE($10, maintenance_mode),
                maintenance_message = COALESCE($11, maintenance_message),
                webhook_url      = COALESCE($12, webhook_url),
                updated_at       = NOW()
            WHERE id = $13
            RETURNING *
            "#,
        )
        .bind(data.display_name.as_deref())
        .bind(data.description.as_deref())
        .bind(data.icon_url.as_deref())
        .bind(data.source_code_url.as_deref())
        .bind(data.version.as_deref())
        .bind(data.subdomain.as_deref())
        .bind(data.container_name.as_deref())
        .bind(data.health_check_url.as_deref())
        .bind(data.is_active)
        .bind(data.maintenance_mode)
        .bind(data.maintenance_message.as_deref())
        .bind(data.webhook_url.as_deref())
        .bind(app_id)
        .fetch_one(pool)
        .await?;

        Ok(app)
    }

    /// Create a new application (admin)
    pub async fn create(pool: &PgPool, data: &CreateApplication) -> Result<Application, AppError> {
        let app = sqlx::query_as::<_, Application>(
            r#"
            INSERT INTO applications (name, slug, display_name, description, icon_url,
                container_name, health_check_url, subdomain, webhook_url, version, source_code_url)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(&data.name)
        .bind(&data.slug)
        .bind(&data.display_name)
        .bind(data.description.as_deref())
        .bind(data.icon_url.as_deref())
        .bind(&data.container_name)
        .bind(data.health_check_url.as_deref())
        .bind(data.subdomain.as_deref())
        .bind(data.webhook_url.as_deref())
        .bind(data.version.as_deref())
        .bind(data.source_code_url.as_deref())
        .fetch_one(pool)
        .await?;

        Ok(app)
    }

    /// Delete an application by ID (admin)
    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM applications WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::not_found("Application"));
        }

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
