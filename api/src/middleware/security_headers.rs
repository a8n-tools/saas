//! Security headers middleware
//!
//! This middleware adds security headers to all HTTP responses.

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{HeaderName, HeaderValue},
    Error,
};
use std::future::{ready, Future, Ready};
use std::pin::Pin;

/// Security headers middleware
///
/// Adds security headers to all responses including:
/// - X-Frame-Options: DENY
/// - X-Content-Type-Options: nosniff
/// - X-XSS-Protection: 1; mode=block
/// - Referrer-Policy: strict-origin-when-cross-origin
/// - Strict-Transport-Security (HSTS)
/// - Content-Security-Policy
/// - Permissions-Policy
pub struct SecurityHeaders;

impl<S, B> Transform<S, ServiceRequest> for SecurityHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = SecurityHeadersMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityHeadersMiddleware { service }))
    }
}

pub struct SecurityHeadersMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;
            add_security_headers(res.headers_mut());
            Ok(res)
        })
    }
}

/// Add security headers to response
fn add_security_headers(headers: &mut actix_web::http::header::HeaderMap) {
    // Prevent clickjacking - deny all framing
    headers.insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );

    // Prevent MIME type sniffing
    headers.insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );

    // XSS protection (legacy but still useful for older browsers)
    headers.insert(
        HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block"),
    );

    // Referrer policy - only send origin for cross-origin requests
    headers.insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // HSTS - enforce HTTPS with preload
    // max-age=31536000 = 1 year
    headers.insert(
        HeaderName::from_static("strict-transport-security"),
        HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
    );

    // Content Security Policy
    // - Allow self for default
    // - Allow Stripe scripts and frames
    // - Allow inline styles (needed for React)
    // - Allow data: URLs for images
    headers.insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static(concat!(
            "default-src 'self'; ",
            "script-src 'self' 'unsafe-inline' https://js.stripe.com; ",
            "style-src 'self' 'unsafe-inline'; ",
            "img-src 'self' data: https:; ",
            "font-src 'self' data:; ",
            "frame-src https://js.stripe.com https://hooks.stripe.com; ",
            "connect-src 'self' https://api.stripe.com; ",
            "object-src 'none'; ",
            "base-uri 'self'; ",
            "form-action 'self'; ",
            "frame-ancestors 'none';"
        )),
    );

    // Permissions Policy - restrict browser features
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static(
            "accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(self), usb=()"
        ),
    );

    // Prevent DNS prefetching
    headers.insert(
        HeaderName::from_static("x-dns-prefetch-control"),
        HeaderValue::from_static("off"),
    );

    // Cross-Origin policies for additional isolation
    headers.insert(
        HeaderName::from_static("cross-origin-opener-policy"),
        HeaderValue::from_static("same-origin"),
    );

    headers.insert(
        HeaderName::from_static("cross-origin-resource-policy"),
        HeaderValue::from_static("same-origin"),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::header::HeaderMap;

    #[test]
    fn test_security_headers_added() {
        let mut headers = HeaderMap::new();
        add_security_headers(&mut headers);

        assert!(headers.contains_key("x-frame-options"));
        assert!(headers.contains_key("x-content-type-options"));
        assert!(headers.contains_key("strict-transport-security"));
        assert!(headers.contains_key("content-security-policy"));
        assert!(headers.contains_key("permissions-policy"));
    }

    #[test]
    fn test_x_frame_options_deny() {
        let mut headers = HeaderMap::new();
        add_security_headers(&mut headers);

        let value = headers.get("x-frame-options").unwrap();
        assert_eq!(value, "DENY");
    }
}
