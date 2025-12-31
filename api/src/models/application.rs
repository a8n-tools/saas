//! Application model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Application database model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Application {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub display_name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub is_active: bool,
    pub maintenance_mode: bool,
    pub maintenance_message: Option<String>,
    pub container_name: String,
    pub health_check_url: Option<String>,
    pub version: Option<String>,
    pub source_code_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Application response for public API (excludes internal fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Create from Application with access flag
    pub fn from_application(app: Application, has_access: bool) -> Self {
        Self {
            id: app.id,
            slug: app.slug,
            display_name: app.display_name,
            description: app.description,
            icon_url: app.icon_url,
            version: app.version,
            source_code_url: app.source_code_url,
            is_accessible: has_access && app.is_active && !app.maintenance_mode,
            maintenance_mode: app.maintenance_mode,
            maintenance_message: if app.maintenance_mode {
                app.maintenance_message
            } else {
                None
            },
        }
    }
}

/// Data for creating/updating an application (admin only)
#[derive(Debug, Clone, Deserialize)]
pub struct CreateApplication {
    pub name: String,
    pub slug: String,
    pub display_name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub container_name: String,
    pub health_check_url: Option<String>,
    pub version: Option<String>,
    pub source_code_url: Option<String>,
}
