//! Authentication middleware and extractors
//!
//! This module provides JWT-based authentication middleware and extractors
//! for securing API endpoints.

use actix_web::{
    cookie::{Cookie, SameSite},
    dev::Payload,
    http::header,
    FromRequest, HttpMessage, HttpRequest,
};
use std::future::{ready, Ready};
use std::sync::Arc;

use crate::errors::AppError;
use crate::services::{AccessTokenClaims, JwtService};

/// Key for storing authenticated user claims in request extensions
#[derive(Debug, Clone)]
pub struct AuthenticatedClaims(pub AccessTokenClaims);

/// Extractor for authenticated users - returns 401 if not authenticated
#[derive(Debug, Clone)]
pub struct AuthenticatedUser(pub AccessTokenClaims);

impl FromRequest for AuthenticatedUser {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // Try to get JWT service from app data
        let jwt_service = match req.app_data::<Arc<JwtService>>() {
            Some(service) => service.clone(),
            None => {
                tracing::error!("JwtService not found in app data");
                return ready(Err(AppError::internal("Authentication service not available")));
            }
        };

        // Try to extract token from cookie first, then Authorization header
        let token = extract_token(req);

        match token {
            Some(token) => match jwt_service.verify_access_token(&token) {
                Ok(claims) => {
                    // Store claims in request extensions for later use
                    req.extensions_mut().insert(AuthenticatedClaims(claims.clone()));
                    ready(Ok(AuthenticatedUser(claims)))
                }
                Err(e) => ready(Err(e)),
            },
            None => ready(Err(AppError::Unauthorized)),
        }
    }
}

/// Extractor for optionally authenticated users - returns None if not authenticated
#[derive(Debug, Clone)]
pub struct OptionalUser(pub Option<AccessTokenClaims>);

impl FromRequest for OptionalUser {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // Try to get JWT service from app data
        let jwt_service = match req.app_data::<Arc<JwtService>>() {
            Some(service) => service.clone(),
            None => {
                tracing::warn!("JwtService not found in app data for optional auth");
                return ready(Ok(OptionalUser(None)));
            }
        };

        // Try to extract token
        let token = extract_token(req);

        match token {
            Some(token) => match jwt_service.verify_access_token(&token) {
                Ok(claims) => {
                    req.extensions_mut().insert(AuthenticatedClaims(claims.clone()));
                    ready(Ok(OptionalUser(Some(claims))))
                }
                Err(_) => ready(Ok(OptionalUser(None))),
            },
            None => ready(Ok(OptionalUser(None))),
        }
    }
}

/// Extractor for admin users - returns 403 if not admin
#[derive(Debug, Clone)]
pub struct AdminUser(pub AccessTokenClaims);

impl FromRequest for AdminUser {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // Try to get JWT service from app data
        let jwt_service = match req.app_data::<Arc<JwtService>>() {
            Some(service) => service.clone(),
            None => {
                tracing::error!("JwtService not found in app data");
                return ready(Err(AppError::internal("Authentication service not available")));
            }
        };

        // Try to extract token
        let token = extract_token(req);

        match token {
            Some(token) => match jwt_service.verify_access_token(&token) {
                Ok(claims) => {
                    if claims.role != "admin" {
                        return ready(Err(AppError::Forbidden));
                    }
                    req.extensions_mut().insert(AuthenticatedClaims(claims.clone()));
                    ready(Ok(AdminUser(claims)))
                }
                Err(e) => ready(Err(e)),
            },
            None => ready(Err(AppError::Unauthorized)),
        }
    }
}

/// Extractor for users with active subscription - returns 403 if not subscribed
#[derive(Debug, Clone)]
pub struct SubscribedUser(pub AccessTokenClaims);

impl FromRequest for SubscribedUser {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let jwt_service = match req.app_data::<Arc<JwtService>>() {
            Some(service) => service.clone(),
            None => {
                tracing::error!("JwtService not found in app data");
                return ready(Err(AppError::internal("Authentication service not available")));
            }
        };

        let token = extract_token(req);

        match token {
            Some(token) => match jwt_service.verify_access_token(&token) {
                Ok(claims) => {
                    // Check subscription status
                    let has_access = claims.subscription_status == "active"
                        || claims.subscription_status == "grace_period";

                    if !has_access {
                        return ready(Err(AppError::Forbidden));
                    }

                    req.extensions_mut().insert(AuthenticatedClaims(claims.clone()));
                    ready(Ok(SubscribedUser(claims)))
                }
                Err(e) => ready(Err(e)),
            },
            None => ready(Err(AppError::Unauthorized)),
        }
    }
}

