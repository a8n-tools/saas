//! Request validation utilities

use validator::ValidationError;

/// Validation rules constants
pub struct ValidationRules;

impl ValidationRules {
    pub const EMAIL_MAX_LENGTH: usize = 255;
    pub const PASSWORD_MIN_LENGTH: usize = 12;
    pub const PASSWORD_MAX_LENGTH: usize = 128;
    pub const SLUG_PATTERN: &'static str = r"^[a-z0-9-]+$";
}

/// Common password list for strength validation
const COMMON_PASSWORDS: &[&str] = &[
    "password1234",
    "123456789012",
    "qwertyuiopas",
    "administrator",
];

/// Validate email format
pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    if email.is_empty() {
        return Err(ValidationError::new("email_required"));
    }

    if email.len() > ValidationRules::EMAIL_MAX_LENGTH {
        return Err(ValidationError::new("email_too_long"));
    }

    // Basic email format check
    if !email.contains('@') || !email.contains('.') {
        return Err(ValidationError::new("invalid_email_format"));
    }

    Ok(())
}

/// Validate password strength
pub fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    if password.len() < ValidationRules::PASSWORD_MIN_LENGTH {
        let mut err = ValidationError::new("password_too_short");
        err.message = Some(
            format!(
                "Password must be at least {} characters",
                ValidationRules::PASSWORD_MIN_LENGTH
            )
            .into(),
        );
        return Err(err);
    }

    if password.len() > ValidationRules::PASSWORD_MAX_LENGTH {
        return Err(ValidationError::new("password_too_long"));
    }

    // Check for uppercase
    if !password.chars().any(|c| c.is_uppercase()) {
        let mut err = ValidationError::new("password_no_uppercase");
        err.message = Some("Password must contain at least one uppercase letter".into());
        return Err(err);
    }

    // Check for lowercase
    if !password.chars().any(|c| c.is_lowercase()) {
        let mut err = ValidationError::new("password_no_lowercase");
        err.message = Some("Password must contain at least one lowercase letter".into());
        return Err(err);
    }

    // Check for digit
    if !password.chars().any(|c| c.is_numeric()) {
        let mut err = ValidationError::new("password_no_digit");
        err.message = Some("Password must contain at least one number".into());
        return Err(err);
    }

    // Check for special character
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        let mut err = ValidationError::new("password_no_special");
        err.message = Some("Password must contain at least one special character".into());
        return Err(err);
    }

    // Check against common passwords
    if COMMON_PASSWORDS.contains(&password.to_lowercase().as_str()) {
        let mut err = ValidationError::new("password_too_common");
        err.message = Some("Password is too common".into());
        return Err(err);
    }

    Ok(())
}

/// Validate UUID format
pub fn validate_uuid(id: &str) -> Result<(), ValidationError> {
    uuid::Uuid::parse_str(id).map_err(|_| ValidationError::new("invalid_uuid"))?;
    Ok(())
}

/// Validate slug format
pub fn validate_slug(slug: &str) -> Result<(), ValidationError> {
    if slug.is_empty() {
        return Err(ValidationError::new("slug_required"));
    }

    if !slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(ValidationError::new("invalid_slug_format"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("invalid").is_err());
        assert!(validate_email("").is_err());
    }

    #[test]
    fn test_validate_password_strength() {
        // Valid password
        assert!(validate_password_strength("SecurePass123!").is_ok());

        // Too short
        assert!(validate_password_strength("Short1!").is_err());

        // No uppercase
        assert!(validate_password_strength("lowercase123!").is_err());

        // No lowercase
        assert!(validate_password_strength("UPPERCASE123!").is_err());

        // No digit
        assert!(validate_password_strength("NoDigitsHere!").is_err());

        // No special char
        assert!(validate_password_strength("NoSpecial123").is_err());
    }

    #[test]
    fn test_validate_uuid() {
        assert!(validate_uuid("550e8400-e29b-41d4-a716-446655440000").is_ok());
        assert!(validate_uuid("invalid-uuid").is_err());
    }

    #[test]
    fn test_validate_slug() {
        assert!(validate_slug("valid-slug-123").is_ok());
        assert!(validate_slug("Invalid Slug").is_err());
        assert!(validate_slug("").is_err());
    }
}
