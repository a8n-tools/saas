# 05 - Stripe Integration

## Overview

This document contains prompts for implementing Stripe payment processing including checkout, webhooks, and subscription management.

## Prerequisites
- Completed 01-04 documents
- Stripe account with test API keys
- Stripe CLI installed for webhook testing

---

## Prompt 5.1: Stripe Client Setup

```text
Set up Stripe API client for Rust.

Add dependencies to Cargo.toml:
- stripe-rust = "0.31"  # Or use reqwest directly with Stripe REST API

Create src/services/stripe.rs:

1. StripeConfig struct:
   ```rust
   pub struct StripeConfig {
       pub secret_key: String,
       pub webhook_secret: String,
       pub price_id: String,
       pub success_url: String,
       pub cancel_url: String,
   }
   ```

2. Load from environment:
   - STRIPE_SECRET_KEY
   - STRIPE_WEBHOOK_SECRET
   - STRIPE_PRICE_ID (price_a8n_monthly_v1)
   - STRIPE_SUCCESS_URL (https://app.example.com/dashboard?checkout=success)
   - STRIPE_CANCEL_URL (https://app.example.com/pricing?checkout=canceled)

3. StripeService struct:
   ```rust
   pub struct StripeService {
       client: stripe::Client,
       config: StripeConfig,
   }

   impl StripeService {
       pub fn new(config: StripeConfig) -> Self;
   }
   ```

4. Customer management:
   ```rust
   pub async fn create_customer(
       &self,
       email: &str,
       user_id: Uuid,
   ) -> Result<stripe::Customer, AppError>;

   pub async fn get_customer(
       &self,
       customer_id: &str,
   ) -> Result<stripe::Customer, AppError>;
   ```

5. Error handling:
   - Map Stripe errors to AppError
   - Log Stripe error details
   - Return user-friendly messages

Write unit tests with mocked Stripe responses.
```

---

## Prompt 5.2: Checkout Session Creation

```text
Implement Stripe Checkout session creation.

Extend src/services/stripe.rs:

1. Create checkout session:
   ```rust
   pub async fn create_checkout_session(
       &self,
       customer_id: &str,
       user_id: Uuid,
       price_id: &str,
   ) -> Result<stripe::CheckoutSession, AppError>;
   ```

   Configure checkout session:
   - Mode: subscription
   - Line items: 1x price_id
   - Customer: customer_id
   - Success URL with session_id parameter
   - Cancel URL
   - Metadata: user_id
   - Allow promotion codes: false
   - Billing address collection: required
   - Payment method types: card

2. Create subscription checkout handler:
   ```rust
   // In src/handlers/subscription.rs

   #[derive(Serialize)]
   pub struct CheckoutResponse {
       pub checkout_url: String,
       pub session_id: String,
   }

   pub async fn create_checkout(
       user: AuthenticatedUser,
       pool: web::Data<PgPool>,
       stripe: web::Data<StripeService>,
   ) -> Result<HttpResponse, AppError>;
   ```

   Flow:
   1. Get user from database
   2. If no stripe_customer_id, create Stripe customer
   3. Save stripe_customer_id to user
   4. Create checkout session
   5. Return checkout URL

3. Verify user doesn't already have active subscription.

4. Create audit log for checkout initiated.

Write integration test with Stripe test mode.
```

---

## Prompt 5.3: Webhook Handler Setup

