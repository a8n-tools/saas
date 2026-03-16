//! Database models
//!
//! This module contains all database models and data transfer objects.

pub mod application;
pub mod audit;
pub mod feedback;
pub mod membership;
pub mod rate_limit;
pub mod token;
pub mod totp;
pub mod user;

// Re-export commonly used types
pub use application::{Application, ApplicationResponse, CreateApplication, DeleteApplicationRequest, SwapApplicationOrderRequest, UpdateApplication};
pub use audit::{
    AdminNotification, AuditAction, AuditLog, AuditSeverity, CreateAdminNotification,
    CreateAuditLog, NotificationType,
};
pub use feedback::{
    AdminFeedbackDetail, AdminFeedbackSummary, ArchivedFeedbackItem, CreateFeedback,
    CreateFeedbackRequest, Feedback, FeedbackAttachmentMeta, FeedbackStatus,
    FeedbackSubmissionResponse, RespondToFeedback, RespondToFeedbackRequest,
    UpdateFeedbackStatusRequest,
};
pub use rate_limit::{RateLimit, RateLimitConfig};
pub use membership::{
    CreatePayment, CreateMembership, PaymentHistory, PaymentResponse, PaymentStatus,
    StripeSubscriptionStatus, Membership, MembershipResponse,
};
pub use token::{
    CreateEmailChangeRequest, CreateEmailVerificationToken, CreateMagicLinkToken,
    CreatePasswordResetToken, CreateRefreshToken, EmailChangeRequest, EmailVerificationToken,
    MagicLinkToken, PasswordResetToken, RefreshToken, SessionInfo,
};
pub use totp::{RecoveryCode, UserTotp};
pub use user::{CreateUser, MembershipStatus, User, UserResponse, UserRole};
