-- =========================
-- Extensions
-- =========================
-- Required extensions (enable as needed)
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
-- Add other extensions if required by your project
-- Note: If any extension creation fails due to permissions, run as a superuser.

-- =========================
-- Schemas
-- =========================
CREATE SCHEMA IF NOT EXISTS auth;
CREATE SCHEMA IF NOT EXISTS storage;
CREATE SCHEMA IF NOT EXISTS realtime;
CREATE SCHEMA IF NOT EXISTS app;
CREATE SCHEMA IF NOT EXISTS vault;
CREATE SCHEMA IF NOT EXISTS graphql;
CREATE SCHEMA IF NOT EXISTS graphql_public;
CREATE SCHEMA IF NOT EXISTS extensions;
CREATE SCHEMA IF NOT EXISTS pgbouncer;
CREATE SCHEMA IF NOT EXISTS public;

-- =========================
-- Types / Enums (user-defined)
-- =========================
-- auth.aal_level
CREATE TYPE IF NOT EXISTS auth.aal_level AS ENUM ('aal1','aal2','aal3');

-- auth.factor_type
CREATE TYPE IF NOT EXISTS auth.factor_type AS ENUM ('totp','webauthn','phone');

-- auth.factor_status
CREATE TYPE IF NOT EXISTS auth.factor_status AS ENUM ('unverified','verified');

-- auth.one_time_token_type
CREATE TYPE IF NOT EXISTS auth.one_time_token_type AS ENUM (
  'confirmation_token','reauthentication_token','recovery_token',
  'email_change_token_new','email_change_token_current','phone_change_token'
);

-- auth.oauth_registration_type
CREATE TYPE IF NOT EXISTS auth.oauth_registration_type AS ENUM ('dynamic','manual');

-- auth.oauth_client_type
CREATE TYPE IF NOT EXISTS auth.oauth_client_type AS ENUM ('public','confidential');

-- auth.oauth_response_type
CREATE TYPE IF NOT EXISTS auth.oauth_response_type AS ENUM ('code');

-- auth.oauth_authorization_status
CREATE TYPE IF NOT EXISTS auth.oauth_authorization_status AS ENUM ('pending','approved','denied','expired');

-- storage.buckettype
CREATE TYPE IF NOT EXISTS storage.buckettype AS ENUM ('STANDARD','ANALYTICS','VECTOR');

-- realtime.user_defined_filter (composite type placeholder)
-- If realtime.user_defined_filter is a custom composite type, recreate if known.
-- Placeholder: skip unless you need exact type.

-- =========================
-- Tables: auth schema
-- =========================

-- auth.users
CREATE TABLE IF NOT EXISTS auth.users (
  instance_id uuid,
  id uuid PRIMARY KEY,
  aud character varying,
  role character varying,
  email character varying,
  encrypted_password character varying,
  invited_at timestamptz,
  confirmation_token character varying,
  confirmation_sent_at timestamptz,
  recovery_token character varying,
  recovery_sent_at timestamptz,
  email_change character varying,
  email_change_sent_at timestamptz,
  last_sign_in_at timestamptz,
  raw_app_meta_data jsonb,
  raw_user_meta_data jsonb,
  is_super_admin boolean,
  created_at timestamptz,
  updated_at timestamptz,
  email_change_token_new character varying,
  phone_confirmed_at timestamptz,
  phone_change_token character varying DEFAULT ''::character varying,
  phone_change_sent_at timestamptz,
  email_confirmed_at timestamptz,
  confirmed_at timestamptz GENERATED ALWAYS AS (LEAST(email_confirmed_at, phone_confirmed_at)) STORED,
  phone text DEFAULT NULL::character varying,
  phone_change text DEFAULT ''::character varying,
  email_change_token_current character varying DEFAULT ''::character varying,
  email_change_confirm_status smallint DEFAULT 0 CHECK (email_change_confirm_status >= 0 AND email_change_confirm_status <= 2),
  banned_until timestamptz,
  reauthentication_token character varying DEFAULT ''::character varying,
  reauthentication_sent_at timestamptz,
  is_sso_user boolean DEFAULT false,
  deleted_at timestamptz,
  is_anonymous boolean DEFAULT false
);
-- Note: UNIQUE constraint on phone (detected). Recreate:
ALTER TABLE auth.users ADD CONSTRAINT IF NOT EXISTS users_phone_key UNIQUE (phone);

