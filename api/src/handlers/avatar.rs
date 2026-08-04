//! DEV-525: profile avatar upload, removal and serving.
//!
//! Every check runs against file CONTENT, never the browser's declared MIME or
//! filename, and lives in `dunite-image-upload` so a8n-tools and bunyip enforce
//! the same rules the same way. The bytes go into a Postgres BYTEA and come back
//! only through [`get_avatar`], with an explicit image `Content-Type`,
//! `Content-Disposition: inline` and `nosniff`. This API serves no static files,
//! so a stored avatar can never be fetched from an origin where it could
//! execute - which is the security constraint on accepting user bytes at all.

use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse};
use dunite_image_upload::{validate_image, ImagePolicy, ImageValidationError};
use futures_util::TryStreamExt;
use sqlx::PgPool;

use crate::errors::AppError;
use crate::middleware::AuthenticatedUser;
use crate::models::UserResponse;
use crate::repositories::UserRepository;
use crate::responses::{get_request_id, success};

/// Convert a shared-crate validation failure into this app's error type.
///
/// The message is the crate's own, which describes the rule and never the
/// bytes; the field name is a8n's, so the client can attach the message to the
/// right input.
fn avatar_err(e: ImageValidationError) -> AppError {
    tracing::warn!(reason = ?e, "Avatar upload rejected");
    AppError::validation("avatar", e.to_string())
}

/// POST /v1/users/me/avatar (multipart, one file part).
///
/// Streams the file part, enforcing the byte cap as it goes so an oversized body
/// is refused mid-stream rather than after being buffered whole, then validates
/// the finished bytes by content and stores them.
pub async fn upload_avatar(
    req: HttpRequest,
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
    mut payload: Multipart,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let policy = ImagePolicy::avatar();

    let mut avatar_bytes: Option<Vec<u8>> = None;
    while let Some(mut field) = payload
        .try_next()
        .await
        .map_err(|_| AppError::validation("avatar", "Invalid multipart data"))?
    {
        let is_file = field
            .content_disposition()
            .and_then(|cd| cd.get_filename())
            .is_some();
        if !is_file {
            // Drain any stray text part rather than aborting: a form that also
            // posts a CSRF token or a caption is not an error.
            while field
                .try_next()
                .await
                .map_err(|_| AppError::validation("avatar", "Failed to read field"))?
                .is_some()
            {}
            continue;
        }

        let mut bytes = Vec::new();
        while let Some(chunk) = field
            .try_next()
            .await
            .map_err(|_| AppError::validation("avatar", "Failed to read field"))?
        {
            bytes.extend_from_slice(&chunk);
            // Stop reading the moment the cap is passed. Buffering the whole
            // body first would let a large upload cost memory before it is
            // refused.
            if bytes.len() > policy.max_bytes {
                return Err(avatar_err(ImageValidationError::TooLarge {
                    max_bytes: policy.max_bytes,
                }));
            }
        }
        avatar_bytes = Some(bytes);
        break;
    }

    let bytes = avatar_bytes.ok_or_else(|| avatar_err(ImageValidationError::Empty))?;
    let mime = validate_image(&bytes, &policy).map_err(avatar_err)?;

    let updated = UserRepository::set_avatar(&pool, user.0.sub, &mime, &bytes).await?;
    tracing::info!(user_id = %user.0.sub, mime = %mime, size = bytes.len(), "Avatar updated");

    Ok(success(UserResponse::from(updated), request_id))
}

/// DELETE /v1/users/me/avatar. Idempotent; returns the refreshed user.
pub async fn delete_avatar(
    req: HttpRequest,
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let request_id = get_request_id(&req);
    let updated = UserRepository::clear_avatar(&pool, user.0.sub).await?;
    tracing::info!(user_id = %user.0.sub, "Avatar removed");
    Ok(success(UserResponse::from(updated), request_id))
}

/// GET /v1/users/me/avatar.
///
/// Authenticated and self-only: a user fetches their own avatar, so there is no
/// cross-user enumeration surface. 404 when none is set, which is the signal for
/// the client to render its initials fallback.
pub async fn get_avatar(
    user: AuthenticatedUser,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, AppError> {
    let (mime, data) = UserRepository::get_avatar(&pool, user.0.sub)
        .await?
        .ok_or_else(|| AppError::not_found("Avatar"))?;

    Ok(HttpResponse::Ok()
        .content_type(mime)
        // The stored type was sniffed from content, but nosniff costs nothing
        // and closes the gap if a future writer ever stores a declared type.
        .insert_header(("X-Content-Type-Options", "nosniff"))
        .insert_header(("Content-Disposition", "inline"))
        // Private: a per-user resource behind auth. The `?v=` version the client
        // appends from `avatar_updated_at` busts the cache on change, so a
        // short max-age is safe.
        .insert_header(("Cache-Control", "private, max-age=300"))
        .body(data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_failures_are_field_scoped_and_carry_the_rule() {
        // The client attaches the message to the avatar input, so the field name
        // has to be a8n's while the wording stays the shared crate's.
        let err = avatar_err(ImageValidationError::UnknownType);
        match err {
            AppError::ValidationError { field, message } => {
                assert_eq!(field, "avatar");
                assert_eq!(message, "Could not verify the file is an image");
            }
            other => panic!("expected a field validation error, got {other:?}"),
        }
    }

    #[test]
    fn the_size_cap_matches_the_storage_constraint() {
        // `user_avatars.size_bytes` has a CHECK at 2 MiB. If the policy ever
        // exceeded it, a valid upload would pass validation and then fail on
        // insert with a database error rather than a useful message.
        assert_eq!(ImagePolicy::avatar().max_bytes, 2 * 1024 * 1024);
    }
}