```text
Implement Stripe webhook handling infrastructure.

Create src/handlers/webhook.rs:

1. Webhook signature verification:
   ```rust
   pub async fn stripe_webhook(
       req: HttpRequest,
       body: web::Bytes,
       stripe: web::Data<StripeService>,
       pool: web::Data<PgPool>,
   ) -> Result<HttpResponse, AppError> {
       // Get Stripe-Signature header
       let signature = req
           .headers()
           .get("Stripe-Signature")
           .ok_or(AppError::Unauthorized)?
           .to_str()
           .map_err(|_| AppError::Unauthorized)?;

       // Verify signature
       let event = stripe::Webhook::construct_event(
           &body,
           signature,
           &stripe.config.webhook_secret,
       )?;

       // Route to handler
       handle_event(event, pool).await
   }
   ```

2. Event router:
   ```rust
   async fn handle_event(
       event: stripe::Event,
       pool: web::Data<PgPool>,
   ) -> Result<HttpResponse, AppError> {
       match event.type_ {
           EventType::CheckoutSessionCompleted => {
               handle_checkout_completed(event, pool).await
           }
           EventType::CustomerSubscriptionCreated => {
               handle_subscription_created(event, pool).await
           }
           EventType::CustomerSubscriptionUpdated => {
               handle_subscription_updated(event, pool).await
           }
           EventType::CustomerSubscriptionDeleted => {
               handle_subscription_deleted(event, pool).await
           }
           EventType::InvoicePaymentSucceeded => {
               handle_payment_succeeded(event, pool).await
           }
           EventType::InvoicePaymentFailed => {
               handle_payment_failed(event, pool).await
           }
           _ => {
               tracing::debug!(event_type = ?event.type_, "unhandled event");
               Ok(HttpResponse::Ok().finish())
           }
       }
   }
   ```

3. Always return 200 quickly:
   - Process in background if needed
   - Never let webhook timeout
   - Log errors but don't fail

4. Idempotency:
   - Store event ID
   - Skip if already processed

Add POST /v1/webhooks/stripe route (no auth required, signature verified).
```

---

## Prompt 5.4: Subscription Event Handlers

```text
Implement handlers for subscription lifecycle events.

In src/handlers/webhook.rs:

1. checkout.session.completed:
   ```rust
   async fn handle_checkout_completed(
       event: stripe::Event,
       pool: web::Data<PgPool>,
   ) -> Result<HttpResponse, AppError> {
       let session: CheckoutSession = // extract from event

       // Get user_id from metadata
       let user_id = session.metadata.get("user_id")
           .ok_or(AppError::InternalError)?;

       // Update user subscription status
       UserRepository::update_subscription_status(
           &pool,
           user_id,
           SubscriptionStatus::Active,
       ).await?;

       // Lock price for life
       UserRepository::lock_price(
           &pool,
           user_id,
           &session.subscription_details.price_id,
           session.amount_total,
       ).await?;

       // Create audit log
       // Send welcome email
       // Create admin notification

       Ok(HttpResponse::Ok().finish())
   }
   ```

2. customer.subscription.created:
   ```rust
   async fn handle_subscription_created(
       event: stripe::Event,
       pool: web::Data<PgPool>,
   ) -> Result<HttpResponse, AppError> {
       let subscription: Subscription = // extract

       // Find user by customer_id
       let user = UserRepository::find_by_stripe_customer_id(
           &pool,
           &subscription.customer,
       ).await?;

       // Create subscription record
       SubscriptionRepository::create(&pool, CreateSubscription {
           user_id: user.id,
           stripe_subscription_id: subscription.id,
           stripe_price_id: subscription.items[0].price.id,
           status: subscription.status.into(),
           current_period_start: subscription.current_period_start,
           current_period_end: subscription.current_period_end,
           amount: subscription.items[0].price.unit_amount,
           currency: "usd".to_string(),
       }).await?;

       Ok(HttpResponse::Ok().finish())
   }
   ```

3. customer.subscription.updated:
   - Update subscription record
   - Handle status changes
   - Handle cancellation scheduling

4. customer.subscription.deleted:
   - Mark subscription as canceled
   - Update user subscription_status
   - Clear grace period if any
   - Send cancellation email

Write tests for each handler with mock events.
```

---

## Prompt 5.5: Payment Event Handlers