-- Enable RLS if desired
ALTER TABLE auth.users ENABLE ROW LEVEL SECURITY;

-- auth.refresh_tokens
CREATE SEQUENCE IF NOT EXISTS auth.refresh_tokens_id_seq;
CREATE TABLE IF NOT EXISTS auth.refresh_tokens (
  instance_id uuid,
  token character varying UNIQUE,
  user_id character varying,
  revoked boolean,
  created_at timestamptz,
  updated_at timestamptz,
  id bigint PRIMARY KEY DEFAULT nextval('auth.refresh_tokens_id_seq'::regclass),
  parent character varying,
  session_id uuid
);
ALTER TABLE auth.refresh_tokens ENABLE ROW LEVEL SECURITY;

-- auth.instances
CREATE TABLE IF NOT EXISTS auth.instances (
  id uuid PRIMARY KEY,
  uuid uuid,
  raw_base_config text,
  created_at timestamptz,
  updated_at timestamptz
);
ALTER TABLE auth.instances ENABLE ROW LEVEL SECURITY;

-- auth.audit_log_entries
CREATE TABLE IF NOT EXISTS auth.audit_log_entries (
  instance_id uuid,
  id uuid PRIMARY KEY,
  payload json,
  created_at timestamptz,
  ip_address character varying DEFAULT ''::character varying
);
ALTER TABLE auth.audit_log_entries ENABLE ROW LEVEL SECURITY;

-- auth.schema_migrations
CREATE TABLE IF NOT EXISTS auth.schema_migrations (
  version character varying PRIMARY KEY
);
ALTER TABLE auth.schema_migrations ENABLE ROW LEVEL SECURITY;

-- auth.identities
CREATE TABLE IF NOT EXISTS auth.identities (
  user_id uuid,
  identity_data jsonb,
  provider text,
  last_sign_in_at timestamptz,
  created_at timestamptz,
  updated_at timestamptz,
  provider_id text,
  email text GENERATED ALWAYS AS (lower((identity_data ->> 'email'::text))) STORED,
  id uuid PRIMARY KEY DEFAULT gen_random_uuid()
);
ALTER TABLE auth.identities ENABLE ROW LEVEL SECURITY;

-- auth.sessions
CREATE TABLE IF NOT EXISTS auth.sessions (
  id uuid PRIMARY KEY,
  user_id uuid,
  created_at timestamptz,
  updated_at timestamptz,
  factor_id uuid,
  aal auth.aal_level,
  not_after timestamptz,
  refreshed_at timestamp,
  user_agent text,
  ip inet,
  tag text,
  refresh_token_hmac_key text,
  refresh_token_counter bigint,
  oauth_client_id uuid,
  scopes text CHECK (char_length(scopes) <= 4096)
);
ALTER TABLE auth.sessions ENABLE ROW LEVEL SECURITY;

-- auth.mfa_factors
CREATE TABLE IF NOT EXISTS auth.mfa_factors (
  id uuid PRIMARY KEY,
  user_id uuid,
  friendly_name text,
  factor_type auth.factor_type,
  status auth.factor_status,
  created_at timestamptz,
  updated_at timestamptz,
  secret text,
  phone text,
  last_challenged_at timestamptz UNIQUE,
  web_authn_credential jsonb,
  web_authn_aaguid uuid,
  last_webauthn_challenge_data jsonb
);
ALTER TABLE auth.mfa_factors ENABLE ROW LEVEL SECURITY;

-- auth.mfa_challenges
CREATE TABLE IF NOT EXISTS auth.mfa_challenges (
  id uuid PRIMARY KEY,
  factor_id uuid,
  created_at timestamptz,
  verified_at timestamptz,
  ip_address inet,
  otp_code text,
  web_authn_session_data jsonb
);
ALTER TABLE auth.mfa_challenges ENABLE ROW LEVEL SECURITY;

-- auth.mfa_amr_claims
CREATE TABLE IF NOT EXISTS auth.mfa_amr_claims (
  session_id uuid,
  created_at timestamptz,
  updated_at timestamptz,
  authentication_method text,
  id uuid PRIMARY KEY
);
ALTER TABLE auth.mfa_amr_claims ENABLE ROW LEVEL SECURITY;

