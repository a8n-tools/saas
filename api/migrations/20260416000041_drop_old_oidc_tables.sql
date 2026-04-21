-- Idempotent cleanup for environments where an early OIDC schema was applied
-- before the final migrations (20260417000040-42). No-op on clean installs.
DROP TABLE IF EXISTS lifecycle_event_delivery CASCADE;
DROP TABLE IF EXISTS lifecycle_event_outbox CASCADE;
DROP TABLE IF EXISTS access_token_blocklist CASCADE;
DROP TABLE IF EXISTS refresh_tokens_v2 CASCADE;
DROP TABLE IF EXISTS refresh_token_families CASCADE;
DROP TABLE IF EXISTS user_application_access CASCADE;
DROP TABLE IF EXISTS oauth_authorization_codes CASCADE;
DROP TABLE IF EXISTS op_sessions CASCADE;
DROP TABLE IF EXISTS oauth_clients CASCADE;
