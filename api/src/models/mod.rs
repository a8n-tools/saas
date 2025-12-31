//! Database models
//!
//! This module contains all database models and data transfer objects.

pub mod application;
pub mod audit;
pub mod rate_limit;
pub mod subscription;
pub mod token;
pub mod user;

// Re-export commonly used types
pub use application::{Application, ApplicationResponse, CreateApplication};
pub use audit::{
    AdminNotification, AuditAction, AuditLog, AuditSeverity, CreateAdminNotification,
    CreateAuditLog, NotificationType,
};
pub use rate_limit::{RateLimit, RateLimitConfig};
pub use subscription::{
    CreatePayment, CreateSubscription, PaymentHistory, PaymentResponse, PaymentStatus,
    StripeSubscriptionStatus, Subscription, SubscriptionResponse,
};
pub use token::{
    CreateMagicLinkToken, CreatePasswordResetToken, CreateRefreshToken, MagicLinkToken,
    PasswordResetToken, RefreshToken, SessionInfo,
};
pub use user::{CreateUser, SubscriptionStatus, User, UserResponse, UserRole};
