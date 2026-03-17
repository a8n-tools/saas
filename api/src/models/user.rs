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

/// Membership tier for users
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipTier {
    Personal,
    Business,
}

impl MembershipTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            MembershipTier::Personal => "personal",
            MembershipTier::Business => "business",
        }
    }
}

impl From<String> for MembershipTier {
    fn from(s: String) -> Self {
        match s.as_str() {
            "business" => MembershipTier::Business,
            _ => MembershipTier::Personal,
        }
    }
}

impl From<&str> for MembershipTier {
    fn from(s: &str) -> Self {
        match s {
            "business" => MembershipTier::Business,
            _ => MembershipTier::Personal,
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
    pub membership_tier: Option<String>,
    pub price_locked: bool,
    pub locked_price_id: Option<String>,
    pub locked_price_amount: Option<i32>,
    pub grace_period_start: Option<DateTime<Utc>>,
    pub grace_period_end: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub two_factor_enabled: bool,
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

    /// Get the user's membership tier as enum
    pub fn membership_tier_enum(&self) -> MembershipTier {
        self.membership_tier
            .as_ref()
            .map(|t| MembershipTier::from(t.as_str()))
            .unwrap_or(MembershipTier::Personal)
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
    pub membership_tier: Option<String>,
    pub price_locked: bool,
    pub locked_price_amount: Option<i32>,
    pub two_factor_enabled: bool,
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
            membership_tier: user.membership_tier,
            price_locked: user.price_locked,
            locked_price_amount: user.locked_price_amount,
            two_factor_enabled: user.two_factor_enabled,
            grace_period_end: user.grace_period_end,
            created_at: user.created_at,
            last_login_at: user.last_login_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn test_user() -> User {
        User {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            email_verified: true,
            password_hash: Some("hash".to_string()),
            role: "subscriber".to_string(),
            stripe_customer_id: None,
            membership_status: "active".to_string(),
            membership_tier: Some("personal".to_string()),
            price_locked: false,
            locked_price_id: None,
            locked_price_amount: None,
            grace_period_start: None,
            grace_period_end: None,
            two_factor_enabled: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_login_at: None,
            deleted_at: None,
        }
    }

    // -- UserRole --

    #[test]
    fn user_role_as_str() {
        assert_eq!(UserRole::Subscriber.as_str(), "subscriber");
        assert_eq!(UserRole::Admin.as_str(), "admin");
    }

    #[test]
    fn user_role_from_string() {
        assert_eq!(UserRole::from("admin".to_string()), UserRole::Admin);
        assert_eq!(UserRole::from("subscriber".to_string()), UserRole::Subscriber);
        assert_eq!(UserRole::from("unknown".to_string()), UserRole::Subscriber);
    }

    #[test]
    fn user_role_from_str() {
        assert_eq!(UserRole::from("admin"), UserRole::Admin);
        assert_eq!(UserRole::from("anything"), UserRole::Subscriber);
    }

    // -- MembershipStatus --

    #[test]
    fn membership_status_as_str() {
        assert_eq!(MembershipStatus::None.as_str(), "none");
        assert_eq!(MembershipStatus::Active.as_str(), "active");
        assert_eq!(MembershipStatus::PastDue.as_str(), "past_due");
        assert_eq!(MembershipStatus::Canceled.as_str(), "canceled");
        assert_eq!(MembershipStatus::GracePeriod.as_str(), "grace_period");
    }

    #[test]
    fn membership_status_has_access() {
        assert!(MembershipStatus::Active.has_access());
        assert!(MembershipStatus::GracePeriod.has_access());
        assert!(!MembershipStatus::None.has_access());
        assert!(!MembershipStatus::PastDue.has_access());
        assert!(!MembershipStatus::Canceled.has_access());
    }

    #[test]
    fn membership_status_from_string() {
        assert_eq!(MembershipStatus::from("active".to_string()), MembershipStatus::Active);
        assert_eq!(MembershipStatus::from("past_due".to_string()), MembershipStatus::PastDue);
        assert_eq!(MembershipStatus::from("canceled".to_string()), MembershipStatus::Canceled);
        assert_eq!(MembershipStatus::from("grace_period".to_string()), MembershipStatus::GracePeriod);
        assert_eq!(MembershipStatus::from("unknown".to_string()), MembershipStatus::None);
    }

    // -- MembershipTier --

    #[test]
    fn membership_tier_as_str() {
        assert_eq!(MembershipTier::Personal.as_str(), "personal");
        assert_eq!(MembershipTier::Business.as_str(), "business");
    }

    #[test]
    fn membership_tier_from_string() {
        assert_eq!(MembershipTier::from("business".to_string()), MembershipTier::Business);
        assert_eq!(MembershipTier::from("personal".to_string()), MembershipTier::Personal);
        assert_eq!(MembershipTier::from("unknown".to_string()), MembershipTier::Personal);
    }

    // -- User methods --

    #[test]
    fn user_role_enum() {
        let user = test_user();
        assert_eq!(user.role_enum(), UserRole::Subscriber);

        let mut admin = test_user();
        admin.role = "admin".to_string();
        assert_eq!(admin.role_enum(), UserRole::Admin);
    }

    #[test]
    fn user_membership_status_enum() {
        let user = test_user();
        assert_eq!(user.membership_status_enum(), MembershipStatus::Active);
    }

    #[test]
    fn user_membership_tier_enum() {
        let user = test_user();
        assert_eq!(user.membership_tier_enum(), MembershipTier::Personal);

        let mut biz = test_user();
        biz.membership_tier = Some("business".to_string());
        assert_eq!(biz.membership_tier_enum(), MembershipTier::Business);

        let mut none = test_user();
        none.membership_tier = None;
        assert_eq!(none.membership_tier_enum(), MembershipTier::Personal);
    }

    #[test]
    fn user_is_admin() {
        let user = test_user();
        assert!(!user.is_admin());

        let mut admin = test_user();
        admin.role = "admin".to_string();
        assert!(admin.is_admin());
    }

    #[test]
    fn user_has_active_membership() {
        let user = test_user();
        assert!(user.has_active_membership()); // "active"

        let mut canceled = test_user();
        canceled.membership_status = "canceled".to_string();
        assert!(!canceled.has_active_membership());

        let mut grace = test_user();
        grace.membership_status = "grace_period".to_string();
        assert!(grace.has_active_membership());
    }

    #[test]
    fn user_is_deleted() {
        let user = test_user();
        assert!(!user.is_deleted());

        let mut deleted = test_user();
        deleted.deleted_at = Some(Utc::now());
        assert!(deleted.is_deleted());
    }

    #[test]
    fn user_response_from_user() {
        let user = test_user();
        let id = user.id;
        let response = UserResponse::from(user);
        assert_eq!(response.id, id);
        assert_eq!(response.email, "test@example.com");
        assert_eq!(response.role, "subscriber");
    }
}
