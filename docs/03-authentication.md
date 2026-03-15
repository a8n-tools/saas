# 03 - Authentication System

## Overview

This document contains prompts for implementing the complete authentication system including JWT tokens, password authentication, magic links, and session management.

## Prerequisites
- Completed 01-project-setup.md
- Completed 02-database-schema.md
- Database migrations applied

---

## Prompt 3.1: JWT Token Infrastructure

```text
Implement JWT token infrastructure using Ed25519 (EdDSA) algorithm.

Add dependencies to Cargo.toml:
- jsonwebtoken = "9"
- ed25519-dalek = "2"
- base64 = "0.21"

Create src/services/jwt.rs:

1. JwtConfig struct:
   ```rust
   pub struct JwtConfig {
       pub private_key: ed25519_dalek::SigningKey,
       pub public_key: ed25519_dalek::VerifyingKey,
       pub access_token_expiry: Duration,
       pub refresh_token_expiry: Duration,
       pub issuer: String,
   }
   ```

2. AccessTokenClaims struct:
   ```rust
   #[derive(Debug, Serialize, Deserialize)]
   pub struct AccessTokenClaims {
       pub sub: Uuid,           // user_id
       pub email: String,
       pub role: String,
       pub subscription_status: String,
       pub price_locked: bool,
       pub price_id: Option<String>,
       pub iat: i64,
       pub exp: i64,
       pub jti: String,         // unique token ID
       pub iss: String,
   }
   ```

3. RefreshTokenClaims struct (minimal):
   ```rust
   #[derive(Debug, Serialize, Deserialize)]
   pub struct RefreshTokenClaims {
       pub sub: Uuid,
       pub jti: String,
       pub exp: i64,
       pub iat: i64,
   }
   ```

4. JwtService with methods:
   ```rust
   impl JwtService {
       pub fn new(config: JwtConfig) -> Self;

       pub fn create_access_token(&self, user: &User) -> Result<String, AppError>;

       pub fn create_refresh_token(&self, user_id: Uuid) -> Result<(String, String), AppError>;
       // Returns (token, token_hash) - hash for database storage

       pub fn verify_access_token(&self, token: &str) -> Result<AccessTokenClaims, AppError>;

       pub fn verify_refresh_token(&self, token: &str) -> Result<RefreshTokenClaims, AppError>;

       pub fn decode_without_validation(&self, token: &str) -> Result<AccessTokenClaims, AppError>;
       // For getting claims from expired tokens
   }
   ```

5. Load keys from PEM files specified in environment:
   - JWT_PRIVATE_KEY_PATH
   - JWT_PUBLIC_KEY_PATH

6. Generate key pair helper script:
   ```bash
   openssl genpkey -algorithm Ed25519 -out jwt_private.pem
   openssl pkey -in jwt_private.pem -pubout -out jwt_public.pem
   ```

Write unit tests for:
- Token creation and verification
- Expired token detection
- Invalid signature detection
- Claims extraction
```

---

## Prompt 3.2: Password Hashing Service

```text
Implement secure password hashing with Argon2id.

Add dependencies:
- argon2 = "0.5"
- password-hash = "0.5"

Create src/services/password.rs:

1. PasswordService struct:
   ```rust
   pub struct PasswordService {
       // Argon2id configuration
   }

   impl PasswordService {
       pub fn new() -> Self;

       pub fn hash(&self, password: &str) -> Result<String, AppError>;

       pub fn verify(&self, password: &str, hash: &str) -> Result<bool, AppError>;

       pub fn validate_strength(&self, password: &str) -> Result<(), ValidationErrors>;
   }
   ```

2. Password validation rules:
   - Minimum 12 characters
   - Must contain at least one uppercase letter
   - Must contain at least one lowercase letter
   - Must contain at least one digit
   - Must contain at least one special character
   - Cannot be a common password (check against list)
   - Cannot contain the user's email

3. Use recommended Argon2id parameters:
   - Memory: 64 MiB
   - Iterations: 3
   - Parallelism: 4

4. ValidationErrors struct:
   ```rust
   pub struct ValidationErrors {
       pub errors: Vec<ValidationError>,
   }

   pub struct ValidationError {
       pub field: String,
       pub code: String,
       pub message: String,
   }
   ```

Write tests for:
- Hash and verify round-trip
- Invalid password detection
- Password strength validation
- Timing-safe comparison
```