-- auth.sso_providers
CREATE TABLE IF NOT EXISTS auth.sso_providers (
  id uuid PRIMARY KEY,
  resource_id text CHECK (resource_id = NULL::text OR char_length(resource_id) > 0),
  created_at timestamptz,
  updated_at timestamptz,
  disabled boolean
);
ALTER TABLE auth.sso_providers ENABLE ROW LEVEL SECURITY;

-- auth.sso_domains
CREATE TABLE IF NOT EXISTS auth.sso_domains (
  id uuid PRIMARY KEY,
  sso_provider_id uuid,
  domain text CHECK (char_length(domain) > 0),
  created_at timestamptz,
  updated_at timestamptz
);
ALTER TABLE auth.sso_domains ENABLE ROW LEVEL SECURITY;

-- auth.saml_providers
CREATE TABLE IF NOT EXISTS auth.saml_providers (
  id uuid PRIMARY KEY,
  sso_provider_id uuid,
  entity_id text UNIQUE CHECK (char_length(entity_id) > 0),
  metadata_xml text CHECK (char_length(metadata_xml) > 0),
  metadata_url text CHECK (metadata_url = NULL::text OR char_length(metadata_url) > 0),
  attribute_mapping jsonb,
  created_at timestamptz,
  updated_at timestamptz,
  name_id_format text
);
ALTER TABLE auth.saml_providers ENABLE ROW LEVEL SECURITY;

-- auth.saml_relay_states
CREATE TABLE IF NOT EXISTS auth.saml_relay_states (
  id uuid PRIMARY KEY,
  sso_provider_id uuid,
  request_id text CHECK (char_length(request_id) > 0),
  for_email text,
  redirect_to text,
  created_at timestamptz,
  updated_at timestamptz,
  flow_state_id uuid
);
ALTER TABLE auth.saml_relay_states ENABLE ROW LEVEL SECURITY;

-- auth.flow_state
CREATE TABLE IF NOT EXISTS auth.flow_state (
  id uuid PRIMARY KEY,
  user_id uuid,
  auth_code text,
  code_challenge_method auth.code_challenge_method, -- placeholder: code_challenge_method enum/type assumed present
  code_challenge text,
  provider_type text,
  provider_access_token text,
  provider_refresh_token text,
  created_at timestamptz,
  updated_at timestamptz,
  authentication_method text,
  auth_code_issued_at timestamptz
);
ALTER TABLE auth.flow_state ENABLE ROW LEVEL SECURITY;

-- auth.one_time_tokens
CREATE TABLE IF NOT EXISTS auth.one_time_tokens (
  created_at timestamp DEFAULT now(),
  id uuid PRIMARY KEY,
  user_id uuid,
  token_type auth.one_time_token_type,
  token_hash text CHECK (char_length(token_hash) > 0),
  relates_to text,
  updated_at timestamp DEFAULT now()
);
ALTER TABLE auth.one_time_tokens ENABLE ROW LEVEL SECURITY;

-- auth.oauth_clients
CREATE TABLE IF NOT EXISTS auth.oauth_clients (
  id uuid PRIMARY KEY,
  registration_type auth.oauth_registration_type,
  redirect_uris text,
  grant_types text,
  client_name text CHECK (char_length(client_name) <= 1024),
  client_uri text CHECK (char_length(client_uri) <= 2048),
  logo_uri text CHECK (char_length(logo_uri) <= 2048),
  deleted_at timestamptz,
  created_at timestamptz DEFAULT now(),
  updated_at timestamptz DEFAULT now(),
  client_secret_hash text,
  client_type auth.oauth_client_type DEFAULT 'confidential'::auth.oauth_client_type
);

-- auth.oauth_authorizations
CREATE TABLE IF NOT EXISTS auth.oauth_authorizations (
  id uuid PRIMARY KEY,
  authorization_id text UNIQUE,
  client_id uuid,
  user_id uuid,
  redirect_uri text CHECK (char_length(redirect_uri) <= 2048),
  scope text CHECK (char_length(scope) <= 4096),
  state text CHECK (char_length(state) <= 4096),
  resource text CHECK (char_length(resource) <= 2048),
  code_challenge text CHECK (char_length(code_challenge) <= 128),
  code_challenge_method auth.code_challenge_method, -- placeholder
  authorization_code text UNIQUE CHECK (char_length(authorization_code) <= 255),
  approved_at timestamptz,
  response_type auth.oauth_response_type DEFAULT 'code'::auth.oauth_response_type,
  status auth.oauth_authorization_status DEFAULT 'pending'::auth.oauth_authorization_status,
  created_at timestamptz DEFAULT now(),
  expires_at timestamptz DEFAULT (now() + '00:03:00'::interval),
  nonce text CHECK (char_length(nonce) <= 255)
);

