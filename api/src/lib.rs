//! a8n-api - Backend API server
//!
//! This crate provides the core API functionality for the platform,
//! including authentication, membership management, and application access.

pub mod config;
pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod repositories;
pub mod responses;
pub mod routes;
pub mod services;
pub mod validation;

// Re-export commonly used types
pub use config::Config;
pub use errors::AppError;
pub use responses::{ApiResponse, ResponseMeta};
