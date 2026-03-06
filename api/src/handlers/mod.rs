//! Request handlers
//!
//! This module contains all HTTP request handlers organized by domain.

pub mod admin;
pub mod application;
pub mod auth;
pub mod membership;
pub mod totp;
pub mod user;
pub mod webhook;

// Re-export handler functions for convenience
pub use application::{get_application, list_applications};
pub use auth::{
    auth_redirect, confirm_password_reset, login, logout, logout_all, logout_redirect,
    refresh_token, register,
    request_magic_link, request_password_reset, verify_magic_link, verify_password_reset_token,
};
pub use totp::{
    confirm_2fa, disable_2fa, get_2fa_status, regenerate_recovery_codes, setup_2fa, verify_2fa,
};
pub use membership::{
    billing_portal, cancel_membership, cancel_membership_immediate, create_checkout,
    get_payment_history, get_membership, reactivate_membership, subscribe,
};
pub use user::{
    change_password, confirm_email_change, confirm_email_verification, get_current_user,
    list_sessions, request_email_change, request_email_verification, revoke_session,
};
pub use webhook::stripe_webhook;

// Admin handlers
pub use admin::{
    admin_reset_password, delete_user, get_dashboard_stats, get_system_health, get_user,
    grant_membership, impersonate_user, list_all_applications, list_audit_logs,
    list_notifications, list_memberships, list_users, mark_all_notifications_read,
    mark_notification_read, revoke_membership, send_test_email, update_application,
    update_user_role, update_user_status,
};
