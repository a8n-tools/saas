//! Custom middleware
//!
//! This module contains custom Actix-Web middleware.

pub mod auth;
pub mod request_id;

// Re-export commonly used items
pub use auth::{
    extract_client_ip, extract_device_info, AdminUser, AuthCookies, AuthenticatedUser,
    OptionalUser, SubscribedUser,
};
