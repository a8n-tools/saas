//! Business logic services
//!
//! This module contains all business logic organized into services.

pub mod auth;
pub mod blob_cache;
pub mod download_cache;
pub mod download_limiter;
pub mod email;
pub mod encryption;
pub mod forgejo;
pub mod forgejo_registry;
pub mod jwt;
pub mod manifest_cache;
pub mod oci_limiter;
pub mod oci_token;
pub mod password;
pub mod release_cache;
pub mod stripe;
pub mod totp;
pub mod webhook;

// Re-export service types
pub use auth::{AcceptInviteResult, AuthService, AuthTokens, LoginResult, MagicLinkResult};
pub use blob_cache::{BlobCache, BlobHandle};
pub use download_cache::{DownloadCache, DownloadCacheError};
pub use download_limiter::{DownloadGuard, DownloadLimiter, LimitDenial};
pub use email::EmailService;
pub use encryption::EncryptionKeySet;
pub use forgejo::{ForgejoClient, ForgejoError};
pub use forgejo_registry::{ForgejoRegistryClient, RegistryError};
pub use jwt::{AccessTokenClaims, JwtConfig, JwtService, RefreshTokenClaims, TwoFactorChallengeClaims};
pub use manifest_cache::ManifestCache;
pub use oci_limiter::{OciLimitDenial, OciLimiter, OciPullGuard};
pub use oci_token::{OciTokenService, RegistryTokenClaims, REGISTRY_AUDIENCE};
pub use password::PasswordService;
pub use release_cache::ReleaseCache;
pub use stripe::{StripeConfig, StripeService};
pub use totp::TotpService;
pub use webhook::WebhookService;
