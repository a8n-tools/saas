//! OCI-flavored error type. Uses the OCI error envelope on the wire.

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use thiserror::Error;

use crate::models::oci::OciErrorEnvelope;

#[derive(Debug, Error)]
pub enum OciError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("denied")]
    Denied,
    #[error("name unknown")]
    NameUnknown,
    #[error("manifest unknown")]
    ManifestUnknown,
    #[error("blob unknown")]
    BlobUnknown,
    #[error("too many requests")]
    TooManyRequests { retry_after_secs: Option<u64> },
    #[error("upstream unavailable")]
    Upstream,
    #[error("unsupported")]
    Unsupported,
    #[error("internal")]
    Internal,
}

impl OciError {
    fn code(&self) -> &'static str {
        match self {
            Self::Unauthorized => "UNAUTHORIZED",
            Self::Denied => "DENIED",
            Self::NameUnknown => "NAME_UNKNOWN",
            Self::ManifestUnknown => "MANIFEST_UNKNOWN",
            Self::BlobUnknown => "BLOB_UNKNOWN",
            Self::TooManyRequests { .. } => "TOOMANYREQUESTS",
            Self::Upstream => "UNKNOWN",
            Self::Unsupported => "UNSUPPORTED",
            Self::Internal => "UNKNOWN",
        }
    }

    fn message(&self) -> &'static str {
        match self {
            Self::Unauthorized => "authentication required",
            Self::Denied => "access denied",
            Self::NameUnknown => "repository name not known",
            Self::ManifestUnknown => "manifest not known",
            Self::BlobUnknown => "blob not known",
            Self::TooManyRequests { .. } => "too many requests",
            Self::Upstream => "upstream error",
            Self::Unsupported => "unsupported operation",
            Self::Internal => "internal error",
        }
    }
}

impl ResponseError for OciError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Denied => StatusCode::FORBIDDEN,
            Self::NameUnknown | Self::ManifestUnknown | Self::BlobUnknown => StatusCode::NOT_FOUND,
            Self::TooManyRequests { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::Upstream => StatusCode::BAD_GATEWAY,
            Self::Unsupported => StatusCode::METHOD_NOT_ALLOWED,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let envelope = OciErrorEnvelope::single(self.code(), self.message());
        let mut builder = HttpResponse::build(self.status_code());
        builder.content_type("application/json");
        if let Self::TooManyRequests {
            retry_after_secs: Some(secs),
        } = self
        {
            builder.insert_header(("Retry-After", secs.to_string()));
        }
        builder.json(envelope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_codes_match_spec() {
        assert_eq!(
            OciError::Unauthorized.status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(OciError::Denied.status_code(), StatusCode::FORBIDDEN);
        assert_eq!(
            OciError::ManifestUnknown.status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(OciError::BlobUnknown.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(
            OciError::TooManyRequests {
                retry_after_secs: None
            }
            .status_code(),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(OciError::Upstream.status_code(), StatusCode::BAD_GATEWAY);
        assert_eq!(
            OciError::Unsupported.status_code(),
            StatusCode::METHOD_NOT_ALLOWED
        );
    }

    #[test]
    fn error_body_uses_oci_envelope() {
        let resp = OciError::ManifestUnknown.error_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            resp.headers().get("content-type").unwrap(),
            "application/json"
        );
    }

    #[test]
    fn retry_after_set_on_daily_cap() {
        let resp = OciError::TooManyRequests {
            retry_after_secs: Some(3600),
        }
        .error_response();
        let val = resp.headers().get("Retry-After").unwrap();
        assert_eq!(val, "3600");
    }
}