```text
Implement handlers for payment events.

In src/handlers/webhook.rs:

1. invoice.payment_succeeded:
   ```rust
   async fn handle_payment_succeeded(
       event: stripe::Event,
       pool: web::Data<PgPool>,
   ) -> Result<HttpResponse, AppError> {
       let invoice: Invoice = // extract

       // Find user
       let user = UserRepository::find_by_stripe_customer_id(
           &pool,
           &invoice.customer,
       ).await?;

       // Record payment
       PaymentRepository::create(&pool, CreatePayment {
           user_id: user.id,
           subscription_id: // find by stripe_subscription_id
           stripe_payment_intent_id: invoice.payment_intent,
           stripe_invoice_id: Some(invoice.id),
           amount: invoice.amount_paid,
           currency: invoice.currency,
           status: PaymentStatus::Succeeded,
           failure_reason: None,
       }).await?;

       // Clear any grace period
       if user.grace_period_start.is_some() {
           UserRepository::clear_grace_period(&pool, user.id).await?;
           UserRepository::update_subscription_status(
               &pool,
               user.id,
               SubscriptionStatus::Active,
           ).await?;
       }

       // Send receipt email
       // Create audit log

       Ok(HttpResponse::Ok().finish())
   }
   ```

2. invoice.payment_failed:
   ```rust
   async fn handle_payment_failed(
       event: stripe::Event,
       pool: web::Data<PgPool>,
   ) -> Result<HttpResponse, AppError> {
       let invoice: Invoice = // extract

       let user = UserRepository::find_by_stripe_customer_id(
           &pool,
           &invoice.customer,
       ).await?;

       // Record failed payment
       PaymentRepository::create(&pool, CreatePayment {
           user_id: user.id,
           status: PaymentStatus::Failed,
           failure_reason: invoice.last_finalization_error.map(|e| e.message),
           // ...
       }).await?;

       // Start grace period if not already started
       if user.grace_period_start.is_none() {
           let now = Utc::now();
           let grace_end = now + Duration::days(30);

           UserRepository::set_grace_period(&pool, user.id, now, grace_end).await?;
           UserRepository::update_subscription_status(
               &pool,
               user.id,
               SubscriptionStatus::GracePeriod,
           ).await?;
       }

       // Send payment failed email
       // Create admin notification

       Ok(HttpResponse::Ok().finish())
   }
   ```

Write tests with various failure scenarios.
```

---

## Prompt 5.6: Subscription Management API

```text
Implement subscription management endpoints.

Create src/handlers/subscription.rs:

1. GET /v1/subscriptions/me:
   ```rust
   #[derive(Serialize)]
   pub struct SubscriptionResponse {
       pub status: String,
       pub price_locked: bool,
       pub locked_price_amount: Option<i32>,
       pub current_period_end: Option<DateTime<Utc>>,
       pub cancel_at_period_end: bool,
       pub grace_period_end: Option<DateTime<Utc>>,
   }

   pub async fn get_subscription(
       user: AuthenticatedUser,
       pool: web::Data<PgPool>,
   ) -> Result<HttpResponse, AppError>;
   ```

2. POST /v1/subscriptions/cancel:
   ```rust
   pub async fn cancel_subscription(
       user: AuthenticatedUser,
       pool: web::Data<PgPool>,
       stripe: web::Data<StripeService>,
   ) -> Result<HttpResponse, AppError>;
   ```
   - Cancel at period end (don't cancel immediately)
   - Update database
   - Create audit log
   - Send cancellation email

3. POST /v1/subscriptions/reactivate:
   ```rust
   pub async fn reactivate_subscription(
       user: AuthenticatedUser,
       pool: web::Data<PgPool>,
       stripe: web::Data<StripeService>,
   ) -> Result<HttpResponse, AppError>;
   ```
   - Only if cancel_at_period_end is true
   - Remove cancellation
   - Create audit log

4. POST /v1/subscriptions/billing-portal:
   ```rust
   #[derive(Serialize)]
   pub struct PortalResponse {
       pub url: String,
   }

   pub async fn get_billing_portal(
       user: AuthenticatedUser,
       pool: web::Data<PgPool>,
       stripe: web::Data<StripeService>,
   ) -> Result<HttpResponse, AppError>;
   ```
   - Create Stripe billing portal session
   - Return portal URL

Create src/routes/subscription.rs and integrate.

Write integration tests for each endpoint.
```

---

## Prompt 5.7: Grace Period Handling

