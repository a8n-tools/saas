//! Webhook handlers
//!
//! This module contains HTTP handlers for external webhooks (Stripe, etc.)

use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use std::sync::Arc;

use crate::errors::AppError;
use crate::models::{CreatePayment, CreateMembership, PaymentStatus, MembershipStatus};
use crate::repositories::{PaymentRepository, MembershipRepository, UserRepository};
use crate::services::StripeService;

/// POST /v1/webhooks/stripe
/// Handle Stripe webhook events
pub async fn stripe_webhook(
    req: HttpRequest,
    body: web::Bytes,
    pool: web::Data<PgPool>,
    stripe: web::Data<Arc<StripeService>>,
) -> Result<HttpResponse, AppError> {
    // Get signature header
    let signature = req
        .headers()
        .get("Stripe-Signature")
        .and_then(|h| h.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    // Verify webhook signature
    stripe.verify_webhook_signature(&body, signature)?;

    // Parse the event
    let payload = String::from_utf8(body.to_vec())
        .map_err(|_| AppError::validation("body", "Invalid UTF-8"))?;

    let event: serde_json::Value = serde_json::from_str(&payload)
        .map_err(|_| AppError::validation("body", "Invalid JSON"))?;

    let event_type = event["type"]
        .as_str()
        .ok_or(AppError::validation("type", "Missing event type"))?;

    tracing::info!(event_type = %event_type, "Processing Stripe webhook");

    // Route to appropriate handler
    match event_type {
        "checkout.session.completed" => {
            handle_checkout_completed(&event, &pool).await?;
        }
        "customer.subscription.created" => {
            handle_subscription_created(&event, &pool).await?;
        }
        "customer.subscription.updated" => {
            handle_subscription_updated(&event, &pool).await?;
        }
        "customer.subscription.deleted" => {
            handle_subscription_deleted(&event, &pool).await?;
        }
        "invoice.payment_succeeded" => {
            handle_payment_succeeded(&event, &pool).await?;
        }
        "invoice.payment_failed" => {
            handle_payment_failed(&event, &pool).await?;
        }
        _ => {
            tracing::debug!(event_type = %event_type, "Unhandled Stripe event type");
        }
    }

    Ok(HttpResponse::Ok().finish())
}

async fn handle_checkout_completed(
    event: &serde_json::Value,
    pool: &PgPool,
) -> Result<(), AppError> {
    let session = &event["data"]["object"];

    // Get user ID from metadata
    let user_id_str = session["metadata"]["user_id"]
        .as_str()
        .ok_or(AppError::validation("metadata", "Missing user_id"))?;

    let user_id: uuid::Uuid = user_id_str
        .parse()
        .map_err(|_| AppError::validation("user_id", "Invalid UUID"))?;

    // Get price info
    let amount = session["amount_total"]
        .as_i64()
        .unwrap_or(300) as i32;

    // Update user membership status and lock price
    UserRepository::update_membership_status(pool, user_id, MembershipStatus::Active).await?;

    // Lock the price for life
    let price_id = session["subscription"]
        .as_str()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "price_default".to_string());

    UserRepository::lock_price(pool, user_id, &price_id, amount).await?;

    tracing::info!(user_id = %user_id, "Checkout completed, membership activated");

    Ok(())
}

async fn handle_subscription_created(
    event: &serde_json::Value,
    pool: &PgPool,
) -> Result<(), AppError> {
    let subscription = &event["data"]["object"];

    let stripe_subscription_id = subscription["id"]
        .as_str()
        .ok_or(AppError::validation("id", "Missing subscription ID"))?;

    let customer_id = subscription["customer"]
        .as_str()
        .ok_or(AppError::validation("customer", "Missing customer ID"))?;

    // Find user by customer ID
    let user = UserRepository::find_by_stripe_customer_id(pool, customer_id)
        .await?
        .ok_or(AppError::not_found("User"))?;

    // Parse dates
    let period_start = subscription["current_period_start"]
        .as_i64()
        .map(|ts| chrono::DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now))
        .unwrap_or_else(Utc::now);

    let period_end = subscription["current_period_end"]
        .as_i64()
        .map(|ts| chrono::DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now))
        .unwrap_or_else(Utc::now);

    let price_id = subscription["items"]["data"][0]["price"]["id"]
        .as_str()
        .unwrap_or("unknown");

    let amount = subscription["items"]["data"][0]["price"]["unit_amount"]
        .as_i64()
        .unwrap_or(300) as i32;

    let status = subscription["status"]
        .as_str()
        .unwrap_or("active");

    // Create membership record
    MembershipRepository::create(
        pool,
        CreateMembership {
            user_id: user.id,
            stripe_subscription_id: stripe_subscription_id.to_string(),
            stripe_price_id: price_id.to_string(),
            status: status.to_string(),
            current_period_start: period_start,
            current_period_end: period_end,
            amount,
            currency: "usd".to_string(),
        },
    )
    .await?;

    tracing::info!(
        user_id = %user.id,
        membership_id = %stripe_subscription_id,
        "Membership created"
    );

    Ok(())
}