---

## Prompt 3.3: Authentication Service Core

```text
Create the core authentication service that orchestrates login flows.

Create src/services/auth.rs:

1. AuthService struct:
   ```rust
   pub struct AuthService {
       pool: PgPool,
       jwt: JwtService,
       password: PasswordService,
   }
   ```

2. Registration:
   ```rust
   pub async fn register(
       &self,
       email: String,
       password: String,
       ip_address: Option<IpAddr>,
   ) -> Result<User, AppError>;
   ```
   - Validate email format
   - Validate password strength
   - Check if email already exists (conflict error)
   - Hash password
   - Create user
   - Create audit log entry
   - Return user (without password hash)

3. Login with password:
   ```rust
   pub async fn login(
       &self,
       email: String,
       password: String,
       device_info: Option<String>,
       ip_address: Option<IpAddr>,
   ) -> Result<AuthTokens, AppError>;
   ```
   - Find user by email
   - Verify password
   - Check if account is deleted (soft delete)
   - Create access token
   - Create refresh token
   - Store refresh token hash in database
   - Update last_login_at
   - Create audit log
   - Return tokens

4. AuthTokens struct:
   ```rust
   pub struct AuthTokens {
       pub access_token: String,
       pub refresh_token: String,
       pub expires_in: i64,
   }
   ```

5. Token refresh:
   ```rust
   pub async fn refresh_tokens(
       &self,
       refresh_token: String,
       device_info: Option<String>,
       ip_address: Option<IpAddr>,
   ) -> Result<AuthTokens, AppError>;
   ```
   - Verify refresh token signature
   - Find token in database by hash
   - Check not revoked and not expired
   - Get user
   - Rotate: revoke old token, create new pair
   - Return new tokens

6. Logout:
   ```rust
   pub async fn logout(
       &self,
       refresh_token: String,
       user_id: Uuid,
   ) -> Result<(), AppError>;
   ```
   - Revoke the refresh token
   - Create audit log

7. Logout all sessions:
   ```rust
   pub async fn logout_all(
       &self,
       user_id: Uuid,
   ) -> Result<(), AppError>;
   ```
   - Revoke all refresh tokens for user
   - Create audit log

Write integration tests for each flow.
```

---

## Prompt 3.4: Magic Link Authentication

```text
Implement passwordless magic link authentication.

Extend src/services/auth.rs:

1. Request magic link:
   ```rust
   pub async fn request_magic_link(
       &self,
       email: String,
       ip_address: Option<IpAddr>,
   ) -> Result<(), AppError>;
   ```
   - Generate 32 bytes random token
   - URL-safe base64 encode
   - Hash for storage
   - Store with 15 minute expiry
   - Check rate limit (3 per email per 10 minutes)
   - Queue email sending (return immediately)
   - Create audit log
   - Always return success (don't reveal if email exists)

2. Verify magic link:
   ```rust
   pub async fn verify_magic_link(
       &self,
       token: String,
       device_info: Option<String>,
       ip_address: Option<IpAddr>,
   ) -> Result<AuthTokens, AppError>;
   ```
   - Hash the token
   - Find in database
   - Check not used and not expired
   - Mark as used
   - Find or create user by email
   - Set email_verified = true
   - Create tokens
   - Create audit log
   - Return auth tokens

3. Token generation helper:
   ```rust
   fn generate_secure_token(length: usize) -> String {
       // Use rand::rngs::OsRng
       // Generate random bytes
       // Base64 URL-safe encode
   }
   ```

4. Create src/services/email.rs placeholder:
   ```rust
   pub struct EmailService;

   impl EmailService {
       pub async fn send_magic_link(
           &self,
           email: &str,
           token: &str,
       ) -> Result<(), AppError>;
   }
   ```
   - For now, just log the link to console
   - Actual email implementation in later prompt

Write tests:
- Token generation produces unique values
- Token verification works
- Expired tokens rejected
- Used tokens rejected
- Rate limiting enforced
```

---

## Prompt 3.5: Password Reset Flow

