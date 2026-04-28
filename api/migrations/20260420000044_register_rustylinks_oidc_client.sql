-- Register Rusty Links as a confidential OIDC client.
-- The client_secret_hash must be updated with a real Argon2id hash before deploying.
-- Generate the plaintext secret with: openssl rand -hex 32
-- Then hash it with: just oauth-client-secret-hash <secret>
-- The plaintext secret goes into OIDC_CLIENT_SECRET in the Rusty Links environment.

INSERT INTO oauth_clients (
    client_id, client_type, name,
    redirect_uris, post_logout_redirect_uris, backchannel_logout_uri, lifecycle_event_uri,
    allowed_scopes, allowed_grant_types,
    token_endpoint_auth_method, require_pkce,
    client_secret_hash,
    audience
) VALUES (
    'a8000000-0000-0000-0000-000000000005',
    'confidential',
    'rustylinks-web-bff',
    ARRAY[
        'https://links.a8n.run/oauth2/callback',
        'http://localhost:4002/oauth2/callback'
    ],
    ARRAY[
        'https://links.a8n.run/',
        'http://localhost:4002/'
    ],
    'https://links.a8n.run/oauth2/backchannel-logout',
    'https://links.a8n.run/oauth2/lifecycle-event',
    ARRAY['openid', 'email', 'offline_access'],
    ARRAY['authorization_code', 'refresh_token'],
    'client_secret_basic', TRUE,
    '$argon2id$v=19$m=65536,t=3,p=4$Y1zaV3Ekui9d5n2WJz+nDA$vO8+IA48tNpT80m7F+D6RauAY+fWRy/dH3XX6Px85uM',
    'https://links.a8n.run/api'
);
