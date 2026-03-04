//! Business logic services
//!
//! This module contains all business logic organized into services.

pub mod auth;
pub mod email;
pub mod jwt;
pub mod password;
pub mod stripe;
pub mod totp;

// Re-export service types
pub use auth::{AuthService, AuthTokens, LoginResult, MagicLinkResult};
pub use email::EmailService;
pub use jwt::{AccessTokenClaims, JwtConfig, JwtService, RefreshTokenClaims, TwoFactorChallengeClaims};
pub use password::PasswordService;
pub use stripe::{StripeConfig, StripeService, MembershipTier};
pub use totp::TotpService;
