//! Application error types and error handling
//!
//! This module provides a comprehensive error handling system for the API.

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::middleware::request_id::RequestId;

/// Application error type
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Validation error on field '{field}': {message}")]
    ValidationError { field: String, message: String },

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token has expired")]
    TokenExpired,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Resource not found: {resource}")]
    NotFound { resource: String },

    #[error("Conflict: {message}")]
    Conflict { message: String },

    #[error("Rate limited, retry after {retry_after} seconds")]
    RateLimited { retry_after: u64 },

    #[error("Internal error: {message}")]
    InternalError { message: String },

    #[error("Database error: {message}")]
    DatabaseError { message: String },
}

impl AppError {
    /// Get the error code string
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::ValidationError { .. } => "VALIDATION_ERROR",
            AppError::InvalidCredentials => "INVALID_CREDENTIALS",
            AppError::TokenExpired => "TOKEN_EXPIRED",
            AppError::Unauthorized => "UNAUTHORIZED",
            AppError::Forbidden => "FORBIDDEN",
            AppError::NotFound { .. } => "NOT_FOUND",
            AppError::Conflict { .. } => "CONFLICT",
            AppError::RateLimited { .. } => "RATE_LIMITED",
            AppError::InternalError { .. } => "INTERNAL_ERROR",
            AppError::DatabaseError { .. } => "DATABASE_ERROR",
        }
    }

    /// Create a validation error
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        AppError::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create a not found error
    pub fn not_found(resource: impl Into<String>) -> Self {
        AppError::NotFound {
            resource: resource.into(),
        }
    }

    /// Create a conflict error
    pub fn conflict(message: impl Into<String>) -> Self {
        AppError::Conflict {
            message: message.into(),
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        AppError::InternalError {
            message: message.into(),
        }
    }
}

/// Error response body
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: ErrorDetails,
    pub meta: ErrorMeta,
}

/// Error details
#[derive(Debug, Serialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Error metadata
#[derive(Debug, Serialize)]
pub struct ErrorMeta {
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            AppError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AppError::TokenExpired => StatusCode::UNAUTHORIZED,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::NotFound { .. } => StatusCode::NOT_FOUND,
            AppError::Conflict { .. } => StatusCode::CONFLICT,
            AppError::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            AppError::InternalError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::DatabaseError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let request_id = RequestId::new().0;

        let details = match self {
            AppError::ValidationError { field, .. } => {
                Some(serde_json::json!({ "field": field }))
            }
            AppError::RateLimited { retry_after } => {
                Some(serde_json::json!({ "retry_after": retry_after }))
            }
            _ => None,
        };

        let error_response = ErrorResponse {
            success: false,
            error: ErrorDetails {
                code: self.error_code().to_string(),
                message: self.to_string(),
                details,
            },
            meta: ErrorMeta {
                request_id,
                timestamp: Utc::now(),
            },
        };

        HttpResponse::build(self.status_code()).json(error_response)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!(error = %err, "Database error");

        match err {
            sqlx::Error::RowNotFound => AppError::NotFound {
                resource: "record".to_string(),
            },
            sqlx::Error::Database(db_err) => {
                // Check for unique constraint violations
                if let Some(code) = db_err.code() {
                    if code == "23505" {
                        return AppError::Conflict {
                            message: "Resource already exists".to_string(),
                        };
                    }
                }
                AppError::DatabaseError {
                    message: "A database error occurred".to_string(),
                }
            }
            _ => AppError::DatabaseError {
                message: "A database error occurred".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(
            AppError::validation("email", "invalid").error_code(),
            "VALIDATION_ERROR"
        );
        assert_eq!(AppError::InvalidCredentials.error_code(), "INVALID_CREDENTIALS");
        assert_eq!(AppError::TokenExpired.error_code(), "TOKEN_EXPIRED");
        assert_eq!(AppError::Unauthorized.error_code(), "UNAUTHORIZED");
        assert_eq!(AppError::Forbidden.error_code(), "FORBIDDEN");
        assert_eq!(AppError::not_found("user").error_code(), "NOT_FOUND");
        assert_eq!(AppError::conflict("exists").error_code(), "CONFLICT");
        assert_eq!(
            AppError::RateLimited { retry_after: 60 }.error_code(),
            "RATE_LIMITED"
        );
        assert_eq!(AppError::internal("oops").error_code(), "INTERNAL_ERROR");
        assert_eq!(
            AppError::DatabaseError {
                message: "err".to_string()
            }
            .error_code(),
            "DATABASE_ERROR"
        );
    }

    #[test]
    fn test_status_codes() {
        assert_eq!(
            AppError::validation("email", "invalid").status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::InvalidCredentials.status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(AppError::TokenExpired.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(AppError::Unauthorized.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(AppError::Forbidden.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(AppError::not_found("user").status_code(), StatusCode::NOT_FOUND);
        assert_eq!(AppError::conflict("exists").status_code(), StatusCode::CONFLICT);
        assert_eq!(
            AppError::RateLimited { retry_after: 60 }.status_code(),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            AppError::internal("oops").status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
