//! Request handlers
//!
//! This module contains all HTTP request handlers organized by domain.

pub mod admin;
pub mod application;
pub mod auth;
pub mod membership;
pub mod user;
pub mod webhook;

// Re-export handler functions for convenience
pub use application::{get_application, list_applications};
pub use auth::{
    confirm_password_reset, login, logout, logout_all, refresh_token, register,
    request_magic_link, request_password_reset, verify_magic_link, verify_password_reset_token,
};
pub use membership::{
    billing_portal, cancel_membership, create_checkout, get_payment_history, get_membership,
    reactivate_membership, subscribe,
};
pub use user::{change_password, get_current_user, list_sessions, revoke_session};
pub use webhook::stripe_webhook;

// Admin handlers
pub use admin::{
    admin_reset_password, delete_user, get_dashboard_stats, get_system_health, get_user,
    grant_membership, impersonate_user, list_all_applications, list_audit_logs,
    list_notifications, list_memberships, list_users, mark_all_notifications_read,
    mark_notification_read, revoke_membership, update_application, update_user_role,
    update_user_status,
};
