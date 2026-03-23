//! Request handlers
//!
//! This module contains all HTTP request handlers organized by domain.

pub mod admin;
pub mod application;
pub mod auth;
pub mod feedback;
pub mod membership;
pub mod totp;
pub mod user;
pub mod webhook;

// Re-export handler functions for convenience
pub use application::{get_application, list_applications};
pub use auth::{
    accept_admin_invite, auth_redirect, confirm_password_reset, login, logout, logout_all,
    logout_redirect, refresh_token, register, request_magic_link, request_password_reset,
    setup_admin, setup_status, verify_magic_link, verify_password_reset_token,
};
pub use feedback::{
    delete_feedback, export_feedback, get_attachment, get_feedback, list_feedback,
    list_feedback_archive, respond_to_feedback, restore_feedback, submit_feedback,
    update_feedback_status,
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
    admin_reset_password, create_admin_invite, create_application, delete_application, delete_user,
    get_dashboard_stats, get_key_health, get_key_health_by_id, get_stripe_config, get_system_health, get_user, grant_lifetime_membership,
    grant_membership, impersonate_user, list_admin_invites, list_all_applications, list_audit_logs,
    list_notifications, list_memberships, list_users, revoke_admin_invite, swap_application_order,
    mark_all_notifications_read, mark_notification_read, revoke_membership, send_test_email,
    update_application, update_stripe_config, update_user_role, update_user_status,
};