-- auth.oauth_consents
CREATE TABLE IF NOT EXISTS auth.oauth_consents (
  id uuid PRIMARY KEY,
  user_id uuid,
  client_id uuid,
  scopes text CHECK (char_length(scopes) <= 2048),
  revoked_at timestamptz,
  granted_at timestamptz DEFAULT now()
);

-- =========================
-- Tables: storage schema
-- =========================

-- storage.buckets
CREATE TYPE IF NOT EXISTS storage.buckettype; -- ensure type exists (created above)
CREATE TABLE IF NOT EXISTS storage.buckets (
  id text PRIMARY KEY,
  name text,
  owner uuid,
  created_at timestamptz DEFAULT now(),
  updated_at timestamptz DEFAULT now(),
  public boolean DEFAULT false,
  avif_autodetection boolean DEFAULT false,
  allowed_mime_types text[],
  file_size_limit bigint,
  owner_id text,
  type storage.buckettype DEFAULT 'STANDARD'::storage.buckettype
);
ALTER TABLE storage.buckets ENABLE ROW LEVEL SECURITY;

-- storage.objects
CREATE TABLE IF NOT EXISTS storage.objects (
  bucket_id text,
  name text,
  owner uuid,
  created_at timestamptz DEFAULT now(),
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  metadata jsonb,
  updated_at timestamptz DEFAULT now(),
  last_accessed_at timestamptz DEFAULT now(),
  level integer,
  path_tokens text[] GENERATED ALWAYS AS (string_to_array(name, '/'::text)) STORED,
  version text,
  owner_id text,
  user_metadata jsonb
);
ALTER TABLE storage.objects ENABLE ROW LEVEL SECURITY;

-- storage.migrations
CREATE TABLE IF NOT EXISTS storage.migrations (
  id integer PRIMARY KEY,
  name character varying UNIQUE,
  hash character varying,
  executed_at timestamp DEFAULT CURRENT_TIMESTAMP
);
ALTER TABLE storage.migrations ENABLE ROW LEVEL SECURITY;

-- storage.s3_multipart_uploads
CREATE TABLE IF NOT EXISTS storage.s3_multipart_uploads (
  id text PRIMARY KEY,
  upload_signature text,
  bucket_id text,
  key text,
  version text,
  owner_id text,
  created_at timestamptz DEFAULT now(),
  in_progress_size bigint DEFAULT 0,
  user_metadata jsonb
);
ALTER TABLE storage.s3_multipart_uploads ENABLE ROW LEVEL SECURITY;

-- storage.s3_multipart_uploads_parts
CREATE TABLE IF NOT EXISTS storage.s3_multipart_uploads_parts (
  upload_id text,
  part_number integer,
  bucket_id text,
  key text,
  etag text,
  owner_id text,
  version text,
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  created_at timestamptz DEFAULT now(),
  size bigint DEFAULT 0
);
ALTER TABLE storage.s3_multipart_uploads_parts ENABLE ROW LEVEL SECURITY;

-- storage.prefixes
CREATE TABLE IF NOT EXISTS storage.prefixes (
  bucket_id text,
  name text,
  level integer GENERATED ALWAYS AS (storage.get_level(name)) STORED,
  created_at timestamptz DEFAULT now(),
  updated_at timestamptz DEFAULT now(),
  PRIMARY KEY (bucket_id, name, level)
);
ALTER TABLE storage.prefixes ENABLE ROW LEVEL SECURITY;

-- storage.buckets_analytics
CREATE TABLE IF NOT EXISTS storage.buckets_analytics (
  type storage.buckettype DEFAULT 'ANALYTICS'::storage.buckettype,
  format text DEFAULT 'ICEBERG'::text,
  created_at timestamptz DEFAULT now(),
  updated_at timestamptz DEFAULT now(),
  name text,
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  deleted_at timestamptz
);
ALTER TABLE storage.buckets_analytics ENABLE ROW LEVEL SECURITY;

