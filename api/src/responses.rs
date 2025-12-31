//! Standardized API response types
//!
//! This module provides consistent response formatting across all API endpoints.

use actix_web::{HttpMessage, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::middleware::request_id::RequestId;

/// Generic API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    pub meta: ResponseMeta,
}

/// Response metadata
#[derive(Debug, Serialize, Clone)]
pub struct ResponseMeta {
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
}

impl ResponseMeta {
    /// Create new response metadata
    pub fn new(request_id: String) -> Self {
        Self {
            request_id,
            timestamp: Utc::now(),
        }
    }

    /// Create metadata from an HTTP request (extracts request ID from extensions)
    pub fn from_request(req: &HttpRequest) -> Self {
        let request_id = req
            .extensions()
            .get::<RequestId>()
            .map(|id| id.0.clone())
            .unwrap_or_else(|| RequestId::new().0);

        Self::new(request_id)
    }
}

/// Paginated response wrapper
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

impl<T: Serialize> PaginatedResponse<T> {
    /// Create a new paginated response
    pub fn new(items: Vec<T>, total: i64, page: i32, per_page: i32) -> Self {
        let total_pages = ((total as f64) / (per_page as f64)).ceil() as i32;
        Self {
            items,
            total,
            page,
            per_page,
            total_pages,
        }
    }
}

/// Create a successful response with data
pub fn success<T: Serialize>(data: T, request_id: String) -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(data),
        meta: ResponseMeta::new(request_id),
    })
}

/// Create a successful response without data
pub fn success_no_data(request_id: String) -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse::<()> {
        success: true,
        data: None,
        meta: ResponseMeta::new(request_id),
    })
}

/// Create a response for resource creation (201 Created)
pub fn created<T: Serialize>(data: T, request_id: String) -> HttpResponse {
    HttpResponse::Created().json(ApiResponse {
        success: true,
        data: Some(data),
        meta: ResponseMeta::new(request_id),
    })
}

/// Create a response with pagination
pub fn paginated<T: Serialize>(
    items: Vec<T>,
    total: i64,
    page: i32,
    per_page: i32,
    request_id: String,
) -> HttpResponse {
    let paginated = PaginatedResponse::new(items, total, page, per_page);
    HttpResponse::Ok().json(ApiResponse {
        success: true,
        data: Some(paginated),
        meta: ResponseMeta::new(request_id),
    })
}

/// Helper to extract request ID from HTTP request
pub fn get_request_id(req: &HttpRequest) -> String {
    req.extensions()
        .get::<RequestId>()
        .map(|id| id.0.clone())
        .unwrap_or_else(|| RequestId::new().0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize)]
    struct TestData {
        id: i32,
        name: String,
    }

    #[test]
    fn test_api_response_serialization() {
        let response = ApiResponse {
            success: true,
            data: Some(TestData {
                id: 1,
                name: "test".to_string(),
            }),
            meta: ResponseMeta::new("req_123".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"request_id\":\"req_123\""));
    }

    #[test]
    fn test_paginated_response() {
        let items = vec![TestData {
            id: 1,
            name: "test".to_string(),
        }];
        let paginated = PaginatedResponse::new(items, 100, 1, 10);

        assert_eq!(paginated.total, 100);
        assert_eq!(paginated.page, 1);
        assert_eq!(paginated.per_page, 10);
        assert_eq!(paginated.total_pages, 10);
    }

    #[test]
    fn test_paginated_response_with_remainder() {
        let items: Vec<TestData> = vec![];
        let paginated = PaginatedResponse::new(items, 25, 1, 10);

        assert_eq!(paginated.total_pages, 3); // 25 / 10 = 2.5, ceil = 3
    }

    #[test]
    fn test_response_meta_timestamp() {
        let before = Utc::now();
        let meta = ResponseMeta::new("req_test".to_string());
        let after = Utc::now();

        assert!(meta.timestamp >= before && meta.timestamp <= after);
    }
}
