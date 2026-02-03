//! Database repository layer
//!
//! This module contains all database access logic organized by domain.

pub mod application;
pub mod audit;
pub mod membership;
pub mod notification;
pub mod payment;
pub mod rate_limit;
pub mod token;
pub mod user;

// Re-export repositories
pub use application::ApplicationRepository;
pub use audit::AuditLogRepository;
pub use membership::MembershipRepository;
pub use notification::NotificationRepository;
pub use payment::PaymentRepository;
pub use rate_limit::RateLimitRepository;
pub use token::TokenRepository;
pub use user::UserRepository;
