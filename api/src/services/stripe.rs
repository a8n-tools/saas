//! Stripe payment service
//!
//! All Stripe state is managed through the Stripe API. This service provides
//! proxy methods for products, prices, subscriptions, invoices, and webhook
//! endpoints. No local database tables are used for Stripe state.

use crate::errors::AppError;
use crate::models::stripe::{
    decrypt_secret, StripeInvoiceResponse, StripePriceResponse, StripeProductResponse,
    StripeSubscriptionItemResponse, StripeSubscriptionResponse, StripeWebhookEndpointResponse,
};
use crate::services::encryption::EncryptionKeySet;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

/// Subscription tier enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
            success_url: env_config.success_url,
            cancel_url: env_config.cancel_url,
        })
    }
}

/// Cached tier resolution entry
struct TierCacheEntry {
    tier: MembershipTier,
    cached_at: Instant,
}

/// Inner state that can be swapped when admin updates Stripe config.
struct StripeServiceInner {
    config: StripeConfig,
    client: Arc<stripe::Client>,
    /// Cache: price_id -> tier (resolved from product metadata)
    tier_cache: HashMap<String, TierCacheEntry>,
}

const TIER_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes

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
                tier_cache: HashMap::new(),
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
        inner.tier_cache.clear();
    }

    /// Snapshot current config + client for use in an async operation.
    fn snapshot(&self) -> (StripeConfig, Arc<stripe::Client>) {
        let inner = self.inner.read().expect("StripeService lock poisoned");
        (inner.config.clone(), inner.client.clone())
    }

    // ─── Products ────────────────────────────────────────────

    /// List all products from Stripe (active and inactive)
    pub async fn list_products(&self) -> Result<Vec<StripeProductResponse>, AppError> {
        let (_config, client) = self.snapshot();

        let mut params = stripe::ListProducts::new();
        params.limit = Some(100);

        let products = stripe::Product::list(&client, &params)
            .await
            .map_err(|e| {
                tracing::error!(
                    error = %e,
                    hint = "A product may have a legacy default_price with a non-standard ID \
                            (expected format: price_...). Remove the default_price from the \
                            product in the Stripe dashboard.",
                    "Failed to list Stripe products"
                );
                AppError::internal("Failed to list products: a product has a legacy default_price with an invalid ID. Remove the default_price from the product in the Stripe dashboard.")
            })?;

        Ok(products
            .data
            .into_iter()
            .map(|p| StripeProductResponse {
                id: p.id.to_string(),
                name: p.name.unwrap_or_default(),
                description: p.description,
                active: p.active.unwrap_or(false),
                metadata: p.metadata.unwrap_or_default(),
                created: p.created.unwrap_or_default(),
            })
            .collect())
    }

    /// Create a product in Stripe
    pub async fn create_product(
        &self,
        name: &str,
        description: Option<&str>,
        metadata: HashMap<String, String>,
    ) -> Result<StripeProductResponse, AppError> {
        let (_config, client) = self.snapshot();

        let mut params = stripe::CreateProduct::new(name);
        if let Some(desc) = description {
            params.description = Some(desc);
        }
        params.metadata = Some(metadata);

        let product = stripe::Product::create(&client, params)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to create Stripe product");
                AppError::internal("Failed to create product")
            })?;

        tracing::info!(product_id = %product.id, name = %name, "Created Stripe product");

        Ok(StripeProductResponse {
            id: product.id.to_string(),
            name: product.name.unwrap_or_default(),
            description: product.description,
            active: product.active.unwrap_or(true),
            metadata: product.metadata.unwrap_or_default(),
            created: product.created.unwrap_or_default(),
        })
    }

    /// Update a product in Stripe
    pub async fn update_product(
        &self,
        product_id: &str,
        name: Option<&str>,
        description: Option<&str>,
        metadata: Option<HashMap<String, String>>,
        active: Option<bool>,
    ) -> Result<StripeProductResponse, AppError> {
        let (_config, client) = self.snapshot();

        let pid: stripe::ProductId = product_id.parse().map_err(|_| {
            AppError::validation("product_id", "Invalid product ID")
        })?;

        let mut params = stripe::UpdateProduct::default();
        if let Some(n) = name {
            params.name = Some(n);
        }
        if let Some(d) = description {
            params.description = Some(d.to_string());
        }
        if let Some(m) = metadata {
            params.metadata = Some(m);
        }
        if let Some(a) = active {
            params.active = Some(a);
        }

        let product = stripe::Product::update(&client, &pid, params)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, product_id = %product_id, "Failed to update Stripe product");
                AppError::internal("Failed to update product")
            })?;

        // Clear tier cache since product metadata may have changed
        self.clear_tier_cache();

        Ok(StripeProductResponse {
            id: product.id.to_string(),
            name: product.name.unwrap_or_default(),
            description: product.description,
            active: product.active.unwrap_or(true),
            metadata: product.metadata.unwrap_or_default(),
            created: product.created.unwrap_or_default(),
        })
    }

    /// Archive (deactivate) a product in Stripe
    pub async fn archive_product(&self, product_id: &str) -> Result<(), AppError> {
        let (_config, client) = self.snapshot();

        let pid: stripe::ProductId = product_id.parse().map_err(|_| {
            AppError::validation("product_id", "Invalid product ID")
        })?;

        let mut params = stripe::UpdateProduct::default();
        params.active = Some(false);

        stripe::Product::update(&client, &pid, params)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, product_id = %product_id, "Failed to archive Stripe product");
                AppError::internal("Failed to archive product")
            })?;

        self.clear_tier_cache();
        tracing::info!(product_id = %product_id, "Archived Stripe product");
        Ok(())
    }

    // ─── Prices ──────────────────────────────────────────────

    /// List prices from Stripe, optionally filtered by product
    pub async fn list_prices(
        &self,
        product_id: Option<&str>,
    ) -> Result<Vec<StripePriceResponse>, AppError> {
        let (_config, client) = self.snapshot();

        let mut params = stripe::ListPrices::new();
        params.limit = Some(100);

        let parsed_product_id: Option<stripe::ProductId> = product_id
            .map(|pid| pid.parse().map_err(|_| AppError::validation("product_id", "Invalid product ID")))
            .transpose()?;
        if let Some(ref pid) = parsed_product_id {
            params.product = Some(stripe::IdOrCreate::Id(pid));
        }

        let prices = stripe::Price::list(&client, &params).await.map_err(|e| {
            tracing::error!(
                error = %e,
                hint = "A price or its parent product may have a legacy ID that doesn't use \
                        the price_... format. Remove or archive the legacy price in the \
                        Stripe dashboard.",
                "Failed to list Stripe prices"
            );
            AppError::internal("Failed to list prices: a price has a legacy ID format. Remove or archive the legacy price in the Stripe dashboard.")
        })?;

        Ok(prices
            .data
            .into_iter()
            .map(|p| {
                let product_id = match &p.product {
                    Some(stripe::Expandable::Id(id)) => id.to_string(),
                    Some(stripe::Expandable::Object(obj)) => obj.id.to_string(),
                    None => String::new(),
                };
                StripePriceResponse {
                    id: p.id.to_string(),
                    product_id,
                    unit_amount: p.unit_amount,
                    currency: p
                        .currency
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "usd".to_string()),
                    recurring_interval: p
                        .recurring
                        .as_ref()
                        .map(|r| format!("{:?}", r.interval).to_lowercase()),
                    active: p.active.unwrap_or(false),
                }
            })
            .collect())
    }

    /// Create a price in Stripe
    pub async fn create_price(
        &self,
        product_id: &str,
        unit_amount: i64,
        currency: &str,
        interval: &str,
    ) -> Result<StripePriceResponse, AppError> {
        let (_config, client) = self.snapshot();

        let cur: stripe::Currency = currency.parse().unwrap_or(stripe::Currency::USD);
        let recurring_interval = match interval {
            "year" => stripe::CreatePriceRecurringInterval::Year,
            "week" => stripe::CreatePriceRecurringInterval::Week,
            "day" => stripe::CreatePriceRecurringInterval::Day,
            _ => stripe::CreatePriceRecurringInterval::Month,
        };

        let mut params = stripe::CreatePrice::new(cur);
        params.product = Some(stripe::IdOrCreate::Id(product_id));
        params.unit_amount = Some(unit_amount);
        params.recurring = Some(stripe::CreatePriceRecurring {
            interval: recurring_interval,
            ..Default::default()
        });

        let price = stripe::Price::create(&client, params)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to create Stripe price");
                AppError::internal("Failed to create price")
            })?;

        tracing::info!(price_id = %price.id, product_id = %product_id, "Created Stripe price");

        let pid = match &price.product {
            Some(stripe::Expandable::Id(id)) => id.to_string(),
            Some(stripe::Expandable::Object(obj)) => obj.id.to_string(),
            None => product_id.to_string(),
        };

        Ok(StripePriceResponse {
            id: price.id.to_string(),
            product_id: pid,
            unit_amount: price.unit_amount,
            currency: price
                .currency
                .map(|c| c.to_string())
                .unwrap_or_else(|| currency.to_string()),
            recurring_interval: Some(interval.to_string()),
            active: price.active.unwrap_or(true),
        })
    }

    /// Archive (deactivate) a price in Stripe
    pub async fn archive_price(&self, price_id: &str) -> Result<(), AppError> {
        let (_config, client) = self.snapshot();

        let pid: stripe::PriceId = price_id.parse().map_err(|_| {
            AppError::validation("price_id", "Invalid price ID")
        })?;

        let mut params = stripe::UpdatePrice::default();
        params.active = Some(false);

        stripe::Price::update(&client, &pid, params)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, price_id = %price_id, "Failed to archive Stripe price");
                AppError::internal("Failed to archive price")
            })?;

        self.clear_tier_cache();
        tracing::info!(price_id = %price_id, "Archived Stripe price");
        Ok(())
    }

    // ─── Subscriptions ───────────────────────────────────────

    /// Get the active subscription for a customer from Stripe
    pub async fn get_customer_subscription(
        &self,
        customer_id: &str,
    ) -> Result<Option<StripeSubscriptionResponse>, AppError> {
        let (_config, client) = self.snapshot();

        let cid: stripe::CustomerId = customer_id.parse().map_err(|_| {
            AppError::validation("customer_id", "Invalid customer ID")
        })?;

        let mut params = stripe::ListSubscriptions::new();
        params.customer = Some(cid);
        params.limit = Some(1);

        let subscriptions =
            stripe::Subscription::list(&client, &params)
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, customer_id = %customer_id, "Failed to list subscriptions");
                    AppError::internal("Failed to fetch subscription")
                })?;

        Ok(subscriptions.data.into_iter().next().map(|sub| {
            let items: Vec<StripeSubscriptionItemResponse> = sub
                .items
                .data
                .iter()
                .map(|item| {
                    let price_id = item
                        .price
                        .as_ref()
                        .map(|p| p.id.to_string())
                        .unwrap_or_default();
                    let product_id = item
                        .price
                        .as_ref()
                        .and_then(|p| p.product.as_ref())
                        .map(|prod| match prod {
                            stripe::Expandable::Id(id) => id.to_string(),
                            stripe::Expandable::Object(obj) => obj.id.to_string(),
                        })
                        .unwrap_or_default();
                    StripeSubscriptionItemResponse {
                        price_id,
                        product_id,
                        quantity: item.quantity.map(|q| q as u64),
                    }
                })
                .collect();

            StripeSubscriptionResponse {
                id: sub.id.to_string(),
                status: format!("{:?}", sub.status).to_lowercase(),
                current_period_start: sub.current_period_start,
                current_period_end: sub.current_period_end,
                cancel_at_period_end: sub.cancel_at_period_end,
                items,
            }
        }))
    }

    // ─── Invoices ────────────────────────────────────────────

    /// List invoices for a customer from Stripe
    pub async fn list_customer_invoices(
        &self,
        customer_id: &str,
        limit: Option<u64>,
    ) -> Result<Vec<StripeInvoiceResponse>, AppError> {
        let (_config, client) = self.snapshot();

        let cid: stripe::CustomerId = customer_id.parse().map_err(|_| {
            AppError::validation("customer_id", "Invalid customer ID")
        })?;

        let mut params = stripe::ListInvoices::new();
        params.customer = Some(cid);
        params.limit = Some(limit.unwrap_or(50));

        let invoices = stripe::Invoice::list(&client, &params)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, customer_id = %customer_id, "Failed to list invoices");
                AppError::internal("Failed to list invoices")
            })?;

        Ok(invoices
            .data
            .into_iter()
            .map(|inv| {
                let customer_id = inv.customer.as_ref().map(|c| match c {
                    stripe::Expandable::Id(id) => id.to_string(),
                    stripe::Expandable::Object(obj) => obj.id.to_string(),
                });
                StripeInvoiceResponse {
                    id: inv.id.to_string(),
                    customer_id,
                    amount_paid: inv.amount_paid.unwrap_or(0),
                    currency: inv
                        .currency
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "usd".to_string()),
                    status: inv.status.map(|s| format!("{:?}", s).to_lowercase()),
                    invoice_pdf: inv.invoice_pdf,
                    hosted_invoice_url: inv.hosted_invoice_url,
                    created: inv.created.unwrap_or_default(),
                    description: inv.description,
                    number: inv.number,
                }
            })
            .collect())
    }

    /// Get a single invoice from Stripe by ID
    pub async fn get_invoice(
        &self,
        invoice_id: &str,
    ) -> Result<StripeInvoiceResponse, AppError> {
        let (_config, client) = self.snapshot();

        let iid: stripe::InvoiceId = invoice_id.parse().map_err(|_| {
            AppError::validation("invoice_id", "Invalid invoice ID")
        })?;

        let inv = stripe::Invoice::retrieve(&client, &iid, &[])
            .await
            .map_err(|e| {
                tracing::error!(error = %e, invoice_id = %invoice_id, "Failed to retrieve invoice");
                AppError::not_found("Invoice")
            })?;

        let customer_id = inv.customer.as_ref().map(|c| match c {
            stripe::Expandable::Id(id) => id.to_string(),
            stripe::Expandable::Object(obj) => obj.id.to_string(),
        });

        Ok(StripeInvoiceResponse {
            id: inv.id.to_string(),
            customer_id,
            amount_paid: inv.amount_paid.unwrap_or(0),
            currency: inv
                .currency
                .map(|c| c.to_string())
                .unwrap_or_else(|| "usd".to_string()),
            status: inv.status.map(|s| format!("{:?}", s).to_lowercase()),
            invoice_pdf: inv.invoice_pdf,
            hosted_invoice_url: inv.hosted_invoice_url,
            created: inv.created.unwrap_or_default(),
            description: inv.description,
            number: inv.number,
        })
    }

    // ─── Webhook Endpoints ───────────────────────────────────

    /// List all webhook endpoints from Stripe
    pub async fn list_webhook_endpoints(
        &self,
    ) -> Result<Vec<StripeWebhookEndpointResponse>, AppError> {
        let (config, _client) = self.snapshot();

        // Use raw reqwest — async-stripe may not expose WebhookEndpoint in current features
        let url = "https://api.stripe.com/v1/webhook_endpoints?limit=100";
        let resp = reqwest::Client::new()
            .get(url)
            .bearer_auth(&config.secret_key)
            .send()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to list webhook endpoints");
                AppError::internal("Failed to list webhook endpoints")
            })?;

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            tracing::error!(error = %e, "Failed to parse webhook endpoints response");
            AppError::internal("Failed to list webhook endpoints")
        })?;

        let endpoints = body["data"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|ep| StripeWebhookEndpointResponse {
                id: ep["id"].as_str().unwrap_or_default().to_string(),
                url: ep["url"].as_str().unwrap_or_default().to_string(),
                enabled_events: ep["enabled_events"]
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|e| e.as_str().map(String::from))
                    .collect(),
                status: ep["status"].as_str().unwrap_or("enabled").to_string(),
                secret: None,
            })
            .collect();

        Ok(endpoints)
    }

    /// Create a webhook endpoint in Stripe, returns the endpoint with secret
    pub async fn create_webhook_endpoint(
        &self,
        url: &str,
        events: Vec<String>,
    ) -> Result<StripeWebhookEndpointResponse, AppError> {
        let (config, _client) = self.snapshot();

        let mut form_params: Vec<(String, String)> = Vec::new();
        form_params.push(("url".to_string(), url.to_string()));
        for (i, event) in events.iter().enumerate() {
            form_params.push((format!("enabled_events[{}]", i), event.clone()));
        }

        let resp = reqwest::Client::new()
            .post("https://api.stripe.com/v1/webhook_endpoints")
            .bearer_auth(&config.secret_key)
            .form(&form_params)
            .send()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to create webhook endpoint");
                AppError::internal("Failed to create webhook endpoint")
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            tracing::error!(status = %status, body = %body, "Stripe webhook endpoint creation failed");
            return Err(AppError::internal("Failed to create webhook endpoint"));
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            tracing::error!(error = %e, "Failed to parse webhook endpoint response");
            AppError::internal("Failed to create webhook endpoint")
        })?;

        let endpoint = StripeWebhookEndpointResponse {
            id: body["id"].as_str().unwrap_or_default().to_string(),
            url: body["url"].as_str().unwrap_or_default().to_string(),
            enabled_events: body["enabled_events"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|e| e.as_str().map(String::from))
                .collect(),
            status: body["status"].as_str().unwrap_or("enabled").to_string(),
            secret: body["secret"].as_str().map(String::from),
        };

        tracing::info!(
            webhook_id = %endpoint.id,
            url = %endpoint.url,
            "Created Stripe webhook endpoint"
        );

        Ok(endpoint)
    }

    /// Delete a webhook endpoint in Stripe
    pub async fn delete_webhook_endpoint(&self, endpoint_id: &str) -> Result<(), AppError> {
        let (config, _client) = self.snapshot();

        let url = format!(
            "https://api.stripe.com/v1/webhook_endpoints/{}",
            endpoint_id
        );

        let resp = reqwest::Client::new()
            .delete(&url)
            .bearer_auth(&config.secret_key)
            .send()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, endpoint_id = %endpoint_id, "Failed to delete webhook endpoint");
                AppError::internal("Failed to delete webhook endpoint")
            })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            tracing::error!(status = %status, body = %body, "Stripe webhook endpoint deletion failed");
            return Err(AppError::internal("Failed to delete webhook endpoint"));
        }

        tracing::info!(endpoint_id = %endpoint_id, "Deleted Stripe webhook endpoint");
        Ok(())
    }

    // ─── Tier Resolution ─────────────────────────────────────

    /// Resolve membership tier from a Stripe price ID by looking at the product's metadata.
    /// Uses an in-memory cache with TTL to avoid hitting Stripe on every call.
    pub async fn resolve_tier_from_price(
        &self,
        price_id: &str,
    ) -> Result<MembershipTier, AppError> {
        // Check cache first
        {
            let inner = self.inner.read().expect("StripeService lock poisoned");
            if let Some(entry) = inner.tier_cache.get(price_id) {
                if entry.cached_at.elapsed() < TIER_CACHE_TTL {
                    return Ok(entry.tier);
                }
            }
        }

        // Fetch price from Stripe to get product
        let (_config, client) = self.snapshot();

        let pid: stripe::PriceId = price_id.parse().map_err(|_| {
            AppError::validation("price_id", "Invalid price ID")
        })?;

        let price = stripe::Price::retrieve(&client, &pid, &["product"])
            .await
            .map_err(|e| {
                tracing::error!(error = %e, price_id = %price_id, "Failed to retrieve price for tier resolution");
                AppError::internal("Failed to resolve tier")
            })?;

        // Get product metadata
        let tier = match &price.product {
            Some(stripe::Expandable::Object(product)) => {
                let metadata = product.metadata.as_ref();
                match metadata.and_then(|m| m.get("tier")).map(|s| s.as_str()) {
                    Some("business") => MembershipTier::Business,
                    _ => MembershipTier::Personal,
                }
            }
            Some(stripe::Expandable::Id(product_id)) => {
                // Product not expanded, fetch it
                let product = stripe::Product::retrieve(&client, product_id, &[])
                    .await
                    .map_err(|e| {
                        tracing::error!(error = %e, "Failed to retrieve product for tier resolution");
                        AppError::internal("Failed to resolve tier")
                    })?;
                let metadata = product.metadata.as_ref();
                match metadata.and_then(|m| m.get("tier")).map(|s| s.as_str()) {
                    Some("business") => MembershipTier::Business,
                    _ => MembershipTier::Personal,
                }
            }
            None => MembershipTier::Personal,
        };

        // Update cache
        {
            let mut inner = self.inner.write().expect("StripeService lock poisoned");
            inner.tier_cache.insert(
                price_id.to_string(),
                TierCacheEntry {
                    tier,
                    cached_at: Instant::now(),
                },
            );
        }

        Ok(tier)
    }

    /// Get active prices grouped by tier. Returns a map of tier -> list of prices.
    pub async fn get_prices_by_tier(
        &self,
    ) -> Result<HashMap<MembershipTier, Vec<StripePriceResponse>>, AppError> {
        let products = self.list_products().await?;
        let prices = self.list_prices(None).await?;

        // Build product_id -> tier mapping from product metadata
        let mut product_tier_map: HashMap<String, MembershipTier> = HashMap::new();
        for product in &products {
            if product.active {
                let tier = match product.metadata.get("tier").map(|s| s.as_str()) {
                    Some("business") => MembershipTier::Business,
                    Some("personal") => MembershipTier::Personal,
                    _ => continue, // Skip products without tier metadata
                };
                product_tier_map.insert(product.id.clone(), tier);
            }
        }

        let mut result: HashMap<MembershipTier, Vec<StripePriceResponse>> = HashMap::new();
        for price in prices {
            if price.active {
                if let Some(tier) = product_tier_map.get(&price.product_id) {
                    result.entry(*tier).or_default().push(price);
                }
            }
        }

        Ok(result)
    }

    /// Clear the tier cache (called after product/price changes)
    fn clear_tier_cache(&self) {
        let mut inner = self.inner.write().expect("StripeService lock poisoned");
        inner.tier_cache.clear();
    }

    // ─── Existing Methods (Checkout, Customer, etc.) ─────────

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

    /// Create a checkout session for a specific membership tier.
    /// Discovers the price from Stripe product metadata.
    pub async fn create_checkout_session(
        &self,
        customer_id: &str,
        user_id: Uuid,
        tier: MembershipTier,
    ) -> Result<(String, String), AppError> {
        let (config, client) = self.snapshot();

        // Discover price for this tier from Stripe
        let prices_by_tier = self.get_prices_by_tier().await?;
        let prices = prices_by_tier.get(&tier).ok_or_else(|| {
            tracing::error!(tier = %tier.as_str(), "No active price found for tier");
            AppError::validation("tier", "No active price configured for this tier")
        })?;

        let price_id = &prices
            .first()
            .ok_or_else(|| {
                AppError::validation("tier", "No active price configured for this tier")
            })?
            .id;

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

        let payload_str = std::str::from_utf8(payload)
            .map_err(|_| AppError::validation("body", "Invalid UTF-8 in webhook payload"))?;
        let signed_payload = format!("{}.{}", timestamp, payload_str);

        let (config, _) = self.snapshot();
        let mut mac = HmacSha256::new_from_slice(config.webhook_secret.as_bytes())
            .map_err(|_| AppError::internal("Invalid webhook secret key"))?;
        mac.update(signed_payload.as_bytes());
        let expected = hex::encode(mac.finalize().into_bytes());

        if signatures.iter().any(|sig| sig == &expected) {
            Ok(())
        } else {
            tracing::warn!("Webhook signature verification failed");
            Err(AppError::Unauthorized)
        }
    }

    /// Create a Stripe Customer and a SetupIntent for $0 card authorization at signup.
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> StripeConfig {
        StripeConfig {
            secret_key: "sk_test_xxx".to_string(),
            webhook_secret: "whsec_test_secret".to_string(),
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

    // -- Webhook signature verification --

    #[test]
    fn verify_webhook_signature_valid() {
        let service = test_service();
        let payload = b"{\"type\":\"test\"}";
        let timestamp = chrono::Utc::now().timestamp().to_string();

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
        let old_ts = (chrono::Utc::now().timestamp() - 600).to_string();

        let signed_payload = format!("{}.{}", old_ts, std::str::from_utf8(payload).unwrap());
        let mut mac = HmacSha256::new_from_slice(b"whsec_test_secret").unwrap();
        mac.update(signed_payload.as_bytes());
        let sig = hex::encode(mac.finalize().into_bytes());

        let header = format!("t={},v1={}", old_ts, sig);
        assert!(service.verify_webhook_signature(payload, &header).is_err());
    }
}
