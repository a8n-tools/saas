//! Audit log and admin notification models

use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;

/// Audit action types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    UserLogin,
    UserLogout,
    UserRegistered,
    MagicLinkRequested,
    MagicLinkUsed,
    PasswordResetRequested,
    PasswordResetCompleted,
    PasswordChanged,
    MembershipCreated,
    MembershipCanceled,
    MembershipReactivated,
    PaymentSucceeded,
    PaymentFailed,
    GracePeriodStarted,
    GracePeriodEnded,
    AdminUserImpersonated,
    AdminPasswordReset,
    AdminMembershipGranted,
    AdminMembershipRevoked,
    AdminUserDeactivated,
    AdminUserActivated,
    ApplicationMaintenanceToggled,
}

impl AuditAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditAction::UserLogin => "user_login",
            AuditAction::UserLogout => "user_logout",
            AuditAction::UserRegistered => "user_registered",
            AuditAction::MagicLinkRequested => "magic_link_requested",
            AuditAction::MagicLinkUsed => "magic_link_used",
            AuditAction::PasswordResetRequested => "password_reset_requested",
            AuditAction::PasswordResetCompleted => "password_reset_completed",
            AuditAction::PasswordChanged => "password_changed",
            AuditAction::MembershipCreated => "membership_created",
            AuditAction::MembershipCanceled => "membership_canceled",
            AuditAction::MembershipReactivated => "membership_reactivated",
            AuditAction::PaymentSucceeded => "payment_succeeded",
            AuditAction::PaymentFailed => "payment_failed",
            AuditAction::GracePeriodStarted => "grace_period_started",
            AuditAction::GracePeriodEnded => "grace_period_ended",
            AuditAction::AdminUserImpersonated => "admin_user_impersonated",
            AuditAction::AdminPasswordReset => "admin_password_reset",
            AuditAction::AdminMembershipGranted => "admin_membership_granted",
            AuditAction::AdminMembershipRevoked => "admin_membership_revoked",
            AuditAction::AdminUserDeactivated => "admin_user_deactivated",
            AuditAction::AdminUserActivated => "admin_user_activated",
            AuditAction::ApplicationMaintenanceToggled => "application_maintenance_toggled",
        }
    }

    pub fn is_admin_action(&self) -> bool {
        matches!(
            self,
            AuditAction::AdminUserImpersonated
                | AuditAction::AdminPasswordReset
                | AuditAction::AdminMembershipGranted
                | AuditAction::AdminMembershipRevoked
                | AuditAction::AdminUserDeactivated
                | AuditAction::AdminUserActivated
                | AuditAction::ApplicationMaintenanceToggled
        )
    }
}

/// Audit severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl AuditSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditSeverity::Info => "info",
            AuditSeverity::Warning => "warning",
            AuditSeverity::Error => "error",
            AuditSeverity::Critical => "critical",
        }
    }
}

impl Default for AuditSeverity {
    fn default() -> Self {
        AuditSeverity::Info
    }
}

/// Audit log database model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub actor_id: Option<Uuid>,
    pub actor_email: Option<String>,
    pub actor_role: Option<String>,
    pub actor_ip_address: Option<IpNetwork>,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub old_values: Option<JsonValue>,
    pub new_values: Option<JsonValue>,
    pub metadata: Option<JsonValue>,
    pub is_admin_action: bool,
    pub severity: String,
    pub created_at: DateTime<Utc>,
}

/// Data for creating a new audit log entry
#[derive(Debug, Clone)]
pub struct CreateAuditLog {
    pub actor_id: Option<Uuid>,
    pub actor_email: Option<String>,
    pub actor_role: Option<String>,
    pub actor_ip_address: Option<IpNetwork>,
    pub action: AuditAction,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub old_values: Option<JsonValue>,
    pub new_values: Option<JsonValue>,
    pub metadata: Option<JsonValue>,
    pub severity: AuditSeverity,
}

impl CreateAuditLog {
    pub fn new(action: AuditAction) -> Self {
        Self {
            actor_id: None,
            actor_email: None,
            actor_role: None,
            actor_ip_address: None,
            action,
            resource_type: None,
            resource_id: None,
            old_values: None,
            new_values: None,
            metadata: None,
            severity: AuditSeverity::Info,
        }
    }

    pub fn with_actor(mut self, id: Uuid, email: &str, role: &str) -> Self {
        self.actor_id = Some(id);
        self.actor_email = Some(email.to_string());
        self.actor_role = Some(role.to_string());
        self
    }

    pub fn with_ip(mut self, ip: Option<IpNetwork>) -> Self {
        self.actor_ip_address = ip;
        self
    }

    pub fn with_resource(mut self, resource_type: &str, resource_id: Uuid) -> Self {
        self.resource_type = Some(resource_type.to_string());
        self.resource_id = Some(resource_id);
        self
    }

    pub fn with_metadata(mut self, metadata: JsonValue) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn with_severity(mut self, severity: AuditSeverity) -> Self {
        self.severity = severity;
        self
    }
}

/// Admin notification types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    NewSignup,
    PaymentFailed,
    MembershipCanceled,
    GracePeriodExpiring,
    SystemAlert,
}

impl NotificationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationType::NewSignup => "new_signup",
            NotificationType::PaymentFailed => "payment_failed",
            NotificationType::MembershipCanceled => "membership_canceled",
            NotificationType::GracePeriodExpiring => "grace_period_expiring",
            NotificationType::SystemAlert => "system_alert",
        }
    }
}

/// Admin notification database model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AdminNotification {
    pub id: Uuid,
    #[sqlx(rename = "type")]
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub metadata: Option<JsonValue>,
    pub user_id: Option<Uuid>,
    pub is_read: bool,
    pub read_by: Option<Uuid>,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Data for creating a new admin notification
#[derive(Debug, Clone)]
pub struct CreateAdminNotification {
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub metadata: Option<JsonValue>,
    pub user_id: Option<Uuid>,
}
