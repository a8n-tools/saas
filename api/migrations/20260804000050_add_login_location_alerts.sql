-- DEV-525: new-login-location alerts, mirroring bunyip's BUNYIP-366.
--
-- A password that has leaked is usually used from somewhere the owner has never
-- signed in from. Recording the country of the last geolocatable login lets the
-- next one be compared against it, and the owner told when it changes - which is
-- often the only signal a user gets that their credentials are in someone else's
-- hands.
--
-- Nullable, and stays NULL until the first login the deployment can attribute to
-- a country. That is deliberate: the first attributable login is recorded
-- silently, so shipping the feature does not mail every existing user at once.
-- An alert nobody can act on trains people to ignore the ones that matter.
ALTER TABLE users
    ADD COLUMN last_login_country TEXT
        CHECK (last_login_country IS NULL OR length(last_login_country) <= 64),
    -- Per-user opt-out, on by default: a security alert the user never asked
    -- for is still worth sending, but not worth forcing on someone who travels
    -- constantly and has told us to stop.
    ADD COLUMN login_location_alerts BOOLEAN NOT NULL DEFAULT TRUE;
