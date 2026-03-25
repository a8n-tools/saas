//! Stripe payment service

use crate::errors::AppError;
use crate::models::stripe::decrypt_secret;
use crate::services::encryption::EncryptionKeySet;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
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
        let frontend_origin = std::env::var("CORS_ORIGIN")
            .unwrap_or_else(|_| "http://localhost:5173".to_string());
        let base = frontend_origin.trim_end_matches('/');

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
                .unwrap_or_else(|_| format!("{base}/checkout/success")),
            cancel_url: std::env::var("STRIPE_CANCEL_URL")
                .unwrap_or_else(|_| format!("{base}/pricing?checkout=canceled")),
        })
    }

    /// Build a `StripeConfig` from the DB model, decrypting secrets.
    /// Falls back to env vars for any fields not set in the DB.
    pub fn from_db_model(
        db: &crate::models::stripe::StripeConfig,
        key_set: &EncryptionKeySet,
    ) -> Result<Self, AppError> {
        let env_config = Self::from_env()?;

        let secret_key = match (&db.secret_key, &db.secret_key_nonce) {
            (Some(ct), Some(nonce)) => decrypt_secret(key_set, ct, nonce, db.key_version)?,
            _ => env_config.secret_key,
        };
        let webhook_secret = match (&db.webhook_secret, &db.webhook_secret_nonce) {
            (Some(ct), Some(nonce)) => decrypt_secret(key_set, ct, nonce, db.key_version)?,
            _ => env_config.webhook_secret,
        };

        Ok(Self {
            secret_key,
            webhook_secret,
            personal_price_id: db
                .price_id_personal
                .clone()
                .unwrap_or(env_config.personal_price_id),
            business_price_id: db
                .price_id_business
                .clone()
                .unwrap_or(env_config.business_price_id),
            success_url: env_config.success_url,
            cancel_url: env_config.cancel_url,
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

/// Inner state that can be swapped when admin updates Stripe config.
struct StripeServiceInner {
    config: StripeConfig,
    client: Arc<stripe::Client>,
}

/// Stripe service for payment operations.
///
/// Uses `RwLock` internally so the config + client can be hot-reloaded
/// when an admin updates Stripe keys via the dashboard.
pub struct StripeService {
    inner: RwLock<StripeServiceInner>,
}

impl StripeService {
    pub fn new(config: StripeConfig) -> Self {
        let client = stripe::Client::new(&config.secret_key);
        Self {
            inner: RwLock::new(StripeServiceInner {
                config,
                client: Arc::new(client),
            }),
        }
    }

    /// Hot-reload the service with a new config (e.g. after admin update).
    /// Builds a new Stripe client with the updated secret key.
    pub fn reload(&self, config: StripeConfig) {
        let client = stripe::Client::new(&config.secret_key);
        let mut inner = self.inner.write().expect("StripeService lock poisoned");
        inner.config = config;
        inner.client = Arc::new(client);
    }

    /// Snapshot current config + client for use in an async operation.
    fn snapshot(&self) -> (StripeConfig, Arc<stripe::Client>) {
        let inner = self.inner.read().expect("StripeService lock poisoned");
        (inner.config.clone(), inner.client.clone())
    }

    /// Create a Stripe customer linked to a platform user
    pub async fn create_customer(
        &self,
        email: &str,
        user_id: Uuid,
    ) -> Result<String, AppError> {
        let (_config, client) = self.snapshot();

        let mut metadata = HashMap::new();
        metadata.insert("user_id".to_string(), user_id.to_string());

        let params = stripe::CreateCustomer {
            email: Some(email),
            metadata: Some(metadata),
            ..Default::default()
        };

        let customer = stripe::Customer::create(&client, params)
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
        let (config, client) = self.snapshot();
        let price_id = config.price_id_for_tier(tier);

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
            success_url: Some(&config.success_url),
            cancel_url: Some(&config.cancel_url),
            metadata: Some(metadata.clone()),
            subscription_data: Some(stripe::CreateCheckoutSessionSubscriptionData {
                metadata: Some(metadata),
                ..Default::default()
            }),
            ..Default::default()
        };

        let session = stripe::CheckoutSession::create(&client, params)
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
        let (config, _) = self.snapshot();
        if price_id == config.business_price_id {
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

        let (_config, client) = self.snapshot();

        if at_period_end {
            // Cancel at end of current billing period
            let params = stripe::UpdateSubscription {
                cancel_at_period_end: Some(true),
                ..Default::default()
            };
            stripe::Subscription::update(&client, &sub_id, params)
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, "Failed to schedule subscription cancellation");
                    AppError::internal("Failed to cancel subscription")
                })?;
        } else {
            // Cancel immediately
            stripe::Subscription::cancel(
                &client,
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
        let (_config, client) = self.snapshot();

        let sub_id: stripe::SubscriptionId = subscription_id.parse()
            .map_err(|_| {
                tracing::error!(subscription_id = %subscription_id, "Invalid subscription ID format");
                AppError::internal("Invalid subscription ID")
            })?;

        let params = stripe::UpdateSubscription {
            cancel_at_period_end: Some(false),
            ..Default::default()
        };

        stripe::Subscription::update(&client, &sub_id, params)
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
        let (config, client) = self.snapshot();

        let customer_id: stripe::CustomerId = customer_id.parse()
            .map_err(|_| {
                tracing::error!(customer_id = %customer_id, "Invalid customer ID format");
                AppError::internal("Invalid customer ID")
            })?;

        let mut params = stripe::CreateBillingPortalSession::new(customer_id);
        params.return_url = Some(&config.success_url);

        let session = stripe::BillingPortalSession::create(&client, params)
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
        let (config, _) = self.snapshot();
        let mut mac = HmacSha256::new_from_slice(config.webhook_secret.as_bytes())
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

    /// Create a Stripe Customer and a SetupIntent for $0 card authorization at signup.
    ///
    /// Returns `(customer_id, client_secret)`. The caller passes the `client_secret`
    /// to the frontend so Stripe.js can confirm the setup, and passes the `customer_id`
    /// back to the register endpoint so it can be stored on the newly-created user.
    pub async fn create_setup_intent(&self, email: &str) -> Result<(String, String), AppError> {
        let (_config, client) = self.snapshot();

        let customer_params = stripe::CreateCustomer {
            email: Some(email),
            ..Default::default()
        };

        let customer = stripe::Customer::create(&client, customer_params)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, email = %email, "Failed to create Stripe customer for signup");
                AppError::internal("Failed to initialize payment")
            })?;

        let customer_id: stripe::CustomerId = customer.id.to_string().parse().map_err(|_| {
            AppError::internal("Invalid customer ID returned from Stripe")
        })?;

        let intent_params = stripe::CreateSetupIntent {
            customer: Some(customer_id),
            ..Default::default()
        };

        let setup_intent = stripe::SetupIntent::create(&client, intent_params)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to create SetupIntent for signup");
                AppError::internal("Failed to initialize payment")
            })?;

        let client_secret = setup_intent
            .client_secret
            .ok_or_else(|| AppError::internal("SetupIntent missing client_secret"))?;

        tracing::info!(
            customer_id = %customer.id,
            "Created Stripe customer and SetupIntent for signup"
        );

        Ok((customer.id.to_string(), client_secret))
    }

    /// Get the configured personal price ID (for backwards compatibility)
    pub fn price_id(&self) -> String {
        let (config, _) = self.snapshot();
        config.personal_price_id
    }

    /// Get the configured business price ID
    pub fn business_price_id(&self) -> String {
        let (config, _) = self.snapshot();
        config.business_price_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> StripeConfig {
        StripeConfig {
            secret_key: "sk_test_xxx".to_string(),
            webhook_secret: "whsec_test_secret".to_string(),
            personal_price_id: "price_personal".to_string(),
            business_price_id: "price_business".to_string(),
            success_url: "http://localhost/checkout/success".to_string(),
            cancel_url: "http://localhost/cancel".to_string(),
        }
    }

    fn test_service() -> StripeService {
        StripeService::new(test_config())
    }

    // -- MembershipTier --

    #[test]
    fn tier_as_str() {
        assert_eq!(MembershipTier::Personal.as_str(), "personal");
        assert_eq!(MembershipTier::Business.as_str(), "business");
    }

    #[test]
    fn tier_amount_cents() {
        assert_eq!(MembershipTier::Personal.amount_cents(), 300);
        assert_eq!(MembershipTier::Business.amount_cents(), 1500);
    }

    #[test]
    fn tier_default() {
        assert_eq!(MembershipTier::default(), MembershipTier::Personal);
    }

    // -- StripeConfig --

    #[test]
    fn price_id_for_tier() {
        let config = test_config();
        assert_eq!(config.price_id_for_tier(MembershipTier::Personal), "price_personal");
        assert_eq!(config.price_id_for_tier(MembershipTier::Business), "price_business");
    }

    // -- StripeService --

    #[test]
    fn tier_from_price_id_personal() {
        let service = test_service();
        assert_eq!(service.tier_from_price_id("price_personal"), MembershipTier::Personal);
    }

    #[test]
    fn tier_from_price_id_business() {
        let service = test_service();
        assert_eq!(service.tier_from_price_id("price_business"), MembershipTier::Business);
    }

    #[test]
    fn tier_from_unknown_price_defaults_to_personal() {
        let service = test_service();
        assert_eq!(service.tier_from_price_id("price_unknown"), MembershipTier::Personal);
    }

    #[test]
    fn price_id_accessors() {
        let service = test_service();
        assert_eq!(service.price_id(), "price_personal");
        assert_eq!(service.business_price_id(), "price_business");
    }

    // -- Webhook signature verification --

    #[test]
    fn verify_webhook_signature_valid() {
        let service = test_service();
        let payload = b"{\"type\":\"test\"}";
        let timestamp = chrono::Utc::now().timestamp().to_string();

        // Compute the expected signature
        let signed_payload = format!("{}.{}", timestamp, std::str::from_utf8(payload).unwrap());
        let mut mac = HmacSha256::new_from_slice(b"whsec_test_secret").unwrap();
        mac.update(signed_payload.as_bytes());
        let sig = hex::encode(mac.finalize().into_bytes());

        let header = format!("t={},v1={}", timestamp, sig);
        assert!(service.verify_webhook_signature(payload, &header).is_ok());
    }

    #[test]
    fn verify_webhook_signature_invalid() {
        let service = test_service();
        let payload = b"{\"type\":\"test\"}";
        let timestamp = chrono::Utc::now().timestamp().to_string();
        let header = format!("t={},v1=invalid_signature", timestamp);

        assert!(service.verify_webhook_signature(payload, &header).is_err());
    }

    #[test]
    fn verify_webhook_signature_missing_timestamp() {
        let service = test_service();
        let payload = b"{\"type\":\"test\"}";
        let header = "v1=some_signature";

        assert!(service.verify_webhook_signature(payload, header).is_err());
    }

    #[test]
    fn verify_webhook_signature_no_v1() {
        let service = test_service();
        let payload = b"{\"type\":\"test\"}";
        let header = "t=12345";

        assert!(service.verify_webhook_signature(payload, header).is_err());
    }

    #[test]
    fn verify_webhook_signature_old_timestamp() {
        let service = test_service();
        let payload = b"{\"type\":\"test\"}";
        // Use a timestamp from 10 minutes ago (beyond 5-minute tolerance)
        let old_ts = (chrono::Utc::now().timestamp() - 600).to_string();

        let signed_payload = format!("{}.{}", old_ts, std::str::from_utf8(payload).unwrap());
        let mut mac = HmacSha256::new_from_slice(b"whsec_test_secret").unwrap();
        mac.update(signed_payload.as_bytes());
        let sig = hex::encode(mac.finalize().into_bytes());

        let header = format!("t={},v1={}", old_ts, sig);
        assert!(service.verify_webhook_signature(payload, &header).is_err());
    }
}
