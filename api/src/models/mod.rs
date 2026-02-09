//! Database models
//!
//! This module contains all database models and data transfer objects.

pub mod application;
pub mod audit;
pub mod membership;
pub mod rate_limit;
pub mod token;
pub mod user;

// Re-export commonly used types
pub use application::{Application, ApplicationResponse, CreateApplication};
pub use audit::{
    AdminNotification, AuditAction, AuditLog, AuditSeverity, CreateAdminNotification,
    CreateAuditLog, NotificationType,
};
pub use rate_limit::{RateLimit, RateLimitConfig};
pub use membership::{
    CreatePayment, CreateMembership, PaymentHistory, PaymentResponse, PaymentStatus,
    StripeSubscriptionStatus, Membership, MembershipResponse,
};
pub use token::{
    CreateMagicLinkToken, CreatePasswordResetToken, CreateRefreshToken, MagicLinkToken,
    PasswordResetToken, RefreshToken, SessionInfo,
};
pub use user::{CreateUser, MembershipStatus, User, UserResponse, UserRole};
