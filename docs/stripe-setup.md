# Stripe Card Authorization — Setup Guide

This feature requires new users to verify a credit card during signup ($0 authorization, no charge). It uses Stripe SetupIntent to collect and validate card details before account creation.

---

## Keys You Need

From the [Stripe Dashboard](https://dashboard.stripe.com/apikeys) (use **Test mode** for development, **Live mode** for production):

| Key | Prefix | Used by |
|-----|--------|---------|
| Secret key | `sk_test_...` / `sk_live_...` | API backend |
| Publishable key | `pk_test_...` / `pk_live_...` | Browser frontend |

---

## Where Each Key Goes

### Secret Key → Admin Panel (authoritative)

The secret key is stored in the database and loaded at runtime. The DB value **overrides** whatever is in the env file, so always set it via the Admin panel.

1. Log in as admin → **Admin** → **Settings** → **Stripe**
2. Paste the secret key → Save

> **Why the Admin panel and not `.env`?**
> The API hot-reloads Stripe config from the database. If you set the key only in `.env` but there is a row in the `stripe_config` table with a different value, the DB value wins. Setting it via the Admin panel writes to the DB, which is always authoritative.

### Publishable Key → `.env` + redeploy

The publishable key is baked into the frontend at build time via Vite. It must be set as an env var before the container starts.

1. Open the root `.env` file:
   ```
   VITE_STRIPE_PUBLISHABLE_KEY=pk_test_51...
   ```
2. If using `compose.dev.yml`, confirm the frontend service passes it through:
   ```yaml
   environment:
     VITE_STRIPE_PUBLISHABLE_KEY: ${VITE_STRIPE_PUBLISHABLE_KEY:-}
   ```
3. Recreate the container to pick up the new value:
   ```
   docker compose -f compose.dev.yml up -d
   ```
   > `docker compose restart` does **not** re-read `.env` or the compose file. Use `up -d`.

4. Verify it reached the container:
   ```
   docker exec saas-api-long env | grep VITE_STRIPE
   ```

---

## Enabling and Disabling Card Authorization

The card authorization step is controlled entirely by `VITE_STRIPE_PUBLISHABLE_KEY` in the root `.env`. No code changes needed.

### Disable (skip card step on signup)

```
VITE_STRIPE_PUBLISHABLE_KEY=
```

Useful during development so you can register test accounts without a card.

### Enable (card step required on signup)

```
VITE_STRIPE_PUBLISHABLE_KEY=pk_test_51...
```

### Apply the change

After editing `.env`, force-recreate the frontend container:

```
docker compose -f compose.dev.yml up --force-recreate --detach frontend
```

Verify the value reached the container:

```
docker exec saas-frontend-long env | grep VITE_STRIPE
```

> `docker compose restart` does **not** re-read `.env`. Always use `up --force-recreate`.

---

## Webhook Secret (optional for local dev)

The webhook secret (`STRIPE_WEBHOOK_SECRET`) is only required if you are testing webhook events (e.g. subscription lifecycle). For the card authorization signup flow, it is not needed.

To test webhooks locally:
```
stripe listen --forward-to localhost:18080/v1/webhooks/stripe
```
Copy the `whsec_...` value it prints and set it in `.env`.

---

## Verifying the Feature Works

1. Go to the registration page
2. Fill in email + password → click **Continue**
3. A Stripe card form appears (Step 2)
4. Use a Stripe test card:

   | Card number | Result |
   |-------------|--------|
   | `4242 4242 4242 4242` | Success |
   | `4000 0000 0000 9995` | Card declined |
   | `4000 0027 6000 3184` | Requires 3D Secure |

   Expiry: any future date. CVC: any 3 digits.

5. Click **Create Account** — registration completes and you are logged in.

---

## Clearing Rate Limits During Development

The setup-intent endpoint shares the registration rate-limit budget. If you hit 429 errors while testing repeatedly, clear the table:

```
docker exec saas-postgres-long psql --username=a8n --dbname=a8n_platform --command="DELETE FROM rate_limits;"
```

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| 500 on `/v1/billing/setup-intent` with "Invalid API Key" | DB `stripe_config` row has a placeholder key | Set real key via Admin → Stripe settings |
| Stripe Elements not rendering / blank card form | `VITE_STRIPE_PUBLISHABLE_KEY` is empty | Add to `.env`, recreate containers with `up -d` |
| 404 for hashed JS files after container restart | Stale browser cache | Hard refresh (`Ctrl+Shift+R`) or open incognito |
| 429 Too Many Requests | Rate limit hit during repeated testing | Clear rate_limits table (see above) |

---

## Config Layering Summary

```
Admin Panel (stripe_config DB table)   ← always wins
        ↑ overrides
root .env  →  compose environment:  →  container env var
```

Set the **secret key** in the Admin panel. Set the **publishable key** in `.env`.