-- storage.buckets_vectors
CREATE TABLE IF NOT EXISTS storage.buckets_vectors (
  id text PRIMARY KEY,
  type storage.buckettype DEFAULT 'VECTOR'::storage.buckettype,
  created_at timestamptz DEFAULT now(),
  updated_at timestamptz DEFAULT now()
);
ALTER TABLE storage.buckets_vectors ENABLE ROW LEVEL SECURITY;

-- storage.vector_indexes
CREATE TABLE IF NOT EXISTS storage.vector_indexes (
  name text,
  bucket_id text,
  data_type text,
  dimension integer,
  distance_metric text,
  metadata_configuration jsonb,
  id text PRIMARY KEY DEFAULT gen_random_uuid(),
  created_at timestamptz DEFAULT now(),
  updated_at timestamptz DEFAULT now()
);
ALTER TABLE storage.vector_indexes ENABLE ROW LEVEL SECURITY;

-- =========================
-- Tables: realtime schema
-- =========================

-- realtime.schema_migrations
CREATE TABLE IF NOT EXISTS realtime.schema_migrations (
  version bigint PRIMARY KEY,
  inserted_at timestamp
);

-- realtime.subscription
CREATE TABLE IF NOT EXISTS realtime.subscription (
  id bigint GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  entity regclass,
  filters realtime.user_defined_filter[] DEFAULT '{}'::realtime.user_defined_filter[],
  subscription_id uuid,
  claims jsonb,
  claims_role regrole GENERATED ALWAYS AS (realtime.to_regrole((claims ->> 'role'::text))) STORED,
  created_at timestamp DEFAULT timezone('utc'::text, now())
);

-- realtime.messages
CREATE TABLE IF NOT EXISTS realtime.messages (
  id uuid DEFAULT gen_random_uuid(),
  private boolean DEFAULT false,
  updated_at timestamp DEFAULT now(),
  topic text,
  extension text,
  payload jsonb,
  event text,
  inserted_at timestamp DEFAULT now(),
  PRIMARY KEY (inserted_at, id)
);
ALTER TABLE realtime.messages ENABLE ROW LEVEL SECURITY;

-- =========================
-- Tables: app schema
-- =========================

-- app.subscriptions
CREATE TABLE IF NOT EXISTS app.subscriptions (
  subscription_id text PRIMARY KEY,
  user_id uuid,
  stripe_customer_id text,
  price_id text,
  plan text CHECK (plan = ANY (ARRAY['pro'::text, 'basic'::text, 'enterprise'::text])),
  status text CHECK (status = ANY (ARRAY['trialing'::text, 'active'::text, 'past_due'::text, 'canceled'::text, 'unpaid'::text, 'incomplete'::text, 'incomplete_expired'::text, 'paused'::text])),
  current_period_end timestamptz,
  created_at timestamptz DEFAULT now(),
  cancel_at_period_end boolean DEFAULT false
);
ALTER TABLE app.subscriptions ENABLE ROW LEVEL SECURITY;

-- app.litellm_keys
CREATE SEQUENCE IF NOT EXISTS app.litellm_keys_id_seq;
CREATE TABLE IF NOT EXISTS app.litellm_keys (
  user_id uuid,
  subscription_id text,
  virtual_key text,
  plan text,
  status text CHECK (status = ANY (ARRAY['active'::text, 'revoked'::text, 'paused'::text])),
  id bigint PRIMARY KEY DEFAULT nextval('app.litellm_keys_id_seq'::regclass),
  created_at timestamptz DEFAULT now(),
  updated_at timestamptz DEFAULT now()
);
ALTER TABLE app.litellm_keys ENABLE ROW LEVEL SECURITY;

-- app.webhook_events
CREATE TABLE IF NOT EXISTS app.webhook_events (
  event_id text PRIMARY KEY,
  event_type text,
  created_at timestamptz DEFAULT now()
);
ALTER TABLE app.webhook_events ENABLE ROW LEVEL SECURITY;

-- =========================
-- Tables: vault schema
-- =========================

-- vault.secrets
CREATE TABLE IF NOT EXISTS vault.secrets (
  name text,
  secret text,
  key_id uuid,
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  description text DEFAULT ''::text,
  nonce bytea DEFAULT vault._crypto_aead_det_noncegen(), -- NOTE: requires vault extension/function
  created_at timestamptz DEFAULT CURRENT_TIMESTAMP,
  updated_at timestamptz DEFAULT CURRENT_TIMESTAMP
);
-- vault.secrets DOES NOT have RLS enabled in source

