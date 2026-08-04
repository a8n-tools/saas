-- DEV-525: the "first setup account" flag, mirroring bunyip's BUNYIP-413.
--
-- Some admin actions are not reversible by the person they are used against:
-- impersonation mints a session as another user, a role change can strip the
-- last admin, a password reset locks the owner out, and a lifetime grant is a
-- permanent billing decision. Today any account with role = 'admin' can do all
-- four. This column marks the single account allowed to, so inviting a
-- day-to-day admin no longer hands over the whole platform.
ALTER TABLE users ADD COLUMN is_super_admin BOOLEAN NOT NULL DEFAULT FALSE;

-- Backfill on existing deployments: the earliest-created admin is the account
-- that set the platform up. Written as "promote the first admin, but only if
-- nobody holds the flag yet" so it is the same statement the application runs
-- whenever an admin is created (`UserRepository::ensure_super_admin`), and so
-- re-running it can never move the flag off an account an operator chose.
UPDATE users
SET is_super_admin = TRUE
WHERE id = (
    SELECT id FROM users
    WHERE role = 'admin' AND deleted_at IS NULL
    ORDER BY created_at ASC
    LIMIT 1
)
AND NOT EXISTS (
    SELECT 1 FROM users WHERE is_super_admin AND deleted_at IS NULL
);

-- A partial unique index would be wrong here: the flag is deliberately
-- transferable by a future admin UI, and an operator may want two during a
-- handover. The invariant the application maintains is "at least one, once any
-- admin exists", not "exactly one".
CREATE INDEX idx_users_super_admin ON users (is_super_admin) WHERE is_super_admin;
