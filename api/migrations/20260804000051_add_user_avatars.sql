-- DEV-525: uploadable profile avatars, mirroring bunyip's BUNYIP-408.
--
-- Storage is a Postgres BYTEA, never a filesystem or a static mount. The bytes
-- come back only through an authenticated handler that sets an explicit image
-- Content-Type plus `Content-Disposition: inline` and `X-Content-Type-Options:
-- nosniff`. The API serves no static files, so a stored avatar can never be
-- fetched from an origin where it could execute - which is the whole security
-- constraint on letting users upload bytes.
--
-- One row per user: a re-upload replaces it via UPSERT rather than accumulating
-- history nobody asked for.
CREATE TABLE user_avatars (
    user_id     UUID        PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    mime_type   TEXT        NOT NULL,
    -- 2 MiB, matching `ImagePolicy::avatar().max_bytes` in dunite-image-upload.
    -- Duplicated here on purpose: the application check can be bypassed by any
    -- future writer, and this one cannot.
    size_bytes  INTEGER     NOT NULL CHECK (size_bytes > 0 AND size_bytes <= 2097152),
    data        BYTEA       NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Keeps the stored payload in lock-step with the recorded size, which also
    -- caps the BYTEA itself: a row claiming 1 KB while carrying 100 MB is not
    -- representable.
    CONSTRAINT user_avatars_data_size CHECK (octet_length(data) = size_bytes)
);

-- Denormalised "an avatar exists, and this is its version" marker on the users
-- row. NULL means no avatar, so the client renders its initials fallback.
--
-- It lives on `users` rather than being read from `user_avatars` so the hot
-- `SELECT * FROM users` path can tell whether to show an avatar - and get a
-- cache-busting version for the <img> URL - without ever transferring the
-- BYTEA. Written in the same transaction as the avatar row so the two cannot
-- drift into "the marker says yes, the table says no".
ALTER TABLE users
    ADD COLUMN avatar_updated_at TIMESTAMPTZ;