-- If vault._crypto_aead_det_noncegen() is not present, replace the DEFAULT with gen_random_bytes(...) or remove default.

-- =========================
-- Foreign Key Constraints
-- (Add after tables are created)
-- =========================

-- auth.identities.user_id -> auth.users.id
ALTER TABLE IF EXISTS auth.identities
  ADD CONSTRAINT IF NOT EXISTS identities_user_id_fkey FOREIGN KEY (user_id) REFERENCES auth.users(id);

-- auth.mfa_factors.user_id -> auth.users.id
ALTER TABLE IF EXISTS auth.mfa_factors
  ADD CONSTRAINT IF NOT EXISTS mfa_factors_user_id_fkey FOREIGN KEY (user_id) REFERENCES auth.users(id);

-- auth.mfa_challenges.factor_id -> auth.mfa_factors.id
ALTER TABLE IF EXISTS auth.mfa_challenges
  ADD CONSTRAINT IF NOT EXISTS mfa_challenges_auth_factor_id_fkey FOREIGN KEY (factor_id) REFERENCES auth.mfa_factors(id);

-- auth.mfa_amr_claims.session_id -> auth.sessions.id
ALTER TABLE IF EXISTS auth.mfa_amr_claims
  ADD CONSTRAINT IF NOT EXISTS mfa_amr_claims_session_id_fkey FOREIGN KEY (session_id) REFERENCES auth.sessions(id);

-- auth.oauth_authorizations.client_id -> auth.oauth_clients.id
ALTER TABLE IF EXISTS auth.oauth_authorizations
  ADD CONSTRAINT IF NOT EXISTS oauth_authorizations_client_id_fkey FOREIGN KEY (client_id) REFERENCES auth.oauth_clients(id);

-- auth.oauth_authorizations.user_id -> auth.users.id
ALTER TABLE IF EXISTS auth.oauth_authorizations
  ADD CONSTRAINT IF NOT EXISTS oauth_authorizations_user_id_fkey FOREIGN KEY (user_id) REFERENCES auth.users(id);

-- auth.oauth_consents.user_id -> auth.users.id
ALTER TABLE IF EXISTS auth.oauth_consents
  ADD CONSTRAINT IF NOT EXISTS oauth_consents_user_id_fkey FOREIGN KEY (user_id) REFERENCES auth.users(id);

-- auth.oauth_consents.client_id -> auth.oauth_clients.id
ALTER TABLE IF EXISTS auth.oauth_consents
  ADD CONSTRAINT IF NOT EXISTS oauth_consents_client_id_fkey FOREIGN KEY (client_id) REFERENCES auth.oauth_clients(id);

-- auth.sessions.user_id -> auth.users.id
ALTER TABLE IF EXISTS auth.sessions
  ADD CONSTRAINT IF NOT EXISTS sessions_user_id_fkey FOREIGN KEY (user_id) REFERENCES auth.users(id);

-- auth.sessions.oauth_client_id -> auth.oauth_clients.id
ALTER TABLE IF EXISTS auth.sessions
  ADD CONSTRAINT IF NOT EXISTS sessions_oauth_client_id_fkey FOREIGN KEY (oauth_client_id) REFERENCES auth.oauth_clients(id);

-- auth.one_time_tokens.user_id -> auth.users.id
ALTER TABLE IF EXISTS auth.one_time_tokens
  ADD CONSTRAINT IF NOT EXISTS one_time_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES auth.users(id);

-- auth.refresh_tokens.session_id -> auth.sessions.id
ALTER TABLE IF EXISTS auth.refresh_tokens
  ADD CONSTRAINT IF NOT EXISTS refresh_tokens_session_id_fkey FOREIGN KEY (session_id) REFERENCES auth.sessions(id);

-- app.subscriptions.user_id -> auth.users.id
ALTER TABLE IF EXISTS app.subscriptions
  ADD CONSTRAINT IF NOT EXISTS subscriptions_user_id_fkey FOREIGN KEY (user_id) REFERENCES auth.users(id);

-- app.litellm_keys.user_id -> auth.users.id
ALTER TABLE IF EXISTS app.litellm_keys
  ADD CONSTRAINT IF NOT EXISTS litellm_keys_user_id_fkey FOREIGN KEY (user_id) REFERENCES auth.users(id);

