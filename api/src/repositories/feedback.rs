//! Feedback repository

use sqlx::{PgPool, QueryBuilder};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{CreateFeedback, Feedback, FeedbackStatus, RespondToFeedback};

pub struct FeedbackRepository;

impl FeedbackRepository {
    pub async fn create(pool: &PgPool, data: CreateFeedback) -> Result<Feedback, AppError> {
        let feedback = sqlx::query_as::<_, Feedback>(
            r#"
            INSERT INTO feedback (name, email, subject, tags, message, page_path, is_spam)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(data.name)
        .bind(data.email)
        .bind(data.subject)
        .bind(data.tags)
        .bind(data.message)
        .bind(data.page_path)
        .bind(data.is_spam)
        .fetch_one(pool)
        .await?;

        Ok(feedback)
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Feedback>, AppError> {
        let feedback = sqlx::query_as::<_, Feedback>(
            "SELECT * FROM feedback WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(feedback)
    }

    pub async fn list_paginated(
        pool: &PgPool,
        page: i32,
        per_page: i32,
        status: Option<&str>,
    ) -> Result<(Vec<Feedback>, i64), AppError> {
        let offset = (page - 1) * per_page;

        let mut query = QueryBuilder::new("SELECT * FROM feedback");
        let mut count_query = QueryBuilder::new("SELECT COUNT(*)::BIGINT FROM feedback");

        if let Some(status) = status {
            query.push(" WHERE status = ").push_bind(status);
            count_query.push(" WHERE status = ").push_bind(status);
        }

        query
            .push(" ORDER BY created_at DESC LIMIT ")
            .push_bind(per_page)
            .push(" OFFSET ")
            .push_bind(offset);

        let feedback = query
            .build_query_as::<Feedback>()
            .fetch_all(pool)
            .await?;

        let total: (i64,) = count_query
            .build_query_as()
            .fetch_one(pool)
            .await?;

        Ok((feedback, total.0))
    }

    pub async fn update_status(
        pool: &PgPool,
        id: Uuid,
        status: FeedbackStatus,
    ) -> Result<Feedback, AppError> {
        let feedback = sqlx::query_as::<_, Feedback>(
            "UPDATE feedback SET status = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
        )
        .bind(status.as_str())
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(feedback)
    }

    pub async fn respond(
        pool: &PgPool,
        feedback_id: Uuid,
        data: RespondToFeedback,
    ) -> Result<Feedback, AppError> {
        let feedback = sqlx::query_as::<_, Feedback>(
            r#"
            UPDATE feedback
            SET status = $1,
                admin_response = $2,
                responded_by = $3,
                responded_at = NOW(),
                updated_at = NOW()
            WHERE id = $4
            RETURNING *
            "#,
        )
        .bind(data.status.as_str())
        .bind(data.admin_response)
        .bind(data.responded_by)
        .bind(feedback_id)
        .fetch_one(pool)
        .await?;

        Ok(feedback)
    }
}
