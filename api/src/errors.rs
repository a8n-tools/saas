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

    #[error("Rate limited ({code})")]
    RateLimitedCoded {
        code: String,
        retry_after_secs: Option<i64>,
    },

    #[error("Upstream error: {message}")]
    Upstream { message: String },

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
            AppError::RateLimitedCoded { .. } => "RATE_LIMITED",
            AppError::Upstream { .. } => "UPSTREAM_ERROR",
            AppError::InternalError { .. } => "INTERNAL_ERROR",
            AppError::DatabaseError { .. } => "DATABASE_ERROR",
        }
    }

    /// Get a dynamic error code (may differ from the static `error_code()`
    /// for variants that carry a code string)
    pub fn dynamic_error_code(&self) -> String {
        match self {
            AppError::RateLimitedCoded { code, .. } => code.clone(),
            other => other.error_code().to_string(),
        }
    }

    /// Create a rate-limited error with a machine-readable code and optional
    /// Retry-After duration in seconds.
    pub fn rate_limited(code: &str, retry_after_secs: Option<i64>) -> Self {
        AppError::RateLimitedCoded {
            code: code.to_string(),
            retry_after_secs,
        }
    }

    /// Create an upstream (502) error with a friendly message.
    pub fn upstream(message: impl Into<String>) -> Self {
        AppError::Upstream {
            message: message.into(),
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
            AppError::RateLimitedCoded { .. } => StatusCode::TOO_MANY_REQUESTS,
            AppError::Upstream { .. } => StatusCode::BAD_GATEWAY,
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
            AppError::RateLimitedCoded {
                retry_after_secs, ..
            } => retry_after_secs
                .map(|n| serde_json::json!({ "retry_after": n })),
            _ => None,
        };

        let client_message = match self {
            AppError::ValidationError { message, .. } => message.clone(),
            AppError::InvalidCredentials => {
                "The email or password you entered is incorrect.".to_string()
            }
            AppError::TokenExpired => {
                "Your session has expired. Please log in again.".to_string()
            }
            AppError::Unauthorized => "You need to log in to access this.".to_string(),
            AppError::Forbidden => "You don't have permission to do this.".to_string(),
            AppError::NotFound { .. } => {
                "The requested resource could not be found.".to_string()
            }
            AppError::Conflict { message } => message.clone(),
            AppError::RateLimited { retry_after } => {
                format!(
                    "Too many requests. Please wait {} seconds and try again.",
                    retry_after
                )
            }
            AppError::RateLimitedCoded {
                retry_after_secs, ..
            } => match retry_after_secs {
                Some(n) => format!(
                    "Too many requests. Please wait {} seconds and try again.",
                    n
                ),
                None => "Too many requests. Please try again later.".to_string(),
            },
            AppError::Upstream { .. } => {
                "The upstream service is temporarily unavailable. Please try again shortly.".to_string()
            }
            AppError::InternalError { .. } | AppError::DatabaseError { .. } => {
                "An unexpected error occurred. Please try again later.".to_string()
            }
        };

        let error_response = ErrorResponse {
            success: false,
            error: ErrorDetails {
                code: self.dynamic_error_code(),
                message: client_message,
                details,
            },
            meta: ErrorMeta {
                request_id,
                timestamp: Utc::now(),
            },
        };

        let mut response = HttpResponse::build(self.status_code());

        match self {
            AppError::RateLimited { retry_after } => {
                response.insert_header(("Retry-After", retry_after.to_string()));
            }
            AppError::RateLimitedCoded {
                retry_after_secs: Some(n),
                ..
            } => {
                response.insert_header(("Retry-After", n.to_string()));
            }
            _ => {}
        }

        response.json(error_response)
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
        assert_eq!(
            AppError::DatabaseError {
                message: "err".to_string()
            }
            .status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_error_constructors() {
        match AppError::validation("email", "bad format") {
            AppError::ValidationError { field, message } => {
                assert_eq!(field, "email");
                assert_eq!(message, "bad format");
            }
            _ => panic!("wrong variant"),
        }
        match AppError::not_found("user") {
            AppError::NotFound { resource } => assert_eq!(resource, "user"),
            _ => panic!("wrong variant"),
        }
        match AppError::conflict("duplicate") {
            AppError::Conflict { message } => assert_eq!(message, "duplicate"),
            _ => panic!("wrong variant"),
        }
        match AppError::internal("oops") {
            AppError::InternalError { message } => assert_eq!(message, "oops"),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_display_messages() {
        assert_eq!(
            AppError::validation("email", "invalid").to_string(),
            "Validation error on field 'email': invalid"
        );
        assert_eq!(AppError::InvalidCredentials.to_string(), "Invalid credentials");
        assert_eq!(AppError::TokenExpired.to_string(), "Token has expired");
        assert_eq!(AppError::Unauthorized.to_string(), "Unauthorized");
        assert_eq!(AppError::Forbidden.to_string(), "Forbidden");
        assert_eq!(
            AppError::not_found("user").to_string(),
            "Resource not found: user"
        );
        assert_eq!(
            AppError::conflict("exists").to_string(),
            "Conflict: exists"
        );
        assert_eq!(
            AppError::RateLimited { retry_after: 60 }.to_string(),
            "Rate limited, retry after 60 seconds"
        );
        assert_eq!(
            AppError::internal("oops").to_string(),
            "Internal error: oops"
        );
        assert_eq!(
            AppError::DatabaseError {
                message: "err".to_string()
            }
            .to_string(),
            "Database error: err"
        );
    }

    #[test]
    fn test_error_response_json_shape() {
        let err = AppError::validation("email", "invalid format");
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body = resp.into_body();
        let bytes = actix_web::body::to_bytes(body);
        // Use a runtime to resolve the future
        let rt = actix_web::rt::Runtime::new().unwrap();
        let bytes = rt.block_on(bytes).unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(json["success"], false);
        assert_eq!(json["error"]["code"], "VALIDATION_ERROR");
        assert_eq!(json["error"]["message"], "invalid format");
        assert_eq!(json["error"]["details"]["field"], "email");
        assert!(json["meta"]["request_id"].is_string());
        assert!(json["meta"]["timestamp"].is_string());
    }

    #[test]
    fn test_rate_limited_response_has_details() {
        let err = AppError::RateLimited { retry_after: 30 };
        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(
            resp.headers().get("Retry-After").unwrap().to_str().unwrap(),
            "30"
        );

        let body = resp.into_body();
        let rt = actix_web::rt::Runtime::new().unwrap();
        let bytes = rt.block_on(actix_web::body::to_bytes(body)).unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["error"]["details"]["retry_after"], 30);
    }

    #[test]
    fn rate_limited_sets_retry_after_header() {
        let err = AppError::rate_limited("download_daily_limit", Some(3600));
        let resp = err.error_response();
        assert_eq!(resp.status().as_u16(), 429);
        assert_eq!(
            resp.headers().get("retry-after").and_then(|v| v.to_str().ok()),
            Some("3600"),
        );
    }

    #[test]
    fn rate_limited_without_retry_after() {
        let err = AppError::rate_limited("download_concurrency_limit", None);
        let resp = err.error_response();
        assert_eq!(resp.status().as_u16(), 429);
        assert!(resp.headers().get("retry-after").is_none());
    }

    #[test]
    fn upstream_error_is_502() {
        let err = AppError::upstream("forgejo timeout");
        let resp = err.error_response();
        assert_eq!(resp.status().as_u16(), 502);
    }

    #[test]
    fn test_internal_error_hides_details() {
        let err = AppError::internal("secret internal info");
        let resp = err.error_response();

        let body = resp.into_body();
        let rt = actix_web::rt::Runtime::new().unwrap();
        let bytes = rt.block_on(actix_web::body::to_bytes(body)).unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        // Client message should NOT leak internal details
        assert_eq!(
            json["error"]["message"],
            "An unexpected error occurred. Please try again later."
        );
        assert!(json["error"]["details"].is_null());
    }
}