-- app.litellm_keys.subscription_id -> app.subscriptions.subscription_id
ALTER TABLE IF EXISTS app.litellm_keys
  ADD CONSTRAINT IF NOT EXISTS litellm_keys_subscription_id_fkey FOREIGN KEY (subscription_id) REFERENCES app.subscriptions(subscription_id);

-- storage.objects.bucket_id -> storage.buckets.id
ALTER TABLE IF EXISTS storage.objects
  ADD CONSTRAINT IF NOT EXISTS objects_bucketId_fkey FOREIGN KEY (bucket_id) REFERENCES storage.buckets(id);

-- storage.s3_multipart_uploads.bucket_id -> storage.buckets.id
ALTER TABLE IF EXISTS storage.s3_multipart_uploads
  ADD CONSTRAINT IF NOT EXISTS s3_multipart_uploads_bucket_id_fkey FOREIGN KEY (bucket_id) REFERENCES storage.buckets(id);

-- storage.s3_multipart_uploads_parts.upload_id -> storage.s3_multipart_uploads.id
ALTER TABLE IF EXISTS storage.s3_multipart_uploads_parts
  ADD CONSTRAINT IF NOT EXISTS s3_multipart_uploads_parts_upload_id_fkey FOREIGN KEY (upload_id) REFERENCES storage.s3_multipart_uploads(id);

-- storage.s3_multipart_uploads_parts.bucket_id -> storage.buckets.id
ALTER TABLE IF EXISTS storage.s3_multipart_uploads_parts
  ADD CONSTRAINT IF NOT EXISTS s3_multipart_uploads_parts_bucket_id_fkey FOREIGN KEY (bucket_id) REFERENCES storage.buckets(id);

-- storage.prefixes.bucket_id -> storage.buckets.id
ALTER TABLE IF EXISTS storage.prefixes
  ADD CONSTRAINT IF NOT EXISTS prefixes_bucketId_fkey FOREIGN KEY (bucket_id) REFERENCES storage.buckets(id);

-- storage.buckets_vectors.id -> storage.vector_indexes.bucket_id (reverse FK earlier)
ALTER TABLE IF EXISTS storage.vector_indexes
  ADD CONSTRAINT IF NOT EXISTS vector_indexes_bucket_id_fkey FOREIGN KEY (bucket_id) REFERENCES storage.buckets_vectors(id);

-- vault.secrets.key_id -> (if referencing a key table, add FK) -- omitted (unknown)

-- =========================
-- Indexes & Additional Constraints
-- =========================
-- Recreate any explicitly known unique constraints already created above.
-- Add indexes for FK columns for performance (recommended)
CREATE INDEX IF NOT EXISTS idx_auth_sessions_user_id ON auth.sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_auth_refresh_tokens_session_id ON auth.refresh_tokens(session_id);
CREATE INDEX IF NOT EXISTS idx_storage_objects_bucket_id ON storage.objects(bucket_id);
CREATE INDEX IF NOT EXISTS idx_storage_s3_uploads_bucket_id ON storage.s3_multipart_uploads(bucket_id);
CREATE INDEX IF NOT EXISTS idx_app_litellm_keys_user_id ON app.litellm_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_app_subscriptions_user_id ON app.subscriptions(user_id);

-- =========================
-- Row Level Security Policies
-- =========================
-- Source had RLS enabled on many tables. Policies themselves are not recreated automatically.
-- Placeholders:
-- Example: ALTER TABLE auth.users ENABLE ROW LEVEL SECURITY; -- already set above
-- CREATE POLICY "example_policy" ON auth.users FOR SELECT TO authenticated USING ((true)); -- adjust per-app

-- =========================
-- Final notes
-- =========================
-- 1) There are references to custom functions/types (e.g., vault._crypto_aead_det_noncegen(), auth.code_challenge_method, realtime.user_defined_filter, realtime.to_regrole) that may be Supabase-managed. Recreate or adjust them as needed in the target DB.
-- 2) If you need RLS policies, triggers, or Supabase-specific functions recreated as well, confirm and I will add them (some may rely on Supabase internals).
-- 3) Run this script in a test environment first. If any CREATE TYPE or CREATE TABLE fails due to missing dependent objects, create those prerequisites or let me know the error and I will correct it.
