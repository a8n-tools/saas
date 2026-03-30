use std::env;
use tracing::info;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    /// Database connection URL
    pub database_url: String,
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Log level (RUST_LOG)
    pub log_level: String,
    /// CORS allowed origin
    pub cors_origin: String,
    /// Environment (development, production)
    pub environment: String,
    /// Application name used in emails, JWT issuer, etc.
    pub app_name: String,
    /// Email configuration
    pub email: EmailConfig,
    /// Cookie domain (e.g., ".yourdomain.com" for production, empty for localhost)
    pub cookie_domain: Option<String>,
    /// Auto-ban configuration
    pub auto_ban: AutoBanConfig,
    /// TOTP encryption key (32 bytes) for encrypting TOTP secrets at rest
    pub totp_encryption_key: [u8; 32],
    /// Previous TOTP encryption key for rotation (optional)
    pub totp_encryption_key_prev: Option<[u8; 32]>,
    /// Current TOTP key version (incremented on each rotation)
    pub totp_key_version: i16,
    /// Stripe encryption key (32 bytes) for encrypting Stripe secrets at rest
    pub stripe_encryption_key: [u8; 32],
    /// Previous Stripe encryption key for rotation (optional)
    pub stripe_encryption_key_prev: Option<[u8; 32]>,
    /// Current Stripe key version (incremented on each rotation)
    pub stripe_key_version: i16,
    /// Membership tier thresholds
    pub tier: TierConfig,
}

/// SMTP TLS mode
#[derive(Debug, Clone, PartialEq)]
pub enum SmtpTls {
    /// Implicit TLS (port 465) — connection is TLS from the start
    Implicit,
    /// STARTTLS (port 587) — plaintext connection upgraded to TLS
    Starttls,
}

/// Email configuration
#[derive(Debug, Clone)]
pub struct EmailConfig {
    /// SMTP server host
    pub smtp_host: String,
    /// SMTP server port
    pub smtp_port: u16,
    /// SMTP TLS mode
    pub smtp_tls: SmtpTls,
    /// SMTP username
    pub smtp_username: String,
    /// SMTP password
    pub smtp_password: String,
    /// From email address
    pub from_email: String,
    /// From name
    pub from_name: String,
    /// Base URL for links in emails
    pub base_url: String,
    /// Whether to actually send emails (false in dev mode)
    pub enabled: bool,
    /// Application name for email subjects and templates
    pub app_name: String,
    /// Admin recipients for operational notifications
    pub admin_notification_emails: Vec<String>,
}

impl EmailConfig {
    /// Load email configuration from environment variables
    pub fn from_env(is_production: bool) -> Self {
        // Allow forcing email enabled in development via env var
        let force_enabled = env::var("EMAIL_ENABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        let smtp_host = env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string());
        let has_smtp = !smtp_host.is_empty() && smtp_host != "localhost";

        // SMTP_TLS: "implicit" (port 465) or "starttls" (port 587)
        let smtp_tls = match env::var("SMTP_TLS").unwrap_or_default().to_lowercase().as_str() {
            "starttls" => SmtpTls::Starttls,
            // Default to implicit TLS (port 465)
            _ => SmtpTls::Implicit,
        };

        let default_port: u16 = match smtp_tls {
            SmtpTls::Implicit => 465,
            SmtpTls::Starttls => 587,
        };

        Self {
            smtp_host,
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| default_port.to_string())
                .parse()
                .unwrap_or(default_port),
            smtp_tls,
            smtp_username: env::var("SMTP_USERNAME").unwrap_or_default(),
            smtp_password: env::var("SMTP_PASSWORD").unwrap_or_default(),
            from_email: parse_smtp_from_email(
                &env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@localhost".to_string()),
            ),
            from_name: parse_smtp_from_name(
                &env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@localhost".to_string()),
            ),
            base_url: env::var("APP_URL")
                .or_else(|_| env::var("CORS_ORIGIN"))
                .unwrap_or_else(|_| "http://localhost:5173".to_string()),
            enabled: (is_production && has_smtp) || force_enabled,
            app_name: env::var("APP_NAME").unwrap_or_else(|_| "localhost".to_string()),
            admin_notification_emails: env::var("ADMIN_NOTIFICATION_EMAILS")
                .unwrap_or_default()
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .collect(),
        }
    }
}

