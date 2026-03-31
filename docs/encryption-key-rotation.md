# Encryption Key Management

The platform encrypts secrets at rest using AES-256-GCM with per-purpose keys.
Two encryption keys are currently in use:

| Key | Env var | Protects |
|-----|---------|----------|
| TOTP | `TOTP_ENCRYPTION_KEY` | TOTP authenticator secrets (`user_totp` table) |
| Stripe | `STRIPE_ENCRYPTION_KEY` | Stripe secret key and webhook secret (`stripe_config` table) |

Each key is a **32-byte (256-bit) value encoded as a 64-character hex string**.

---

## Generating a Key

Use the same command everywhere a new encryption key is needed:

```bash
openssl rand -hex 32
```

This produces a cryptographically random 64-character hex string (32 bytes), suitable for AES-256-GCM.

Example output:

```
a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90
```

---

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `TOTP_ENCRYPTION_KEY` | Production | Current TOTP encryption key |
| `TOTP_ENCRYPTION_KEY_PREV` | During rotation | Previous TOTP key (fallback decryption) |
| `TOTP_KEY_VERSION` | During rotation | Integer version of the current TOTP key (default: `1`) |
| `STRIPE_ENCRYPTION_KEY` | Production | Current Stripe encryption key |
| `STRIPE_ENCRYPTION_KEY_PREV` | During rotation | Previous Stripe key (fallback decryption) |
| `STRIPE_KEY_VERSION` | During rotation | Integer version of the current Stripe key (default: `1`) |

In development mode, if a key is not set the API falls back to a deterministic dev key. In production, missing keys cause a startup panic.

---

## Rotation Workflow

Key rotation is zero-downtime. The process is identical for TOTP and Stripe keys — just swap the variable names and key ID.

### Step 1 — Generate a new key

```bash
openssl rand -hex 32
```

Save the output. This is your new key.

### Step 2 — Update environment variables

Move the current key to `*_PREV`, set the new key as current, and bump the version:

```bash
# Example: rotating the TOTP key from version 1 → 2
TOTP_ENCRYPTION_KEY=<new key from step 1>
TOTP_ENCRYPTION_KEY_PREV=<old key that was in TOTP_ENCRYPTION_KEY>
TOTP_KEY_VERSION=2
```

For Stripe, the equivalent is:

```bash
STRIPE_ENCRYPTION_KEY=<new key>
STRIPE_ENCRYPTION_KEY_PREV=<old key>
STRIPE_KEY_VERSION=2
```

### Step 3 — Deploy

After deploying with the new env vars:

- **New writes** encrypt with the new key.
- **Reads** try the current key first; on version mismatch or failure, fall back to the previous key. No downtime.

### Step 4 — Re-encrypt existing records

Trigger re-encryption so all rows use the new key:

```bash
curl -b cookies -X POST http://localhost:18080/v1/admin/key-rotation/totp/reencrypt
curl -b cookies -X POST http://localhost:18080/v1/admin/key-rotation/stripe/reencrypt
```

Response:

```json
{
  "success": true,
  "data": {
    "key_id": "totp",
    "reencrypted": 3,
    "total": 15,
    "key_version": 2
  }
}
```

### Step 5 — Verify rotation is complete

```bash
curl -b cookies http://localhost:18080/v1/admin/key-rotation/totp/status
curl -b cookies http://localhost:18080/v1/admin/key-rotation/stripe/status
```

Confirm `rotation_complete` is `true` and `old_version_count` is `0`:

```json
{
  "success": true,
  "data": {
    "key_id": "totp",
    "current_version": 2,
    "total_records": 15,
    "current_version_count": 15,
    "old_version_count": 0,
    "rotation_complete": true
  }
}
```

### Step 6 — Clean up

Remove the `*_PREV` variable and redeploy:

```bash
# Remove these from your env / secrets manager:
# TOTP_ENCRYPTION_KEY_PREV
# STRIPE_ENCRYPTION_KEY_PREV
```

The previous key is no longer needed once all records are re-encrypted.

---

## Health Checks

Use these endpoints to verify keys can still decrypt stored data:

```bash
# All keys at once
curl -b cookies http://localhost:18080/v1/admin/key-health

# Individual key
curl -b cookies http://localhost:18080/v1/admin/key-health/totp
curl -b cookies http://localhost:18080/v1/admin/key-health/stripe
```

Per-key status values:

| Status | Meaning |
|--------|---------|
| `healthy` | Decryption succeeded |
| `unhealthy` | Data exists but decryption failed (response includes error message) |
| `no_data` | No encrypted records in DB to verify |

The aggregated endpoint returns `"healthy"` unless any individual check is `"unhealthy"`, in which case it returns `"degraded"`.

Health responses also include rotation info:

```json
{
  "status": "healthy",
  "has_data": true,
  "key_version": 2,
  "needs_reencrypt": false
}
```

---

## Quick Reference

Rotate a key end-to-end (replace `totp` / `TOTP` with `stripe` / `STRIPE` as needed):

```bash
# 1. Generate
NEW_KEY=$(openssl rand -hex 32)
echo "New key: $NEW_KEY"

# 2. Set env vars (in your .env or secrets manager)
#    TOTP_ENCRYPTION_KEY=<$NEW_KEY>
#    TOTP_ENCRYPTION_KEY_PREV=<old key>
#    TOTP_KEY_VERSION=<current version + 1>

# 3. Deploy

# 4. Re-encrypt
curl -b cookies -X POST http://localhost:18080/v1/admin/key-rotation/totp/reencrypt

# 5. Verify
curl -b cookies http://localhost:18080/v1/admin/key-rotation/totp/status

# 6. Remove TOTP_ENCRYPTION_KEY_PREV, redeploy
```