/// Extract JWT token from request
/// Checks cookie first (access_token), then Authorization header
fn extract_token(req: &HttpRequest) -> Option<String> {
    // Try cookie first
    if let Some(cookie) = req.cookie("access_token") {
        return Some(cookie.value().to_string());
    }

    // Try Authorization header
    if let Some(auth_header) = req.headers().get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str[7..].to_string());
            }
        }
    }

    None
}

/// Cookie configuration for auth tokens
pub struct AuthCookies;

impl AuthCookies {
    /// Create access token cookie
    pub fn access_token(token: &str, secure: bool, cookie_domain: Option<&str>) -> Cookie<'static> {
        let mut builder = Cookie::build("access_token", token.to_owned())
            .path("/")
            .http_only(true)
            .secure(secure)
            .same_site(SameSite::Lax)
            .max_age(actix_web::cookie::time::Duration::minutes(15));

        if let Some(domain) = cookie_domain {
            builder = builder.domain(domain.to_owned());
        }

        builder.finish()
    }

    /// Create refresh token cookie
    pub fn refresh_token(token: &str, secure: bool, remember: bool, cookie_domain: Option<&str>) -> Cookie<'static> {
        let max_age = if remember {
            actix_web::cookie::time::Duration::days(30)
        } else {
            actix_web::cookie::time::Duration::days(1)
        };

        let mut builder = Cookie::build("refresh_token", token.to_owned())
            .path("/")
            .http_only(true)
            .secure(secure)
            .same_site(SameSite::Lax)
            .max_age(max_age);

        if let Some(domain) = cookie_domain {
            builder = builder.domain(domain.to_owned());
        }

        builder.finish()
    }

    /// Create cookies to clear auth tokens
    pub fn clear(secure: bool, cookie_domain: Option<&str>) -> Vec<Cookie<'static>> {
        let mut access_builder = Cookie::build("access_token", "")
            .path("/")
            .http_only(true)
            .secure(secure)
            .same_site(SameSite::Lax)
            .max_age(actix_web::cookie::time::Duration::seconds(0));

        let mut refresh_builder = Cookie::build("refresh_token", "")
            .path("/")
            .http_only(true)
            .secure(secure)
            .same_site(SameSite::Lax)
            .max_age(actix_web::cookie::time::Duration::seconds(0));

        if let Some(domain) = cookie_domain {
            access_builder = access_builder.domain(domain.to_owned());
            refresh_builder = refresh_builder.domain(domain.to_owned());
        }

        vec![access_builder.finish(), refresh_builder.finish()]
    }
}

/// Extract client IP address from request
pub fn extract_client_ip(req: &HttpRequest) -> Option<std::net::IpAddr> {
    // Try X-Forwarded-For header first (for proxied requests)
    if let Some(forwarded) = req.headers().get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse() {
                    return Some(ip);
                }
            }
        }
    }

    // Try X-Real-IP header
    if let Some(real_ip) = req.headers().get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse() {
                return Some(ip);
            }
        }
    }

    // Fall back to connection info
    req.connection_info()
        .realip_remote_addr()
        .and_then(|addr| addr.parse().ok())
}

/// Extract device info from User-Agent header
pub fn extract_device_info(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get(header::USER_AGENT)
        .and_then(|ua| ua.to_str().ok())
        .map(|s| {
            // Truncate to reasonable length
            if s.len() > 256 {
                s[..256].to_string()
            } else {
                s.to_string()
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_cookies_clear() {
        let cookies = AuthCookies::clear(false, None);
        assert_eq!(cookies.len(), 2);
        assert!(cookies.iter().any(|c| c.name() == "access_token"));
        assert!(cookies.iter().any(|c| c.name() == "refresh_token"));
    }

    #[test]
    fn test_auth_cookies_clear_with_domain() {
        let cookies = AuthCookies::clear(true, Some(".a8n.tools"));
        assert_eq!(cookies.len(), 2);
        for cookie in cookies {
            assert_eq!(cookie.domain(), Some(".a8n.tools"));
        }
    }
}
