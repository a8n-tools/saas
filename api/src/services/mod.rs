//! Business logic services
//!
//! This module contains all business logic organized into services.

pub mod auth;
pub mod download_limiter;
pub mod email;
pub mod encryption;
pub mod forgejo;
pub mod jwt;
pub mod password;
pub mod release_cache;
pub mod stripe;
pub mod totp;
pub mod webhook;

// Re-export service types
pub use auth::{AcceptInviteResult, AuthService, AuthTokens, LoginResult, MagicLinkResult};
pub use download_limiter::{DownloadGuard, DownloadLimiter, LimitDenial};
pub use email::EmailService;
pub use encryption::EncryptionKeySet;
pub use forgejo::{ForgejoClient, ForgejoError};
pub use jwt::{AccessTokenClaims, JwtConfig, JwtService, RefreshTokenClaims, TwoFactorChallengeClaims};
pub use password::PasswordService;
pub use release_cache::ReleaseCache;
pub use stripe::{StripeConfig, StripeService};
pub use totp::TotpService;
pub use webhook::WebhookService;
