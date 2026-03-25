//! Membership and payment models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Stripe subscription status (kept as Stripe terminology since it's Stripe's API)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StripeSubscriptionStatus {
    Active,
    PastDue,
    Canceled,
    Trialing,
    Incomplete,
    IncompleteExpired,
    Unpaid,
    Paused,
}

impl StripeSubscriptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            StripeSubscriptionStatus::Active => "active",
            StripeSubscriptionStatus::PastDue => "past_due",
            StripeSubscriptionStatus::Canceled => "canceled",
            StripeSubscriptionStatus::Trialing => "trialing",
            StripeSubscriptionStatus::Incomplete => "incomplete",
            StripeSubscriptionStatus::IncompleteExpired => "incomplete_expired",
            StripeSubscriptionStatus::Unpaid => "unpaid",
            StripeSubscriptionStatus::Paused => "paused",
        }
    }
}

impl From<String> for StripeSubscriptionStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "active" => StripeSubscriptionStatus::Active,
            "past_due" => StripeSubscriptionStatus::PastDue,
            "canceled" => StripeSubscriptionStatus::Canceled,
            "trialing" => StripeSubscriptionStatus::Trialing,
            "incomplete" => StripeSubscriptionStatus::Incomplete,
            "incomplete_expired" => StripeSubscriptionStatus::IncompleteExpired,
            "unpaid" => StripeSubscriptionStatus::Unpaid,
            "paused" => StripeSubscriptionStatus::Paused,
            _ => StripeSubscriptionStatus::Incomplete,
        }
    }
}

/// Membership database model (maps to subscriptions table)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Membership {
    pub id: Uuid,
    pub user_id: Uuid,
    pub stripe_subscription_id: String,
    pub stripe_price_id: String,
    pub status: String,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub cancel_at_period_end: bool,
    pub canceled_at: Option<DateTime<Utc>>,
    pub amount: i32,
    pub currency: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Membership {
    /// Get status as enum
    pub fn status_enum(&self) -> StripeSubscriptionStatus {
        StripeSubscriptionStatus::from(self.status.clone())
    }

    /// Check if membership is active
    pub fn is_active(&self) -> bool {
        self.status == "active"
    }
}

/// Data for creating a new membership
#[derive(Debug, Clone)]
pub struct CreateMembership {
    pub user_id: Uuid,
    pub stripe_subscription_id: String,
    pub stripe_price_id: String,
    pub status: String,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub amount: i32,
    pub currency: String,
}

/// Membership response for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MembershipResponse {
    pub status: String,
    pub price_locked: bool,
    pub locked_price_amount: Option<i32>,
    pub current_period_end: Option<DateTime<Utc>>,
    pub cancel_at_period_end: bool,
    pub grace_period_end: Option<DateTime<Utc>>,
}

/// Admin membership response (includes user email and tier from users table)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AdminMembershipResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_email: String,
    pub stripe_subscription_id: String,
    pub status: String,
    pub tier: String,
    pub amount: i32,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub cancel_at_period_end: bool,
    pub created_at: DateTime<Utc>,
}

/// Payment status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Succeeded,
    Failed,
    Pending,
    Refunded,
}

impl PaymentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentStatus::Succeeded => "succeeded",
            PaymentStatus::Failed => "failed",
            PaymentStatus::Pending => "pending",
            PaymentStatus::Refunded => "refunded",
        }
    }
}

impl From<String> for PaymentStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "succeeded" => PaymentStatus::Succeeded,
            "failed" => PaymentStatus::Failed,
            "pending" => PaymentStatus::Pending,
            "refunded" => PaymentStatus::Refunded,
            _ => PaymentStatus::Pending,
        }
    }
}

/// Payment history database model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentHistory {
    pub id: Uuid,
    pub user_id: Uuid,
    pub subscription_id: Option<Uuid>,
    pub stripe_payment_intent_id: Option<String>,
    pub stripe_invoice_id: Option<String>,
    pub amount: i32,
    pub currency: String,
    pub status: String,
    pub failure_reason: Option<String>,
    pub refunded_at: Option<DateTime<Utc>>,
    pub refund_amount: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// Data for creating a new payment record
