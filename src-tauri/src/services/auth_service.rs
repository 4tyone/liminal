use serde::{Deserialize, Serialize};
use std::fs;
use super::file_service::get_app_data_dir;

// ============================================================================
// AUTH DATA STRUCTURES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthState {
    #[serde(default)]
    pub access_token: Option<String>,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub expires_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub user_id: String,
    pub email: String,
    pub subscribed: bool,
}

/// Response from Supabase token exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub token_type: String,
    pub user: SupabaseUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseUser {
    pub id: String,
    pub email: Option<String>,
}

/// Response from my-key edge function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyKeyResponse {
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub virtual_key: Option<String>,
    #[serde(default)]
    pub has_key: Option<bool>,
    #[serde(default)]
    pub subscription_status: Option<String>,
}

// ============================================================================
// CONFIGURATION - Supabase
// ============================================================================

const SUPABASE_URL: &str = "https://nvmpgdnrufodyfcdykna.supabase.co";
const SUPABASE_ANON_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im52bXBnZG5ydWZvZHlmY2R5a25hIiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjQzMDg3NjUsImV4cCI6MjA3OTg4NDc2NX0.XE70p9-nFpAsdiEFjk-q69XUOhN5pb6sqLnLGsQQg5U";

// Landing page URL for auth
const LANDING_PAGE_URL: &str = "https://liminal.melshakobyan.com";

// Deep link scheme for the app
const DEEP_LINK_SCHEME: &str = "liminal";

// ============================================================================
// AUTH STATE PERSISTENCE
// ============================================================================

fn get_auth_path() -> Result<std::path::PathBuf, String> {
    Ok(get_app_data_dir()?.join("auth.json"))
}

pub fn load_auth_state() -> Result<AuthState, String> {
    let auth_path = get_auth_path()?;

    if !auth_path.exists() {
        return Ok(AuthState::default());
    }

    let content = fs::read_to_string(&auth_path)
        .map_err(|e| format!("Failed to read auth state: {}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse auth state: {}", e))
}

pub fn save_auth_state(state: &AuthState) -> Result<(), String> {
    let auth_path = get_auth_path()?;
    let content = serde_json::to_string_pretty(state)
        .map_err(|e| format!("Failed to serialize auth state: {}", e))?;
    fs::write(&auth_path, content)
        .map_err(|e| format!("Failed to write auth state: {}", e))?;
    Ok(())
}

pub fn clear_auth_state() -> Result<(), String> {
    let auth_path = get_auth_path()?;
    if auth_path.exists() {
        fs::remove_file(&auth_path)
            .map_err(|e| format!("Failed to remove auth state: {}", e))?;
    }
    Ok(())
}

// ============================================================================
// SUPABASE AUTH FLOW
// ============================================================================

/// Generate the Supabase OAuth authorization URL
/// Uses PKCE flow - redirects through landing page which handles the OAuth
pub fn get_auth_url(state: &str) -> String {
    let redirect_uri = format!("{}://auth/callback", DEEP_LINK_SCHEME);

    // Redirect to landing page auth, which will handle Supabase OAuth
    // and redirect back to the app with tokens
    format!(
        "{}/auth.html?app_redirect={}&state={}",
        LANDING_PAGE_URL,
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(state)
    )
}

/// Exchange Supabase refresh token for new access token
pub async fn refresh_access_token(refresh_token: &str) -> Result<SupabaseTokenResponse, String> {
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/auth/v1/token?grant_type=refresh_token", SUPABASE_URL))
        .header("apikey", SUPABASE_ANON_KEY)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "refresh_token": refresh_token
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to refresh token: {}", e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Token refresh failed: {}", error_text));
    }

    response
        .json::<SupabaseTokenResponse>()
        .await
        .map_err(|e| format!("Failed to parse refresh response: {}", e))
}

/// Fetch the API key from the my-key edge function
pub async fn fetch_api_key(access_token: &str) -> Result<String, String> {
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/functions/v1/my-key", SUPABASE_URL))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("apikey", SUPABASE_ANON_KEY)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch API key: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();

        if status.as_u16() == 401 {
            return Err("Token expired".to_string());
        }

        return Err(format!("Failed to fetch API key: {}", error_text));
    }

    let my_key_response: MyKeyResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse API key response: {}", e))?;

    // Get API key from response (check multiple possible field names)
    my_key_response.key
        .or(my_key_response.api_key)
        .or(my_key_response.virtual_key)
        .ok_or_else(|| "No API key in response - subscription may be required".to_string())
}

/// Get user info from Supabase
pub async fn get_user_info(access_token: &str) -> Result<UserInfo, String> {
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/auth/v1/user", SUPABASE_URL))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("apikey", SUPABASE_ANON_KEY)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch user info: {}", e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Failed to fetch user info: {}", error_text));
    }

    let user: SupabaseUser = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse user info: {}", e))?;

    Ok(UserInfo {
        user_id: user.id,
        email: user.email.unwrap_or_default(),
        subscribed: false, // Will be determined by my-key response
    })
}

// ============================================================================
// HIGH-LEVEL AUTH OPERATIONS
// ============================================================================

/// Handle the OAuth callback with tokens from the landing page
/// The landing page will redirect with: liminal://auth/callback?access_token=xxx&refresh_token=yyy&expires_in=zzz
pub async fn handle_auth_callback(
    access_token: &str,
    refresh_token: &str,
    expires_in: i64,
    user_id: &str,
    email: &str,
) -> Result<AuthState, String> {
    // Calculate expiration time
    let expires_at = chrono::Utc::now().timestamp() + expires_in;

    // Create auth state
    let auth_state = AuthState {
        access_token: Some(access_token.to_string()),
        refresh_token: Some(refresh_token.to_string()),
        user_id: Some(user_id.to_string()),
        email: Some(email.to_string()),
        expires_at: Some(expires_at),
    };

    // Save auth state
    save_auth_state(&auth_state)?;

    Ok(auth_state)
}

/// Get a valid access token, refreshing if necessary
pub async fn get_valid_access_token() -> Result<String, String> {
    let mut state = load_auth_state()?;

    let access_token = state.access_token.as_ref()
        .ok_or("Not authenticated")?;

    // Check if token is expired (with 60 second buffer)
    let is_expired = state.expires_at
        .map(|exp| chrono::Utc::now().timestamp() >= exp - 60)
        .unwrap_or(false);

    if is_expired {
        // Try to refresh
        if let Some(refresh_token) = &state.refresh_token {
            let token_response = refresh_access_token(refresh_token).await?;

            // Update state
            state.access_token = Some(token_response.access_token.clone());
            state.refresh_token = Some(token_response.refresh_token);
            state.expires_at = Some(chrono::Utc::now().timestamp() + token_response.expires_in);
            state.user_id = Some(token_response.user.id);
            state.email = token_response.user.email;

            save_auth_state(&state)?;

            return Ok(token_response.access_token);
        } else {
            return Err("Token expired and no refresh token available".to_string());
        }
    }

    Ok(access_token.clone())
}

/// Fetch and store the API key
pub async fn fetch_and_store_api_key() -> Result<String, String> {
    let access_token = get_valid_access_token().await?;
    let api_key = fetch_api_key(&access_token).await?;

    // Store in config
    super::config_service::set_api_key(&api_key)?;

    Ok(api_key)
}

/// Sign out - clear all auth state
pub fn sign_out() -> Result<(), String> {
    clear_auth_state()?;
    Ok(())
}
