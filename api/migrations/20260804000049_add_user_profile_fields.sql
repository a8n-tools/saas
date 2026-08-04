-- DEV-525: optional profile fields, mirroring bunyip's BUNYIP-139.
--
-- a8n is an OIDC provider with registered relying parties, but it has only ever
-- been able to tell them an email address. These three columns back the standard
-- `profile` and `phone` scope claims (`given_name`, `family_name`, `name`,
-- `phone_number`), so an RP can render a signed-in user as a person rather than
-- as an address.
--
-- All nullable: legacy rows have no source for them, and none is required to
-- hold an account. The 64-character ceiling matches bunyip column for column so
-- the two schemas converge rather than drift, and it is enforced in the database
-- as well as the handler because the handler is not the only writer.
ALTER TABLE users
    ADD COLUMN first_name TEXT CHECK (first_name IS NULL OR length(first_name) <= 64),
    ADD COLUMN last_name  TEXT CHECK (last_name  IS NULL OR length(last_name)  <= 64),
    ADD COLUMN phone      TEXT CHECK (phone      IS NULL OR length(phone)      <= 64);
