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
    /// Email configuration
    pub email: EmailConfig,
    /// Cookie domain (e.g., ".a8n.tools" for production, empty for localhost)
    pub cookie_domain: Option<String>,
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
            from_email: env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@a8n.tools".to_string()),
            from_name: env::var("EMAIL_FROM_NAME").unwrap_or_else(|_| "a8n.tools".to_string()),
            base_url: env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:5173".to_string()),
            enabled: (is_production && has_smtp) || force_enabled,
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

        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|_| ConfigError::InvalidValue("PORT".to_string(), "must be a valid port number".to_string()))?;

        let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        let cors_origin = env::var("CORS_ORIGIN")
            .unwrap_or_else(|_| "https://app.a8n.tools".to_string());

        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        let is_production = environment == "production";
        let email = EmailConfig::from_env(is_production);

        // Cookie domain: use .a8n.tools in production, None for localhost in development
        let cookie_domain = env::var("COOKIE_DOMAIN").ok().filter(|s| !s.is_empty());
        let cookie_domain = if cookie_domain.is_none() && is_production {
            Some(".a8n.tools".to_string())
        } else {
            cookie_domain
        };

        let config = Self {
            database_url,
            host,
            port,
            log_level,
            cors_origin,
            environment,
            email,
            cookie_domain,
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
        // Set required env var
        env::set_var("DATABASE_URL", "postgres://test:test@localhost/test");
        env::remove_var("HOST");
        env::remove_var("PORT");
        env::remove_var("RUST_LOG");
        env::remove_var("CORS_ORIGIN");
        env::remove_var("ENVIRONMENT");
        env::remove_var("SMTP_HOST");
        env::remove_var("COOKIE_DOMAIN");

        let config = Config::from_env().unwrap();

        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert_eq!(config.log_level, "info");
        assert_eq!(config.cors_origin, "https://app.a8n.tools");
        assert_eq!(config.environment, "development");
        assert!(!config.email.enabled);
        // In development mode without COOKIE_DOMAIN set, it should be None (for localhost)
        assert!(config.cookie_domain.is_none());
    }

    #[test]
    fn test_missing_database_url() {
        env::remove_var("DATABASE_URL");

        let result = Config::from_env();
        assert!(result.is_err());
    }
}
