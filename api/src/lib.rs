//! a8n-api - Backend API for a8n.tools SaaS platform
//!
//! This crate provides the core API functionality for the a8n.tools platform,
//! including authentication, subscription management, and application access.

pub mod config;
pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod responses;
pub mod routes;
pub mod services;

// Re-export commonly used types
pub use config::Config;
pub use errors::AppError;
pub use responses::{ApiResponse, ResponseMeta};
