//! Stripe payment service (placeholder)

use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Subscription tier enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionTier {
    Personal,
    Business,
}

impl SubscriptionTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            SubscriptionTier::Personal => "personal",
            SubscriptionTier::Business => "business",
        }
    }

    /// Get the amount in cents for this tier
    pub fn amount_cents(&self) -> i32 {
        match self {
            SubscriptionTier::Personal => 300,   // $3.00
            SubscriptionTier::Business => 1500,  // $15.00
        }
    }
}

impl Default for SubscriptionTier {
    fn default() -> Self {
        SubscriptionTier::Personal
    }
}

/// Stripe configuration
#[derive(Clone)]
pub struct StripeConfig {
    pub secret_key: String,
    pub webhook_secret: String,
    pub personal_price_id: String,
    pub business_price_id: String,
    pub success_url: String,
    pub cancel_url: String,
}

impl StripeConfig {
    pub fn from_env() -> Result<Self, AppError> {
        Ok(Self {
            secret_key: std::env::var("STRIPE_SECRET_KEY")
                .unwrap_or_else(|_| "sk_test_placeholder".to_string()),
            webhook_secret: std::env::var("STRIPE_WEBHOOK_SECRET")
                .unwrap_or_else(|_| "whsec_placeholder".to_string()),
            personal_price_id: std::env::var("STRIPE_PRICE_ID")
                .unwrap_or_else(|_| "price_personal_placeholder".to_string()),
            business_price_id: std::env::var("STRIPE_BUSINESS_PRICE_ID")
                .unwrap_or_else(|_| "price_business_placeholder".to_string()),
            success_url: std::env::var("STRIPE_SUCCESS_URL")
                .unwrap_or_else(|_| "https://app.a8n.tools/dashboard?checkout=success".to_string()),
            cancel_url: std::env::var("STRIPE_CANCEL_URL")
                .unwrap_or_else(|_| "https://app.a8n.tools/pricing?checkout=canceled".to_string()),
        })
    }

    /// Get the price ID for a given tier
    pub fn price_id_for_tier(&self, tier: SubscriptionTier) -> &str {
        match tier {
            SubscriptionTier::Personal => &self.personal_price_id,
            SubscriptionTier::Business => &self.business_price_id,
        }
    }
}

/// Stripe service for payment operations
#[derive(Clone)]
pub struct StripeService {
    config: StripeConfig,
}

impl StripeService {
    pub fn new(config: StripeConfig) -> Self {
        Self { config }
    }

    /// Create a Stripe customer
    pub async fn create_customer(
        &self,
        email: &str,
        _user_id: Uuid,
    ) -> Result<String, AppError> {
        // TODO: Implement actual Stripe API call
        // For now, return a mock customer ID
        tracing::info!(email = %email, "Would create Stripe customer");
        Ok(format!("cus_mock_{}", Uuid::new_v4().as_simple()))
    }

    /// Create a checkout session for a specific tier
    pub async fn create_checkout_session(
        &self,
        customer_id: &str,
        user_id: Uuid,
        tier: SubscriptionTier,
    ) -> Result<(String, String), AppError> {
        // TODO: Implement actual Stripe API call
        // Returns (session_id, checkout_url)
        let price_id = self.config.price_id_for_tier(tier);
        tracing::info!(
            customer_id = %customer_id,
            user_id = %user_id,
            tier = %tier.as_str(),
            price_id = %price_id,
            "Would create Stripe checkout session"
        );

        let session_id = format!("cs_mock_{}", Uuid::new_v4().as_simple());
        let checkout_url = format!("https://checkout.stripe.com/mock/{}", session_id);

        Ok((session_id, checkout_url))
    }

    /// Get the tier from a price ID
    pub fn tier_from_price_id(&self, price_id: &str) -> SubscriptionTier {
        if price_id == self.config.business_price_id {
            SubscriptionTier::Business
        } else {
            SubscriptionTier::Personal
        }
    }

    /// Cancel a subscription
    pub async fn cancel_subscription(
        &self,
        subscription_id: &str,
        at_period_end: bool,
    ) -> Result<(), AppError> {
        tracing::info!(
            subscription_id = %subscription_id,
            at_period_end = at_period_end,
            "Would cancel Stripe subscription"
        );
        Ok(())
    }

    /// Reactivate a subscription (remove cancel at period end)
    pub async fn reactivate_subscription(&self, subscription_id: &str) -> Result<(), AppError> {
        tracing::info!(
            subscription_id = %subscription_id,
            "Would reactivate Stripe subscription"
        );
        Ok(())
    }

    /// Create a billing portal session
    pub async fn create_billing_portal_session(
        &self,
        customer_id: &str,
    ) -> Result<String, AppError> {
        tracing::info!(
            customer_id = %customer_id,
            "Would create Stripe billing portal session"
        );
        Ok("https://billing.stripe.com/mock/portal".to_string())
    }

    /// Verify webhook signature
    pub fn verify_webhook_signature(
        &self,
        _payload: &[u8],
        _signature: &str,
    ) -> Result<(), AppError> {
        // TODO: Implement actual signature verification
        Ok(())
    }

    /// Get the configured personal price ID (for backwards compatibility)
    pub fn price_id(&self) -> &str {
        &self.config.personal_price_id
    }

    /// Get the configured business price ID
    pub fn business_price_id(&self) -> &str {
        &self.config.business_price_id
    }
}