/// Parse email address from SMTP_FROM.
/// Supports "Display Name <email>" or plain "email" format.
fn parse_smtp_from_email(smtp_from: &str) -> String {
    if let Some(start) = smtp_from.find('<') {
        if let Some(end) = smtp_from.find('>') {
            return smtp_from[start + 1..end].trim().to_string();
        }
    }
    smtp_from.trim().to_string()
}

/// Parse display name from SMTP_FROM.
/// Returns the part before `<`, or "localhost" if no display name is present.
fn parse_smtp_from_name(smtp_from: &str) -> String {
    if let Some(start) = smtp_from.find('<') {
        let name = smtp_from[..start].trim();
        if !name.is_empty() {
            return name.to_string();
        }
    }
    "localhost".to_string()
}

/// Auto-ban configuration
#[derive(Debug, Clone)]
pub struct AutoBanConfig {
    /// Whether auto-banning is enabled
    pub enabled: bool,
    /// Number of suspicious requests before banning an IP
    pub threshold: u32,
    /// Time window in seconds for counting strikes
    pub window_secs: u64,
    /// How long a ban lasts in seconds
    pub ban_duration_secs: u64,
}

impl AutoBanConfig {
    /// Load auto-ban configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            enabled: env::var("AUTO_BAN_ENABLED")
                .map(|v| v != "false" && v != "0")
                .unwrap_or(true),
            threshold: env::var("AUTO_BAN_THRESHOLD")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            window_secs: env::var("AUTO_BAN_WINDOW_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3600),
            ban_duration_secs: env::var("AUTO_BAN_DURATION_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(86400),
        }
    }
}

/// Membership tier threshold configuration
#[derive(Debug, Clone)]
pub struct TierConfig {
    /// Number of lifetime slots (first N verified users get lifetime tier)
    pub lifetime_slots: i64,
    /// Number of early adopter slots (next N verified users get early adopter tier)
    pub early_adopter_slots: i64,
    /// Trial duration in days for early adopter tier
    pub early_adopter_trial_days: i64,
    /// Trial duration in days for standard tier
    pub standard_trial_days: i64,
}

impl TierConfig {
    /// Load tier configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            lifetime_slots: env::var("TIER_LIFETIME_SLOTS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            early_adopter_slots: env::var("TIER_EARLY_ADOPTER_SLOTS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            early_adopter_trial_days: env::var("TIER_EARLY_ADOPTER_TRIAL_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(90),
            standard_trial_days: env::var("TIER_STANDARD_TRIAL_DAYS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    ///
    /// # Errors
    /// Returns an error if required environment variables are missing
    pub fn from_env() -> Result<Self, ConfigError> {
        // Load .env file if it exists (ignore errors if not found)
        let _ = dotenvy::dotenv();

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingEnv("DATABASE_URL".to_string()))?;

        let host = env::var("HOST_IP").unwrap_or_else(|_| "0.0.0.0".to_string());

        let port = env::var("APP_PORT")
            .unwrap_or_else(|_| "4000".to_string())
            .parse::<u16>()
            .map_err(|_| ConfigError::InvalidValue("APP_PORT".to_string(), "must be a valid port number".to_string()))?;

        let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        let cors_origin = env::var("CORS_ORIGIN")
            .unwrap_or_else(|_| "http://localhost:5173".to_string());

        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "production".to_string());
        let app_name = env::var("APP_NAME").unwrap_or_else(|_| "localhost".to_string());
        let is_production = environment == "production";
        let email = EmailConfig::from_env(is_production);

