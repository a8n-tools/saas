//! Database repository layer
//!
//! This module contains all database access logic organized by domain.

pub mod application;
pub mod audit;
pub mod feedback;
pub mod invite;
pub mod membership;
pub mod notification;
pub mod payment;
pub mod rate_limit;
pub mod stripe;
pub mod token;
pub mod totp;
pub mod user;

// Re-export repositories
pub use application::ApplicationRepository;
pub use audit::AuditLogRepository;
pub use feedback::FeedbackRepository;
pub use invite::InviteRepository;
pub use membership::MembershipRepository;
pub use notification::NotificationRepository;
pub use payment::PaymentRepository;
pub use rate_limit::RateLimitRepository;
pub use stripe::StripeConfigRepository;
pub use token::TokenRepository;
pub use totp::TotpRepository;
pub use user::UserRepository;
