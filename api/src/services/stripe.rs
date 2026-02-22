//! Stripe payment service

use crate::errors::AppError;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

/// Subscription tier enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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

    /// Get the amount in cents for this tier
    pub fn amount_cents(&self) -> i32 {
        match self {
            MembershipTier::Personal => 300,   // $3.00
            MembershipTier::Business => 1500,  // $15.00
        }
    }
}

impl Default for MembershipTier {
    fn default() -> Self {
        MembershipTier::Personal
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
                .unwrap_or_else(|_| "http://localhost:5173/dashboard?checkout=success".to_string()),
            cancel_url: std::env::var("STRIPE_CANCEL_URL")
                .unwrap_or_else(|_| "http://localhost:5173/pricing?checkout=canceled".to_string()),
        })
    }

    /// Get the price ID for a given tier
    pub fn price_id_for_tier(&self, tier: MembershipTier) -> &str {
        match tier {
            MembershipTier::Personal => &self.personal_price_id,
            MembershipTier::Business => &self.business_price_id,
        }
    }
}

/// Stripe service for payment operations
pub struct StripeService {
    config: StripeConfig,
    client: Arc<stripe::Client>,
}

impl Clone for StripeService {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            client: self.client.clone(),
        }
    }
}

impl StripeService {
    pub fn new(config: StripeConfig) -> Self {
        let client = stripe::Client::new(&config.secret_key);
        Self {
            config,
            client: Arc::new(client),
        }
    }

