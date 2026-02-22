//! Custom middleware
//!
//! This module contains custom Actix-Web middleware.

pub mod auth;
pub mod auto_ban;
pub mod request_id;
pub mod security_headers;

// Re-export commonly used items
pub use auth::{
    extract_client_ip, extract_device_info, AdminUser, AuthCookies, AuthenticatedUser,
    MemberUser, OptionalUser,
};
pub use auto_ban::{AutoBanMiddleware, AutoBanService};
pub use security_headers::SecurityHeaders;
