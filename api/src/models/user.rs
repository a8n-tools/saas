//! User model and related types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// User roles in the system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Subscriber,
    Admin,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Subscriber => "subscriber",
            UserRole::Admin => "admin",
        }
    }
}

impl From<String> for UserRole {
    fn from(s: String) -> Self {
        match s.as_str() {
            "admin" => UserRole::Admin,
            _ => UserRole::Subscriber,
        }
    }
}

impl From<&str> for UserRole {
    fn from(s: &str) -> Self {
        match s {
            "admin" => UserRole::Admin,
            _ => UserRole::Subscriber,
        }
    }
}

/// Membership status for users
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipStatus {
    None,
    Active,
    PastDue,
    Canceled,
    GracePeriod,
}

impl MembershipStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MembershipStatus::None => "none",
            MembershipStatus::Active => "active",
            MembershipStatus::PastDue => "past_due",
            MembershipStatus::Canceled => "canceled",
            MembershipStatus::GracePeriod => "grace_period",
        }
    }

    /// Check if the user has access to paid features
    pub fn has_access(&self) -> bool {
        matches!(self, MembershipStatus::Active | MembershipStatus::GracePeriod)
    }
}

impl From<String> for MembershipStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "active" => MembershipStatus::Active,
            "past_due" => MembershipStatus::PastDue,
            "canceled" => MembershipStatus::Canceled,
            "grace_period" => MembershipStatus::GracePeriod,
            _ => MembershipStatus::None,
        }
    }
}

impl From<&str> for MembershipStatus {
    fn from(s: &str) -> Self {
        match s {
            "active" => MembershipStatus::Active,
            "past_due" => MembershipStatus::PastDue,
            "canceled" => MembershipStatus::Canceled,
            "grace_period" => MembershipStatus::GracePeriod,
            _ => MembershipStatus::None,
        }
    }
}

/// User database model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub email_verified: bool,
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
    pub role: String,
    pub stripe_customer_id: Option<String>,
    #[sqlx(rename = "subscription_status")]
    #[serde(rename = "membership_status")]
    pub membership_status: String,
    pub price_locked: bool,
    pub locked_price_id: Option<String>,
    pub locked_price_amount: Option<i32>,
    pub grace_period_start: Option<DateTime<Utc>>,
    pub grace_period_end: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl User {
    /// Get the user's role as enum
    pub fn role_enum(&self) -> UserRole {
        UserRole::from(self.role.as_str())
    }

    /// Get the user's membership status as enum
    pub fn membership_status_enum(&self) -> MembershipStatus {
        MembershipStatus::from(self.membership_status.as_str())
    }

    /// Check if user is admin
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    /// Check if user has active membership
    pub fn has_active_membership(&self) -> bool {
        self.membership_status_enum().has_access()
    }

    /// Check if user is soft deleted
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

/// Data for creating a new user
#[derive(Debug, Clone)]
pub struct CreateUser {
    pub email: String,
    pub password_hash: Option<String>,
    pub role: UserRole,
}

/// Public user response (no sensitive data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub email_verified: bool,
    pub role: String,
    pub membership_status: String,
    pub price_locked: bool,
    pub locked_price_amount: Option<i32>,
    pub grace_period_end: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            email_verified: user.email_verified,
            role: user.role,
            membership_status: user.membership_status,
            price_locked: user.price_locked,
            locked_price_amount: user.locked_price_amount,
            grace_period_end: user.grace_period_end,
            created_at: user.created_at,
            last_login_at: user.last_login_at,
        }
    }
}
