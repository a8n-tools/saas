//! Application handlers
//!
//! This module contains HTTP handlers for application endpoints.

use actix_web::{web, HttpRequest, HttpResponse};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::middleware::OptionalUser;
use crate::models::Application;
use crate::repositories::ApplicationRepository;
use crate::responses::{get_request_id, success};

/// Application response with access information
#[derive(Debug, Serialize)]
pub struct ApplicationResponse {
    pub id: Uuid,
    pub slug: String,
    pub display_name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub version: Option<String>,
    pub source_code_url: Option<String>,
    pub is_accessible: bool,
    pub maintenance_mode: bool,
    pub maintenance_message: Option<String>,
}

impl ApplicationResponse {
    fn from_app(app: Application, has_access: bool) -> Self {
        Self {
            id: app.id,
            slug: app.slug,
            display_name: app.display_name,
            description: app.description,
            icon_url: app.icon_url,
            version: app.version,
            source_code_url: app.source_code_url,
            is_accessible: has_access && !app.maintenance_mode,
            maintenance_mode: app.maintenance_mode,
            maintenance_message: if app.maintenance_mode {
                app.maintenance_message
            } else {
                None
            },
        }
    }
}

/// GET /v1/applications
/// List all active applications
pub async fn list_applications(
    req: HttpRequest,
    user: OptionalUser,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    // Check if user has active subscription
    let has_access = user
        .0
        .as_ref()
        .map(|claims| {
            claims.subscription_status == "active" || claims.subscription_status == "grace_period"
        })
        .unwrap_or(false);

    // Get all active applications
    let apps = ApplicationRepository::list_active(&pool).await?;

    let apps_response: Vec<ApplicationResponse> = apps
        .into_iter()
        .map(|app| ApplicationResponse::from_app(app, has_access))
        .collect();

    Ok(success(
        serde_json::json!({ "applications": apps_response }),
        request_id,
    ))
}

/// GET /v1/applications/{slug}
/// Get a specific application by slug
pub async fn get_application(
    req: HttpRequest,
    user: OptionalUser,
    path: web::Path<String>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let slug = path.into_inner();

    // Check if user has active subscription
    let has_access = user
        .0
        .as_ref()
        .map(|claims| {
            claims.subscription_status == "active" || claims.subscription_status == "grace_period"
        })
        .unwrap_or(false);

    // Get the application
    let app = ApplicationRepository::find_active_by_slug(&pool, &slug)
        .await?
        .ok_or(AppError::not_found("Application"))?;

    let app_response = ApplicationResponse::from_app(app, has_access);

    Ok(success(app_response, request_id))
}