    /// Create a Stripe customer linked to a platform user
    pub async fn create_customer(
        &self,
        email: &str,
        user_id: Uuid,
    ) -> Result<String, AppError> {
        let mut metadata = HashMap::new();
        metadata.insert("user_id".to_string(), user_id.to_string());

        let params = stripe::CreateCustomer {
            email: Some(email),
            metadata: Some(metadata),
            ..Default::default()
        };

        let customer = stripe::Customer::create(&self.client, params)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, email = %email, "Failed to create Stripe customer");
                AppError::internal("Failed to create payment customer")
            })?;

        tracing::info!(
            customer_id = %customer.id,
            user_id = %user_id,
            "Created Stripe customer"
        );

        Ok(customer.id.to_string())
    }

    /// Create a checkout session for a specific membership tier
    pub async fn create_checkout_session(
        &self,
        customer_id: &str,
        user_id: Uuid,
        tier: MembershipTier,
    ) -> Result<(String, String), AppError> {
        let price_id = self.config.price_id_for_tier(tier);

        let mut metadata = HashMap::new();
        metadata.insert("user_id".to_string(), user_id.to_string());
        metadata.insert("tier".to_string(), tier.as_str().to_string());

        let customer_id: stripe::CustomerId = customer_id.parse()
            .map_err(|_| {
                tracing::error!(customer_id = %customer_id, "Invalid Stripe customer ID format");
                AppError::internal("Invalid customer ID")
            })?;

        let params = stripe::CreateCheckoutSession {
            mode: Some(stripe::CheckoutSessionMode::Subscription),
            customer: Some(customer_id),
            line_items: Some(vec![stripe::CreateCheckoutSessionLineItems {
                price: Some(price_id.to_string()),
                quantity: Some(1),
                ..Default::default()
            }]),
            success_url: Some(&self.config.success_url),
            cancel_url: Some(&self.config.cancel_url),
            metadata: Some(metadata.clone()),
            subscription_data: Some(stripe::CreateCheckoutSessionSubscriptionData {
                metadata: Some(metadata),
                ..Default::default()
            }),
            ..Default::default()
        };

        let session = stripe::CheckoutSession::create(&self.client, params)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to create Stripe checkout session");
                AppError::internal("Failed to create checkout session")
            })?;

        let session_id = session.id.to_string();
        let checkout_url = session.url
            .ok_or_else(|| AppError::internal("Checkout session missing URL"))?;

        tracing::info!(
            session_id = %session_id,
            tier = %tier.as_str(),
            "Created Stripe checkout session"
        );

        Ok((session_id, checkout_url))
    }

    /// Get the tier from a price ID
    pub fn tier_from_price_id(&self, price_id: &str) -> MembershipTier {
        if price_id == self.config.business_price_id {
            MembershipTier::Business
        } else {
            MembershipTier::Personal
        }
    }

    /// Cancel a subscription (at period end or immediately)
    pub async fn cancel_subscription(
        &self,
        subscription_id: &str,
        at_period_end: bool,
    ) -> Result<(), AppError> {
        let sub_id: stripe::SubscriptionId = subscription_id.parse()
            .map_err(|_| {
                tracing::error!(subscription_id = %subscription_id, "Invalid subscription ID format");
                AppError::internal("Invalid subscription ID")
            })?;

        if at_period_end {
            // Cancel at end of current billing period
            let params = stripe::UpdateSubscription {
                cancel_at_period_end: Some(true),
                ..Default::default()
            };
            stripe::Subscription::update(&self.client, &sub_id, params)
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, "Failed to schedule subscription cancellation");
                    AppError::internal("Failed to cancel subscription")
                })?;
        } else {
            // Cancel immediately
            stripe::Subscription::cancel(
                &self.client,
                &sub_id,
                stripe::CancelSubscription::default(),
            )
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to cancel subscription immediately");
                AppError::internal("Failed to cancel subscription")
            })?;
        }

        tracing::info!(
            subscription_id = %subscription_id,
            at_period_end = at_period_end,
            "Canceled Stripe subscription"
        );

        Ok(())
    }

    /// Reactivate a subscription (remove cancel_at_period_end flag)
    pub async fn reactivate_subscription(&self, subscription_id: &str) -> Result<(), AppError> {
        let sub_id: stripe::SubscriptionId = subscription_id.parse()
            .map_err(|_| {
                tracing::error!(subscription_id = %subscription_id, "Invalid subscription ID format");
                AppError::internal("Invalid subscription ID")
            })?;

        let params = stripe::UpdateSubscription {
            cancel_at_period_end: Some(false),
            ..Default::default()
        };

        stripe::Subscription::update(&self.client, &sub_id, params)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to reactivate subscription");
                AppError::internal("Failed to reactivate subscription")
            })?;

        tracing::info!(subscription_id = %subscription_id, "Reactivated Stripe subscription");

        Ok(())
    }

    /// Create a Stripe billing portal session for self-service management
    pub async fn create_billing_portal_session(
        &self,
        customer_id: &str,
    ) -> Result<String, AppError> {
        let customer_id: stripe::CustomerId = customer_id.parse()
            .map_err(|_| {
                tracing::error!(customer_id = %customer_id, "Invalid customer ID format");
                AppError::internal("Invalid customer ID")
            })?;

        let mut params = stripe::CreateBillingPortalSession::new(customer_id);
        params.return_url = Some(&self.config.success_url);

        let session = stripe::BillingPortalSession::create(&self.client, params)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to create billing portal session");
                AppError::internal("Failed to create billing portal session")
            })?;

        Ok(session.url)
    }

    /// Verify Stripe webhook signature (HMAC-SHA256)
    pub fn verify_webhook_signature(
        &self,
        payload: &[u8],
        signature: &str,
    ) -> Result<(), AppError> {
        // Parse the Stripe-Signature header: t=timestamp,v1=sig1,v1=sig2,...
        let mut timestamp = None;
        let mut signatures = Vec::new();

        for part in signature.split(',') {
            if let Some((key, value)) = part.split_once('=') {
                match key.trim() {
                    "t" => timestamp = Some(value.trim()),
                    "v1" => signatures.push(value.trim().to_string()),
                    _ => {}
                }
            }
        }

        let timestamp = timestamp
            .ok_or_else(|| AppError::validation("signature", "Missing timestamp in webhook signature"))?;

        if signatures.is_empty() {
            return Err(AppError::validation("signature", "No v1 signature found"));
        }

        // Verify timestamp is within tolerance (5 minutes)
        let ts: i64 = timestamp.parse()
            .map_err(|_| AppError::validation("signature", "Invalid timestamp"))?;
        let now = chrono::Utc::now().timestamp();
        if (now - ts).abs() > 300 {
            tracing::warn!(
                timestamp = ts,
                now = now,
                "Webhook timestamp outside tolerance window"
            );
            return Err(AppError::validation("signature", "Webhook timestamp too old"));
        }

        // Build signed payload: "{timestamp}.{payload}"
        let payload_str = std::str::from_utf8(payload)
            .map_err(|_| AppError::validation("body", "Invalid UTF-8 in webhook payload"))?;
        let signed_payload = format!("{}.{}", timestamp, payload_str);

        // Compute expected HMAC-SHA256 signature
        let mut mac = HmacSha256::new_from_slice(self.config.webhook_secret.as_bytes())
            .map_err(|_| AppError::internal("Invalid webhook secret key"))?;
        mac.update(signed_payload.as_bytes());
        let expected = hex::encode(mac.finalize().into_bytes());

        // Compare against all v1 signatures (Stripe may include multiple)
        if signatures.iter().any(|sig| sig == &expected) {
            Ok(())
        } else {
            tracing::warn!("Webhook signature verification failed");
            Err(AppError::Unauthorized)
        }
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
