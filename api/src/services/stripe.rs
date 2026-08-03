//! Stripe config construction + the shared Stripe service (DEV-515).
//!
//! The async-stripe `StripeService`, runtime `StripeConfig`, and response DTOs
//! live in the shared `dunite-stripe` crate (consumed by bunyip too) and are
//! re-exported here, so `crate::services::stripe::*` paths are unchanged.
//!
//! The crate's methods return a neutral `StripeServiceError`; [`stripe_err`]
//! maps it to a8n's `AppError` at the call sites (the orphan rule forbids a
//! blanket `From` impl between the two foreign types). What stays a8n-specific
//! is building a `StripeConfig` from env / the DB row: a8n's env names, the
//! `a8n-tools` app-tag default, and decryption with a8n's `EncryptionKeySet`.

pub use dunite_stripe::{StripeConfig, StripeService, StripeServiceError};
use dunite_stripe::{SECRET_KEY_PLACEHOLDER, WEBHOOK_SECRET_PLACEHOLDER};

use crate::errors::AppError;
use crate::models::stripe::decrypt_secret;
use crate::services::encryption::EncryptionKeySet;

/// Map the shared crate's neutral [`StripeServiceError`] to a8n's `AppError`.
/// Used as `.map_err(stripe_err)?` at every `StripeService` call site: a blanket
/// `From<StripeServiceError> for AppError` is impossible (both are foreign
/// types, so the impl would violate the orphan rule).
pub fn stripe_err(e: StripeServiceError) -> AppError {
    match e {
        StripeServiceError::Internal(message) => AppError::internal(message),
        StripeServiceError::Validation { field, message } => AppError::validation(field, message),
        StripeServiceError::NotFound(resource) => AppError::not_found(resource),
        StripeServiceError::Unauthorized => AppError::Unauthorized,
    }
}

/// Build a runtime [`StripeConfig`] from env (was `StripeConfig::from_env`).
/// Kept a8n-side because the env names, the checkout-URL derivation and the
/// `a8n-tools` app-tag default are a8n's, not the shared crate's.
pub fn stripe_config_from_env() -> Result<StripeConfig, AppError> {
    let frontend_origin =
        std::env::var("CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:5173".to_string());
    let base = frontend_origin.trim_end_matches('/');

    Ok(StripeConfig {
        secret_key: std::env::var("STRIPE_SECRET_KEY")
            .unwrap_or_else(|_| SECRET_KEY_PLACEHOLDER.to_string()),
        webhook_secret: std::env::var("STRIPE_WEBHOOK_SECRET")
            .unwrap_or_else(|_| WEBHOOK_SECRET_PLACEHOLDER.to_string()),
        success_url: std::env::var("STRIPE_SUCCESS_URL")
            .unwrap_or_else(|_| format!("{base}/checkout/success")),
        cancel_url: std::env::var("STRIPE_CANCEL_URL")
            .unwrap_or_else(|_| format!("{base}/pricing?checkout=canceled")),
        free_price_id: std::env::var("STRIPE_FREE_PRICE_ID").ok(),
        app_tag: std::env::var("STRIPE_APP_TAG")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "a8n-tools".to_string()),
        // a8n runs no signup free trial: every `create_checkout_session` call
        // passes `eligible_for_trial = false`, so this length is never read. It
        // exists because bunyip's trial (BUNYIP-209) needs it on the shared
        // config. Held at the 30-day shared default rather than 0, so switching
        // trials on here stays a one-line change and never sends Stripe an
        // invalid zero-day trial.
        trial_period_days: 30,
    })
}

/// Build a runtime [`StripeConfig`] from the DB model, decrypting secrets and
/// falling back to env for any field not set in the DB (was
/// `StripeConfig::from_db_model`). a8n-side because it decrypts with a8n's
/// [`EncryptionKeySet`].
pub fn stripe_config_from_db_model(
    db: &crate::models::stripe::StripeConfig,
    key_set: &EncryptionKeySet,
) -> Result<StripeConfig, AppError> {
    let env_config = stripe_config_from_env()?;

    let secret_key = match (&db.secret_key, &db.secret_key_nonce) {
        (Some(ct), Some(nonce)) => decrypt_secret(key_set, ct, nonce, db.key_version)?,
        _ => env_config.secret_key,
    };
    let webhook_secret = match (&db.webhook_secret, &db.webhook_secret_nonce) {
        (Some(ct), Some(nonce)) => decrypt_secret(key_set, ct, nonce, db.key_version)?,
        _ => env_config.webhook_secret,
    };

    let app_tag = db.app_tag.clone().unwrap_or(env_config.app_tag);

    Ok(StripeConfig {
        secret_key,
        webhook_secret,
        success_url: env_config.success_url,
        cancel_url: env_config.cancel_url,
        free_price_id: env_config.free_price_id,
        app_tag,
        trial_period_days: env_config.trial_period_days,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // No `stripe_config_from_env` test here on purpose: a8n has no env lock for
    // tests, and mutating `STRIPE_*` from one test perturbs the other
    // env-reading tests running in parallel (the `config::*` suite).

    #[test]
    fn stripe_err_maps_variants() {
        assert!(matches!(
            stripe_err(StripeServiceError::internal("x")),
            AppError::InternalError { .. }
        ));
        assert!(matches!(
            stripe_err(StripeServiceError::validation("secret_key", "missing")),
            AppError::ValidationError { .. }
        ));
        assert!(matches!(
            stripe_err(StripeServiceError::not_found("Product")),
            AppError::NotFound { .. }
        ));
        assert!(matches!(
            stripe_err(StripeServiceError::Unauthorized),
            AppError::Unauthorized
        ));
    }
}
