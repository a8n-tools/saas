//! a8n-api - Backend API for a8n.tools SaaS platform
//!
//! This crate provides the core API functionality for the a8n.tools platform,
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
