//! User model and related types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// The account vocabulary (roles, membership status, subscription tier) lives in
// the shared `dunite-user-core` crate (DEV-517) and is re-exported here, so
// `crate::models::user::*` paths are unchanged and a8n-tools and bunyip cannot
// drift on the string values. The `User` row struct below stays a8n-side: the
// two schemas have not been reconciled.
pub use dunite_user_core::{MembershipStatus, SubscriptionTier, UserRole};

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
    pub stripe_payment_method_id: Option<String>,
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
    pub two_factor_enabled: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    /// Tier assigned at email verification: 'lifetime', 'early_adopter', 'standard'
    pub subscription_tier: String,
    /// Null for lifetime members; set for trial members
    pub trial_ends_at: Option<DateTime<Utc>>,
    /// True for the first 20 verified users and admin-granted lifetime members
    pub lifetime_member: bool,
    /// Set when an admin manually granted lifetime membership
    pub subscription_override_by: Option<Uuid>,
    /// DEV-525: the "first setup account" flag. Backfilled onto the
    /// earliest-created admin by the migration and set on the first admin of a
    /// fresh install by [`crate::repositories::UserRepository::ensure_super_admin`].
    /// Gates the admin actions an ordinary admin should not be able to take
    /// alone: impersonation, role changes, password resets and lifetime grants.
    pub is_super_admin: bool,
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

    /// Get the user's subscription tier as enum
    pub fn subscription_tier_enum(&self) -> SubscriptionTier {
        SubscriptionTier::from(self.subscription_tier.as_str())
    }

    /// Check if the user is allowed to access protected features.
    ///
    /// Access is granted when ANY of the following are true:
    /// - User is an admin (admins bypass all access checks)
    /// - User is a lifetime member
    /// - User's trial has not yet expired
    /// - User has an active/grace-period Stripe subscription
    pub fn is_access_allowed(&self) -> bool {
        if self.is_admin() {
            return true;
        }
        if self.lifetime_member {
            return true;
        }
        if let Some(trial_ends_at) = self.trial_ends_at {
            if trial_ends_at > chrono::Utc::now() {
                return true;
            }
        }
        self.membership_status_enum().has_access()
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
    pub two_factor_enabled: bool,
    pub grace_period_end: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub subscription_tier: String,
    pub trial_ends_at: Option<DateTime<Utc>>,
    pub lifetime_member: bool,
    /// DEV-525: see [`User::is_super_admin`]. Surfaced so the admin UI can
    /// render (or hide) the controls only the super admin may use, rather than
    /// offering a button that the API will then refuse.
    pub is_super_admin: bool,
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
            two_factor_enabled: user.two_factor_enabled,
            grace_period_end: user.grace_period_end,
            created_at: user.created_at,
            last_login_at: user.last_login_at,
            subscription_tier: user.subscription_tier,
            trial_ends_at: user.trial_ends_at,
            lifetime_member: user.lifetime_member,
            is_super_admin: user.is_super_admin,
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
            stripe_payment_method_id: None,
            membership_status: "active".to_string(),
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
            subscription_tier: "standard".to_string(),
            trial_ends_at: None,
            lifetime_member: false,
            subscription_override_by: None,
            is_super_admin: false,
        }
    }

    // -- UserRole --

    // -- MembershipStatus --

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

    /// DEV-525: the admin UI decides whether to render the super-admin-only
    /// controls from this field, so it has to survive the conversion. It also
    /// must not default to `true` for an ordinary account.
    #[test]
    fn user_response_carries_the_super_admin_flag() {
        let ordinary = test_user();
        assert!(!UserResponse::from(ordinary).is_super_admin);

        let mut super_admin = test_user();
        super_admin.role = "admin".to_string();
        super_admin.is_super_admin = true;
        assert!(UserResponse::from(super_admin).is_super_admin);
    }

    // -- SubscriptionTier --

    fn user_with_tier(
        lifetime_member: bool,
        trial_ends_at: Option<DateTime<Utc>>,
        subscription_tier: &str,
    ) -> User {
        let mut user = test_user();
        user.membership_status = "none".to_string();
        user.lifetime_member = lifetime_member;
        user.trial_ends_at = trial_ends_at;
        user.subscription_tier = subscription_tier.to_string();
        user
    }

    // -- is_access_allowed --

    #[test]
    fn access_allowed_for_admin() {
        let mut user = test_user();
        user.role = "admin".to_string();
        user.membership_status = "none".to_string();
        user.lifetime_member = false;
        user.trial_ends_at = None;
        assert!(user.is_access_allowed());
    }

    #[test]
    fn access_allowed_for_lifetime_member() {
        let user = user_with_tier(true, None, "lifetime");
        assert!(user.is_access_allowed());
    }

    #[test]
    fn access_allowed_for_free_member() {
        let user = user_with_tier(true, None, "free");
        assert!(user.is_access_allowed());
    }

    #[test]
    fn access_allowed_for_active_trial() {
        let future = Utc::now() + chrono::Duration::days(10);
        let user = user_with_tier(false, Some(future), "standard");
        assert!(user.is_access_allowed());
    }

    #[test]
    fn access_denied_for_expired_trial() {
        let past = Utc::now() - chrono::Duration::days(1);
        let user = user_with_tier(false, Some(past), "standard");
        assert!(!user.is_access_allowed());
    }

    #[test]
    fn access_allowed_for_active_stripe_subscription() {
        let mut user = user_with_tier(false, None, "standard");
        user.membership_status = "active".to_string();
        assert!(user.is_access_allowed());
    }

    #[test]
    fn access_denied_for_no_membership_no_trial() {
        let user = user_with_tier(false, None, "standard");
        assert!(!user.is_access_allowed());
    }

    // -- Tier assignment logic (mirrors auth service) --
    // Tiers are assigned based on per-tier counts, not total user count.
    // This ensures slots fill correctly even if users existed before the tier system.

    fn tier_for_counts(
        lifetime_count: i64,
        early_adopter_count: i64,
        lifetime_slots: i64,
        early_adopter_slots: i64,
    ) -> SubscriptionTier {
        if lifetime_count < lifetime_slots {
            SubscriptionTier::Lifetime
        } else if early_adopter_count < early_adopter_slots {
            SubscriptionTier::EarlyAdopter
        } else {
            SubscriptionTier::Standard
        }
    }

    // Default thresholds: 5 lifetime slots, 5 early adopter slots

    #[test]
    fn tier_assignment_lifetime_slots_available() {
        // 4 lifetime assigned, slot still open
        assert_eq!(tier_for_counts(4, 0, 5, 5), SubscriptionTier::Lifetime);
    }

    #[test]
    fn tier_assignment_lifetime_slots_full() {
        // 5 lifetime assigned, falls through to early adopter
        assert_eq!(tier_for_counts(5, 0, 5, 5), SubscriptionTier::EarlyAdopter);
    }

    #[test]
    fn tier_assignment_early_adopter_slots_filling() {
        // lifetime full, 4 early adopter assigned
        assert_eq!(tier_for_counts(5, 4, 5, 5), SubscriptionTier::EarlyAdopter);
    }

    #[test]
    fn tier_assignment_all_slots_full() {
        // both tiers full
        assert_eq!(tier_for_counts(5, 5, 5, 5), SubscriptionTier::Standard);
    }

    #[test]
    fn tier_assignment_custom_thresholds() {
        // 3 lifetime slots, 7 early adopter slots
        assert_eq!(tier_for_counts(2, 0, 3, 7), SubscriptionTier::Lifetime);
        assert_eq!(tier_for_counts(3, 0, 3, 7), SubscriptionTier::EarlyAdopter);
        assert_eq!(tier_for_counts(3, 6, 3, 7), SubscriptionTier::EarlyAdopter);
        assert_eq!(tier_for_counts(3, 7, 3, 7), SubscriptionTier::Standard);
    }

    #[test]
    fn tier_assignment_existing_users_dont_consume_slots() {
        // 100 standard users exist but 0 lifetime assigned — lifetime still available
        assert_eq!(tier_for_counts(0, 0, 5, 5), SubscriptionTier::Lifetime);
    }
}
