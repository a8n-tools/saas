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
