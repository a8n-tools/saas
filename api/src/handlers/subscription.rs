//! Subscription handlers
//!
//! This module contains HTTP handlers for subscription management endpoints.

use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

use crate::errors::AppError;
use crate::middleware::AuthenticatedUser;
use crate::models::{PaymentResponse, SubscriptionResponse};
use crate::repositories::{PaymentRepository, SubscriptionRepository, UserRepository};
use crate::responses::{get_request_id, success};
use crate::services::StripeService;

/// Response for checkout session creation
#[derive(Debug, Serialize)]
pub struct CheckoutResponse {
    pub checkout_url: String,
    pub session_id: String,
}

/// Response for billing portal
#[derive(Debug, Serialize)]
pub struct PortalResponse {
    pub url: String,
}

/// GET /v1/subscriptions/me
/// Get current user's subscription status
pub async fn get_subscription(
    req: HttpRequest,
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    // Get user from database for fresh data
    let db_user = UserRepository::find_by_id(&pool, user.0.sub)
        .await?
        .ok_or(AppError::not_found("User"))?;

    // Get active subscription if any
    let subscription = SubscriptionRepository::find_by_user_id(&pool, user.0.sub).await?;

    let response = SubscriptionResponse {
        status: db_user.subscription_status.clone(),
        price_locked: db_user.price_locked,
        locked_price_amount: db_user.locked_price_amount,
        current_period_end: subscription.as_ref().map(|s| s.current_period_end),
        cancel_at_period_end: subscription.as_ref().map(|s| s.cancel_at_period_end).unwrap_or(false),
        grace_period_end: db_user.grace_period_end,
    };

    Ok(success(response, request_id))
}

/// POST /v1/subscriptions/checkout
/// Create a Stripe checkout session
pub async fn create_checkout(
    req: HttpRequest,
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
    stripe: web::Data<Arc<StripeService>>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    // Get user from database
    let db_user = UserRepository::find_by_id(&pool, user.0.sub)
        .await?
        .ok_or(AppError::not_found("User"))?;

    // Check if user already has active subscription
    if db_user.subscription_status == "active" {
        return Err(AppError::conflict("You already have an active subscription"));
    }

    // Get or create Stripe customer
    let customer_id = match db_user.stripe_customer_id {
        Some(id) => id,
        None => {
            let customer_id = stripe.create_customer(&db_user.email, db_user.id).await?;
            UserRepository::update_stripe_customer_id(&pool, db_user.id, &customer_id).await?;
            customer_id
        }
    };

    // Create checkout session
    let (session_id, checkout_url) = stripe
        .create_checkout_session(&customer_id, db_user.id)
        .await?;

    Ok(success(
        CheckoutResponse {
            checkout_url,
            session_id,
        },
        request_id,
    ))
}

/// POST /v1/subscriptions/cancel
/// Cancel subscription at period end
pub async fn cancel_subscription(
    req: HttpRequest,
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
    stripe: web::Data<Arc<StripeService>>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    // Get user's subscription
    let subscription = SubscriptionRepository::find_by_user_id(&pool, user.0.sub)
        .await?
        .ok_or(AppError::not_found("Subscription"))?;

    if subscription.cancel_at_period_end {
        return Err(AppError::conflict("Subscription is already scheduled for cancellation"));
    }

    // Cancel at period end in Stripe
    stripe
        .cancel_subscription(&subscription.stripe_subscription_id, true)
        .await?;

    // Update local database
    SubscriptionRepository::set_cancel_at_period_end(&pool, subscription.id, true).await?;

    Ok(crate::responses::success_no_data(request_id))
}

/// POST /v1/subscriptions/reactivate
/// Reactivate a subscription that's scheduled for cancellation
pub async fn reactivate_subscription(
    req: HttpRequest,
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
    stripe: web::Data<Arc<StripeService>>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    // Get user's subscription
    let subscription = SubscriptionRepository::find_by_user_id(&pool, user.0.sub)
        .await?
        .ok_or(AppError::not_found("Subscription"))?;

    if !subscription.cancel_at_period_end {
        return Err(AppError::conflict("Subscription is not scheduled for cancellation"));
    }

    // Reactivate in Stripe
    stripe
        .reactivate_subscription(&subscription.stripe_subscription_id)
        .await?;

    // Update local database
    SubscriptionRepository::set_cancel_at_period_end(&pool, subscription.id, false).await?;

    Ok(crate::responses::success_no_data(request_id))
}

/// POST /v1/subscriptions/billing-portal
/// Get a link to the Stripe billing portal
pub async fn billing_portal(
    req: HttpRequest,
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
    stripe: web::Data<Arc<StripeService>>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    // Get user from database
    let db_user = UserRepository::find_by_id(&pool, user.0.sub)
        .await?
        .ok_or(AppError::not_found("User"))?;

    let customer_id = db_user
        .stripe_customer_id
        .ok_or(AppError::not_found("No billing account found"))?;

    let url = stripe.create_billing_portal_session(&customer_id).await?;

    Ok(success(PortalResponse { url }, request_id))
}

/// GET /v1/subscriptions/payments
/// Get payment history
pub async fn get_payment_history(
    req: HttpRequest,
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);

    let (payments, total) = PaymentRepository::list_by_user(&pool, user.0.sub, page, per_page).await?;

    let payment_responses: Vec<PaymentResponse> = payments.into_iter().map(PaymentResponse::from).collect();

    Ok(crate::responses::paginated(
        payment_responses,
        total,
        page,
        per_page,
        request_id,
    ))
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}
