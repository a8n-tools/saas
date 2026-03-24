//! Business logic services
//!
//! This module contains all business logic organized into services.

pub mod auth;
pub mod email;
pub mod encryption;
pub mod invoice;
pub mod jwt;
pub mod password;
pub mod stripe;
pub mod totp;
pub mod webhook;

// Re-export service types
pub use auth::{AcceptInviteResult, AuthService, AuthTokens, LoginResult, MagicLinkResult};
pub use invoice::InvoiceService;
pub use email::EmailService;
pub use encryption::EncryptionKeySet;
pub use jwt::{AccessTokenClaims, JwtConfig, JwtService, RefreshTokenClaims, TwoFactorChallengeClaims};
pub use password::PasswordService;
pub use stripe::{StripeConfig, StripeService, MembershipTier};
pub use totp::TotpService;
pub use webhook::WebhookService;