async fn handle_subscription_updated(
    event: &serde_json::Value,
    pool: &PgPool,
) -> Result<(), AppError> {
    let subscription = &event["data"]["object"];

    let stripe_subscription_id = subscription["id"]
        .as_str()
        .ok_or(AppError::validation("id", "Missing subscription ID"))?;

    let status = subscription["status"]
        .as_str()
        .unwrap_or("active");

    let cancel_at_period_end = subscription["cancel_at_period_end"]
        .as_bool()
        .unwrap_or(false);

    // Find membership by Stripe ID
    if let Some(membership) = MembershipRepository::find_by_stripe_subscription_id(pool, stripe_subscription_id).await? {
        // Update status
        MembershipRepository::update_status(pool, membership.id, status).await?;

        // Update cancel_at_period_end
        MembershipRepository::set_cancel_at_period_end(pool, membership.id, cancel_at_period_end).await?;

        // Update user membership status
        let user_status = match status {
            "active" => MembershipStatus::Active,
            "past_due" => MembershipStatus::PastDue,
            "canceled" => MembershipStatus::Canceled,
            _ => MembershipStatus::Active,
        };
        UserRepository::update_membership_status(pool, membership.user_id, user_status).await?;

        tracing::info!(
            membership_id = %stripe_subscription_id,
            status = %status,
            "Membership updated"
        );
    }

    Ok(())
}

async fn handle_subscription_deleted(
    event: &serde_json::Value,
    pool: &PgPool,
) -> Result<(), AppError> {
    let subscription = &event["data"]["object"];

    let stripe_subscription_id = subscription["id"]
        .as_str()
        .ok_or(AppError::validation("id", "Missing subscription ID"))?;

    // Find membership by Stripe ID
    if let Some(membership) = MembershipRepository::find_by_stripe_subscription_id(pool, stripe_subscription_id).await? {
        // Update status to canceled
        MembershipRepository::update_status(pool, membership.id, "canceled").await?;

        // Update user membership status
        UserRepository::update_membership_status(pool, membership.user_id, MembershipStatus::Canceled).await?;

        // Clear any grace period
        UserRepository::clear_grace_period(pool, membership.user_id).await?;

        tracing::info!(
            user_id = %membership.user_id,
            membership_id = %stripe_subscription_id,
            "Membership deleted"
        );
    }

    Ok(())
}

async fn handle_payment_succeeded(
    event: &serde_json::Value,
    pool: &PgPool,
) -> Result<(), AppError> {
    let invoice = &event["data"]["object"];

    let customer_id = invoice["customer"]
        .as_str()
        .ok_or(AppError::validation("customer", "Missing customer ID"))?;

    // Find user by customer ID
    let user = match UserRepository::find_by_stripe_customer_id(pool, customer_id).await? {
        Some(u) => u,
        None => {
            tracing::warn!(customer_id = %customer_id, "User not found for payment");
            return Ok(());
        }
    };

    let amount = invoice["amount_paid"]
        .as_i64()
        .unwrap_or(0) as i32;

    let payment_intent_id = invoice["payment_intent"]
        .as_str()
        .map(|s| s.to_string());

    let invoice_id = invoice["id"]
        .as_str()
        .map(|s| s.to_string());

    // Get membership ID if available
    let subscription_id = if let Some(stripe_sub_id) = invoice["subscription"].as_str() {
        MembershipRepository::find_by_stripe_subscription_id(pool, stripe_sub_id)
            .await?
            .map(|m| m.id)
    } else {
        None
    };

    // Record the payment
    PaymentRepository::create(
        pool,
        CreatePayment {
            user_id: user.id,
            subscription_id,
            stripe_payment_intent_id: payment_intent_id,
            stripe_invoice_id: invoice_id,
            amount,
            currency: "usd".to_string(),
            status: PaymentStatus::Succeeded,
            failure_reason: None,
        },
    )
    .await?;

    // Clear any grace period if exists
    if user.grace_period_start.is_some() {
        UserRepository::clear_grace_period(pool, user.id).await?;
        UserRepository::update_membership_status(pool, user.id, MembershipStatus::Active).await?;
    }

    tracing::info!(
        user_id = %user.id,
        amount = amount,
        "Payment succeeded"
    );

    Ok(())
}

async fn handle_payment_failed(
    event: &serde_json::Value,
    pool: &PgPool,
) -> Result<(), AppError> {
    let invoice = &event["data"]["object"];

    let customer_id = invoice["customer"]
        .as_str()
        .ok_or(AppError::validation("customer", "Missing customer ID"))?;

    // Find user by customer ID
    let user = match UserRepository::find_by_stripe_customer_id(pool, customer_id).await? {
        Some(u) => u,
        None => {
            tracing::warn!(customer_id = %customer_id, "User not found for failed payment");
            return Ok(());
        }
    };

    let amount = invoice["amount_due"]
        .as_i64()
        .unwrap_or(0) as i32;

    let failure_message = invoice["last_finalization_error"]["message"]
        .as_str()
        .map(|s| s.to_string());

    // Get membership ID if available
    let subscription_id = if let Some(stripe_sub_id) = invoice["subscription"].as_str() {
        MembershipRepository::find_by_stripe_subscription_id(pool, stripe_sub_id)
            .await?
            .map(|m| m.id)
    } else {
        None
    };

    // Record the failed payment
    PaymentRepository::create(
        pool,
        CreatePayment {
            user_id: user.id,
            subscription_id,
            stripe_payment_intent_id: invoice["payment_intent"].as_str().map(|s| s.to_string()),
            stripe_invoice_id: invoice["id"].as_str().map(|s| s.to_string()),
            amount,
            currency: "usd".to_string(),
            status: PaymentStatus::Failed,
            failure_reason: failure_message,
        },
    )
    .await?;

    // Start grace period if not already started
    if user.grace_period_start.is_none() {
        let now = Utc::now();
        let grace_end = now + Duration::days(30);

        UserRepository::set_grace_period(pool, user.id, now, grace_end).await?;
        UserRepository::update_membership_status(pool, user.id, MembershipStatus::GracePeriod).await?;

        tracing::info!(
            user_id = %user.id,
            grace_period_end = %grace_end,
            "Payment failed, grace period started"
        );
    }

    Ok(())
}