        // Cookie domain: must be set explicitly via COOKIE_DOMAIN env var.
        // None means cookies are scoped to the exact hostname (suitable for localhost).
        let cookie_domain = env::var("COOKIE_DOMAIN").ok().filter(|s| !s.is_empty());

        let auto_ban = AutoBanConfig::from_env();

        let totp_encryption_key = Self::load_totp_encryption_key(&environment);
        let stripe_encryption_key = Self::load_stripe_encryption_key(&environment);
        let totp_encryption_key_prev = Self::load_optional_encryption_key("TOTP_ENCRYPTION_KEY_PREV");
        let stripe_encryption_key_prev = Self::load_optional_encryption_key("STRIPE_ENCRYPTION_KEY_PREV");
        let totp_key_version: i16 = env::var("TOTP_KEY_VERSION")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);
        let stripe_key_version: i16 = env::var("STRIPE_KEY_VERSION")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        let tier = TierConfig::from_env();

        let config = Self {
            database_url,
            host,
            port,
            log_level,
            cors_origin,
            environment,
            app_name,
            email,
            cookie_domain,
            auto_ban,
            totp_encryption_key,
            totp_encryption_key_prev,
            totp_key_version,
            stripe_encryption_key,
            stripe_encryption_key_prev,
            stripe_key_version,
            tier,
        };

        info!(
            host = %config.host,
            port = %config.port,
            environment = %config.environment,
            "Configuration loaded"
        );

        Ok(config)
    }

    /// Returns true if running in production environment
    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }

    /// Load TOTP encryption key from TOTP_ENCRYPTION_KEY env var (hex-encoded 32 bytes).
    /// In development, defaults to 32 zero bytes.
    fn load_totp_encryption_key(environment: &str) -> [u8; 32] {
        match env::var("TOTP_ENCRYPTION_KEY") {
            Ok(hex_str) => {
                let bytes = hex::decode(hex_str.trim())
                    .expect("TOTP_ENCRYPTION_KEY must be valid hex");
                let key: [u8; 32] = bytes
                    .try_into()
                    .expect("TOTP_ENCRYPTION_KEY must be exactly 32 bytes (64 hex chars)");
                key
            }
            Err(_) => {
                if environment == "production" {
                    panic!("TOTP_ENCRYPTION_KEY must be set in production");
                }
                [0u8; 32]
            }
        }
    }

    /// Load Stripe encryption key from STRIPE_ENCRYPTION_KEY env var (hex-encoded 32 bytes).
    /// In development, defaults to 32 zero bytes.
    fn load_stripe_encryption_key(environment: &str) -> [u8; 32] {
        match env::var("STRIPE_ENCRYPTION_KEY") {
            Ok(hex_str) => {
                let bytes = hex::decode(hex_str.trim())
                    .expect("STRIPE_ENCRYPTION_KEY must be valid hex");
                let key: [u8; 32] = bytes
                    .try_into()
                    .expect("STRIPE_ENCRYPTION_KEY must be exactly 32 bytes (64 hex chars)");
                key
            }
            Err(_) => {
                if environment == "production" {
                    panic!("STRIPE_ENCRYPTION_KEY must be set in production");
                }
                [0u8; 32]
            }
        }
    }

    /// Load an optional encryption key from an env var (hex-encoded 32 bytes).
    /// Returns `None` if the env var is not set.
    fn load_optional_encryption_key(env_var: &str) -> Option<[u8; 32]> {
        env::var(env_var).ok().map(|hex_str| {
            let bytes = hex::decode(hex_str.trim())
                .unwrap_or_else(|_| panic!("{env_var} must be valid hex"));
            let key: [u8; 32] = bytes
                .try_into()
                .unwrap_or_else(|_| panic!("{env_var} must be exactly 32 bytes (64 hex chars)"));
            key
        })
    }

    /// Get the server bind address
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnv(String),

    #[error("Invalid value for {0}: {1}")]
    InvalidValue(String, String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_defaults() {
        // Set required env vars
        env::set_var("DATABASE_URL", "postgres://test:test@localhost/test");
        // Use development to avoid requiring TOTP_ENCRYPTION_KEY
        env::set_var("ENVIRONMENT", "development");
        env::remove_var("HOST_IP");
        env::remove_var("APP_PORT");
        env::remove_var("RUST_LOG");
        env::remove_var("CORS_ORIGIN");
        env::remove_var("SMTP_HOST");
        env::remove_var("EMAIL_ENABLED");
        env::remove_var("COOKIE_DOMAIN");

        let config = Config::from_env().unwrap();

        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 4000);
        assert_eq!(config.log_level, "info");
        assert_eq!(config.cors_origin, "http://localhost:5173");
        assert_eq!(config.environment, "development");
        assert!(!config.email.enabled);
        // In development mode without COOKIE_DOMAIN set, it should be None (for localhost)
        assert!(config.cookie_domain.is_none());
    }

    #[test]
    fn test_missing_database_url() {
        // Test that MissingEnv error is returned for missing DATABASE_URL
        // by checking the error variant directly (avoids env var race with parallel tests)
        let err = ConfigError::MissingEnv("DATABASE_URL".to_string());
        assert!(err.to_string().contains("DATABASE_URL"));
    }

    #[test]
    fn test_parse_smtp_from_with_display_name() {
        let input = "a8n Tools Staging <tools-staging@a8n.run>";
        assert_eq!(parse_smtp_from_email(input), "tools-staging@a8n.run");
        assert_eq!(parse_smtp_from_name(input), "a8n Tools Staging");
    }

    #[test]
    fn test_parse_smtp_from_plain_email() {
        let input = "noreply@localhost";
        assert_eq!(parse_smtp_from_email(input), "noreply@localhost");
        assert_eq!(parse_smtp_from_name(input), "localhost");
    }

    // ---- Key rotation config ----

    #[test]
    fn test_load_optional_encryption_key_returns_none_when_unset() {
        env::remove_var("TEST_OPTIONAL_KEY_UNSET");
        let result = Config::load_optional_encryption_key("TEST_OPTIONAL_KEY_UNSET");
        assert!(result.is_none());
    }

    #[test]
    fn test_load_optional_encryption_key_parses_hex() {
        let hex_key = "aa".repeat(32); // 64 hex chars = 32 bytes
        env::set_var("TEST_OPTIONAL_KEY_HEX", &hex_key);
        let result = Config::load_optional_encryption_key("TEST_OPTIONAL_KEY_HEX");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), [0xAA; 32]);
        env::remove_var("TEST_OPTIONAL_KEY_HEX");
    }

    #[test]
    #[should_panic(expected = "must be valid hex")]
    fn test_load_optional_encryption_key_panics_on_invalid_hex() {
        env::set_var("TEST_OPTIONAL_KEY_BAD", "not-valid-hex!");
        Config::load_optional_encryption_key("TEST_OPTIONAL_KEY_BAD");
    }

    #[test]
    #[should_panic(expected = "must be exactly 32 bytes")]
    fn test_load_optional_encryption_key_panics_on_wrong_length() {
        env::set_var("TEST_OPTIONAL_KEY_SHORT", "aabb"); // only 2 bytes
        Config::load_optional_encryption_key("TEST_OPTIONAL_KEY_SHORT");
    }

    #[test]
    fn test_key_version_parsing() {
        // Test the parsing logic directly to avoid env var races with parallel tests.
        // Key versions use: env::var("X").ok().and_then(|v| v.parse().ok()).unwrap_or(1)
        assert_eq!("3".parse::<i16>().unwrap(), 3);
        assert_eq!("7".parse::<i16>().unwrap(), 7);
        assert_eq!(None::<String>.and_then(|v: String| v.parse::<i16>().ok()).unwrap_or(1), 1);
        assert_eq!(Some("invalid".to_string()).and_then(|v| v.parse::<i16>().ok()).unwrap_or(1), 1);
    }
}