#[derive(Debug, Clone)]
pub struct CreatePayment {
    pub user_id: Uuid,
    pub subscription_id: Option<Uuid>,
    pub stripe_payment_intent_id: Option<String>,
    pub stripe_invoice_id: Option<String>,
    pub amount: i32,
    pub currency: String,
    pub status: PaymentStatus,
    pub failure_reason: Option<String>,
}

/// Payment response for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResponse {
    pub id: Uuid,
    pub amount: i32,
    pub currency: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl From<PaymentHistory> for PaymentResponse {
    fn from(payment: PaymentHistory) -> Self {
        Self {
            id: payment.id,
            amount: payment.amount,
            currency: payment.currency,
            status: payment.status,
            created_at: payment.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    // -- StripeSubscriptionStatus --

    #[test]
    fn stripe_status_as_str() {
        assert_eq!(StripeSubscriptionStatus::Active.as_str(), "active");
        assert_eq!(StripeSubscriptionStatus::PastDue.as_str(), "past_due");
        assert_eq!(StripeSubscriptionStatus::Canceled.as_str(), "canceled");
        assert_eq!(StripeSubscriptionStatus::Trialing.as_str(), "trialing");
        assert_eq!(StripeSubscriptionStatus::IncompleteExpired.as_str(), "incomplete_expired");
        assert_eq!(StripeSubscriptionStatus::Unpaid.as_str(), "unpaid");
        assert_eq!(StripeSubscriptionStatus::Paused.as_str(), "paused");
    }

    #[test]
    fn stripe_status_from_string() {
        assert_eq!(StripeSubscriptionStatus::from("active".to_string()), StripeSubscriptionStatus::Active);
        assert_eq!(StripeSubscriptionStatus::from("past_due".to_string()), StripeSubscriptionStatus::PastDue);
        assert_eq!(StripeSubscriptionStatus::from("unknown".to_string()), StripeSubscriptionStatus::Incomplete);
    }

    // -- PaymentStatus --

    #[test]
    fn payment_status_as_str() {
        assert_eq!(PaymentStatus::Succeeded.as_str(), "succeeded");
        assert_eq!(PaymentStatus::Failed.as_str(), "failed");
        assert_eq!(PaymentStatus::Pending.as_str(), "pending");
        assert_eq!(PaymentStatus::Refunded.as_str(), "refunded");
    }

    #[test]
    fn payment_status_from_string() {
        assert_eq!(PaymentStatus::from("succeeded".to_string()), PaymentStatus::Succeeded);
        assert_eq!(PaymentStatus::from("failed".to_string()), PaymentStatus::Failed);
        assert_eq!(PaymentStatus::from("unknown".to_string()), PaymentStatus::Pending);
    }

    // -- Membership --

    #[test]
    fn membership_is_active() {
        let m = Membership {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            stripe_subscription_id: "sub_123".to_string(),
            stripe_price_id: "price_123".to_string(),
            status: "active".to_string(),
            current_period_start: Utc::now(),
            current_period_end: Utc::now(),
            cancel_at_period_end: false,
            canceled_at: None,
            amount: 300,
            currency: "usd".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert!(m.is_active());
        assert_eq!(m.status_enum(), StripeSubscriptionStatus::Active);
    }

    #[test]
    fn membership_not_active() {
        let m = Membership {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            stripe_subscription_id: "sub_123".to_string(),
            stripe_price_id: "price_123".to_string(),
            status: "canceled".to_string(),
            current_period_start: Utc::now(),
            current_period_end: Utc::now(),
            cancel_at_period_end: false,
            canceled_at: Some(Utc::now()),
            amount: 300,
            currency: "usd".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert!(!m.is_active());
    }

    // -- PaymentResponse from PaymentHistory --

    #[test]
    fn payment_response_from_history() {
        let payment = PaymentHistory {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            subscription_id: None,
            stripe_payment_intent_id: None,
            stripe_invoice_id: None,
            amount: 300,
            currency: "usd".to_string(),
            status: "succeeded".to_string(),
            failure_reason: None,
            refunded_at: None,
            refund_amount: None,
            created_at: Utc::now(),
        };
        let id = payment.id;
        let response = PaymentResponse::from(payment);
        assert_eq!(response.id, id);
        assert_eq!(response.amount, 300);
        assert_eq!(response.currency, "usd");
    }
}