```text
Implement password reset functionality.

Extend src/services/auth.rs:

1. Request password reset:
   ```rust
   pub async fn request_password_reset(
       &self,
       email: String,
       ip_address: Option<IpAddr>,
   ) -> Result<(), AppError>;
   ```
   - Check rate limit (3 per email per hour)
   - Find user by email
   - If not found, return success (don't reveal)
   - If user has no password (magic link only), return success
   - Generate 32 byte token
   - Store hash with 1 hour expiry
   - Queue password reset email
   - Create audit log
   - Return success

2. Verify reset token (check only):
   ```rust
   pub async fn verify_reset_token(
       &self,
       token: String,
   ) -> Result<Uuid, AppError>;
   ```
   - Hash token
   - Find in database
   - Check not used and not expired
   - Return user_id (don't consume token yet)

3. Complete password reset:
   ```rust
   pub async fn complete_password_reset(
       &self,
       token: String,
       new_password: String,
       ip_address: Option<IpAddr>,
   ) -> Result<(), AppError>;
   ```
   - Hash token
   - Find and validate token
   - Validate new password strength
   - Hash new password
   - Update user password
   - Mark token as used
   - Revoke all refresh tokens (logout everywhere)
   - Create audit log
   - Send confirmation email

4. Change password (for logged-in users):
   ```rust
   pub async fn change_password(
       &self,
       user_id: Uuid,
       current_password: String,
       new_password: String,
       ip_address: Option<IpAddr>,
   ) -> Result<(), AppError>;
   ```
   - Get user
   - Verify current password
   - Validate new password strength
   - Hash and update password
   - Optionally revoke other sessions
   - Create audit log

Write tests for each scenario including edge cases.
```

---

## Prompt 3.6: Authentication Middleware

```text
Create Actix-Web middleware for JWT authentication.

Create src/middleware/auth.rs:

1. JwtAuth middleware:
   ```rust
   pub struct JwtAuth {
       jwt_service: Arc<JwtService>,
   }

   impl JwtAuth {
       pub fn new(jwt_service: Arc<JwtService>) -> Self;
   }
   ```

2. Implement Transform and Service traits to:
   - Extract JWT from cookie named "access_token"
   - Also support Authorization: Bearer header
   - Verify token
   - Store claims in request extensions
   - Return 401 if invalid/missing

3. AuthenticatedUser extractor:
   ```rust
   pub struct AuthenticatedUser(pub AccessTokenClaims);

   impl FromRequest for AuthenticatedUser {
       // Extract from request extensions
       // Return 401 if not present
   }
   ```

4. OptionalUser extractor:
   ```rust
   pub struct OptionalUser(pub Option<AccessTokenClaims>);

   impl FromRequest for OptionalUser {
       // Like AuthenticatedUser but returns None instead of error
   }
   ```

5. RequireAdmin guard:
   ```rust
   pub struct RequireAdmin;

   impl Guard for RequireAdmin {
       fn check(&self, ctx: &GuardContext<'_>) -> bool {
           // Check claims in extensions have role = "admin"
       }
   }
   ```

6. RequireActiveSubscription guard:
   ```rust
   pub struct RequireActiveSubscription;

   impl Guard for RequireActiveSubscription {
       fn check(&self, ctx: &GuardContext<'_>) -> bool {
           // Check subscription_status in ["active", "grace_period"]
       }
   }
   ```

7. Cookie helpers:
   ```rust
   pub fn create_auth_cookies(tokens: &AuthTokens, remember: bool) -> Vec<Cookie>;
   pub fn clear_auth_cookies() -> Vec<Cookie>;
   ```
   - Domain: .example.com
   - HttpOnly: true
   - Secure: true (configurable for dev)
   - SameSite: Lax
   - Path: /

Write tests with mock requests.
```

---

## Prompt 3.7: Authentication API Handlers