```text
Implement grace period logic and background jobs.

Create src/services/grace_period.rs:

1. Grace period scheduler (run daily):
   ```rust
   pub async fn check_grace_periods(pool: &PgPool) -> Result<(), AppError> {
       // Find users in grace period
       let users = UserRepository::find_in_grace_period(pool).await?;

       for user in users {
           // Check if grace period expired
           if user.grace_period_end <= Utc::now() {
               // Revoke access
               UserRepository::update_subscription_status(
                   pool,
                   user.id,
                   SubscriptionStatus::None,
               ).await?;

               // Clear grace period fields
               UserRepository::clear_grace_period(pool, user.id).await?;

               // Send final notice email
               // Create audit log
           } else {
               // Check for reminder emails
               let days_remaining = (user.grace_period_end - Utc::now()).num_days();

               match days_remaining {
                   25 | 14 | 7 | 1 => {
                       // Send reminder email
                   }
                   _ => {}
               }
           }
       }

       Ok(())
   }
   ```

2. Grace period email schedule:
   - Day 1: Payment failed, 30 days to update
   - Day 7: Reminder, 23 days remaining
   - Day 14: Reminder, 16 days remaining
   - Day 25: Urgent, 5 days remaining
   - Day 30: Final notice, access revoked

3. Create background task runner:
   ```rust
   pub async fn start_background_tasks(pool: PgPool) {
       tokio::spawn(async move {
           let mut interval = tokio::time::interval(Duration::from_secs(86400));

           loop {
               interval.tick().await;

               if let Err(e) = check_grace_periods(&pool).await {
                   tracing::error!(error = ?e, "grace period check failed");
               }

               if let Err(e) = cleanup_expired_tokens(&pool).await {
                   tracing::error!(error = ?e, "token cleanup failed");
               }
           }
       });
   }
   ```

4. Add to main.rs startup.

Write tests for grace period transitions.
```

---

## Prompt 5.8: Stripe Testing Helpers

```text
Create testing utilities for Stripe integration.

Create src/testing/stripe_mocks.rs:

1. Mock Stripe responses:
   ```rust
   pub fn mock_customer() -> stripe::Customer {
       // Return valid customer object
   }

   pub fn mock_checkout_session() -> stripe::CheckoutSession {
       // Return valid session
   }

   pub fn mock_subscription() -> stripe::Subscription {
       // Return valid subscription
   }

   pub fn mock_invoice_paid() -> stripe::Invoice {
       // Return paid invoice
   }

   pub fn mock_invoice_failed() -> stripe::Invoice {
       // Return failed invoice
   }
   ```

2. Webhook event generator:
   ```rust
   pub fn generate_webhook_event(
       event_type: EventType,
       data: serde_json::Value,
   ) -> stripe::Event {
       // Generate valid event structure
   }

   pub fn sign_webhook_payload(
       payload: &[u8],
       secret: &str,
   ) -> String {
       // Generate valid Stripe-Signature header
   }
   ```

3. Integration test helpers:
   ```rust
   pub async fn setup_test_customer(pool: &PgPool) -> (User, String) {
       // Create user with stripe_customer_id
   }

   pub async fn setup_test_subscription(
       pool: &PgPool,
       user_id: Uuid,
   ) -> Subscription {
       // Create subscription record
   }
   ```

4. Stripe CLI webhook forwarding script:
   ```bash
   #!/bin/bash
   stripe listen --forward-to localhost:8080/v1/webhooks/stripe
   ```

Write comprehensive webhook handler tests.
```

---

## Validation Checklist

After completing all prompts in this section, verify:

- [ ] Checkout session creates and redirects correctly
- [ ] Webhook signature verification works
- [ ] checkout.session.completed updates user
- [ ] Price is locked on successful subscription
- [ ] subscription.deleted revokes access
- [ ] Payment failed starts grace period
- [ ] Grace period emails scheduled correctly
- [ ] Subscription cancellation works (cancel at period end)
- [ ] Billing portal redirect works
- [ ] All events create audit logs
- [ ] Idempotency prevents duplicate processing

---

## Stripe Test Cards

For testing:
- Success: 4242 4242 4242 4242
- Decline: 4000 0000 0000 0002
- Requires auth: 4000 0025 0000 3155
- Insufficient funds: 4000 0000 0000 9995

---

## Next Steps

Proceed to **[06-frontend-foundation.md](./06-frontend-foundation.md)** to set up the React frontend.