```text
Create the authentication API endpoints.

Create src/handlers/auth.rs:

1. POST /v1/auth/register
   ```rust
   #[derive(Deserialize, Validate)]
   pub struct RegisterRequest {
       #[validate(email)]
       pub email: String,
       #[validate(length(min = 12))]
       pub password: String,
   }

   pub async fn register(
       pool: web::Data<PgPool>,
       auth_service: web::Data<AuthService>,
       req: web::Json<RegisterRequest>,
       http_req: HttpRequest,
   ) -> Result<HttpResponse, AppError>;
   ```
   - Validate request
   - Call auth service
   - Return user (no auto-login)

2. POST /v1/auth/login
   ```rust
   #[derive(Deserialize)]
   pub struct LoginRequest {
       pub email: String,
       pub password: String,
       pub remember: Option<bool>,
   }

   pub async fn login(...) -> Result<HttpResponse, AppError>;
   ```
   - Extract device info from User-Agent
   - Extract IP address
   - Call auth service
   - Set cookies
   - Return user info

3. POST /v1/auth/magic-link
   ```rust
   #[derive(Deserialize)]
   pub struct MagicLinkRequest {
       pub email: String,
   }

   pub async fn request_magic_link(...) -> Result<HttpResponse, AppError>;
   ```
   - Always return success (202 Accepted)

4. POST /v1/auth/magic-link/verify
   ```rust
   #[derive(Deserialize)]
   pub struct VerifyMagicLinkRequest {
       pub token: String,
   }

   pub async fn verify_magic_link(...) -> Result<HttpResponse, AppError>;
   ```
   - Set cookies
   - Return user info

5. POST /v1/auth/refresh
   ```rust
   pub async fn refresh_token(...) -> Result<HttpResponse, AppError>;
   ```
   - Read refresh token from cookie
   - Call auth service
   - Set new cookies

6. POST /v1/auth/logout
   ```rust
   pub async fn logout(...) -> Result<HttpResponse, AppError>;
   ```
   - Clear cookies
   - Revoke refresh token

7. POST /v1/auth/password-reset
8. POST /v1/auth/password-reset/confirm

Create src/routes/auth.rs to configure all routes.

Add to main.rs route configuration.

Write integration tests for each endpoint.
```

---

## Prompt 3.8: User API Handlers

```text
Create user management endpoints.

Create src/handlers/user.rs:

1. GET /v1/users/me
   ```rust
   pub async fn get_current_user(
       user: AuthenticatedUser,
       pool: web::Data<PgPool>,
   ) -> Result<HttpResponse, AppError>;
   ```
   - Get fresh user data from database
   - Return UserResponse (no password hash)

2. PUT /v1/users/me/password
   ```rust
   #[derive(Deserialize)]
   pub struct ChangePasswordRequest {
       pub current_password: String,
       pub new_password: String,
   }

   pub async fn change_password(
       user: AuthenticatedUser,
       auth_service: web::Data<AuthService>,
       req: web::Json<ChangePasswordRequest>,
   ) -> Result<HttpResponse, AppError>;
   ```

3. GET /v1/users/me/sessions
   ```rust
   pub async fn list_sessions(
       user: AuthenticatedUser,
       pool: web::Data<PgPool>,
   ) -> Result<HttpResponse, AppError>;
   ```
   - Return list of active refresh tokens
   - Include device info, IP, last used

4. DELETE /v1/users/me/sessions/:id
   ```rust
   pub async fn revoke_session(
       user: AuthenticatedUser,
       session_id: web::Path<Uuid>,
       pool: web::Data<PgPool>,
   ) -> Result<HttpResponse, AppError>;
   ```
   - Revoke specific session
   - Cannot revoke current session (use logout)

Create src/routes/user.rs and add to main configuration.

Write integration tests.
```

---

## Validation Checklist

After completing all prompts in this section, verify:

- [ ] Registration creates user with hashed password
- [ ] Login returns valid JWT tokens
- [ ] Access token expires in 15 minutes
- [ ] Refresh token rotates correctly
- [ ] Magic link flow works end-to-end
- [ ] Password reset flow works
- [ ] Middleware correctly extracts user from JWT
- [ ] Guards enforce admin/subscription requirements
- [ ] Cookies set correctly with proper flags
- [ ] Rate limiting prevents abuse
- [ ] All endpoints return correct error formats
- [ ] Audit logs created for security events

---

## Next Steps

Proceed to **[04-api-core.md](./04-api-core.md)** to implement core API endpoints.
