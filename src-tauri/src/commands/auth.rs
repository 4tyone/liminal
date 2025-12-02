use crate::services::auth_service;
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use uuid::Uuid;
use std::sync::Mutex;

// Store the auth state parameter to verify callbacks
static AUTH_STATE: Mutex<Option<String>> = Mutex::new(None);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatus {
    pub is_authenticated: bool,
    pub email: Option<String>,
    pub user_id: Option<String>,
}

/// Start the sign-in flow by opening the browser to the auth URL
#[tauri::command]
pub async fn start_signin() -> Result<String, String> {
    // Generate a random state parameter for CSRF protection
    let state = Uuid::new_v4().to_string();

    // Store the state to verify later
    {
        let mut stored_state = AUTH_STATE.lock().map_err(|e| e.to_string())?;
        *stored_state = Some(state.clone());
    }

    // Get the auth URL
    let auth_url = auth_service::get_auth_url(&state);

    // Open the URL in the default browser
    open::that(&auth_url).map_err(|e| format!("Failed to open browser: {}", e))?;

    Ok(auth_url)
}

/// Handle the OAuth callback (called when deep link is received)
/// Expects tokens from the landing page: access_token, refresh_token, expires_in, user_id, email
#[tauri::command]
pub async fn handle_auth_callback(
    access_token: String,
    refresh_token: String,
    expires_in: i64,
    user_id: String,
    email: String,
    state: String,
) -> Result<AuthStatus, String> {
    // Verify the state parameter
    {
        let stored_state = AUTH_STATE.lock().map_err(|e| e.to_string())?;
        if let Some(expected_state) = stored_state.as_ref() {
            if expected_state != &state {
                return Err("Invalid state parameter".to_string());
            }
        }
    }

    // Clear the stored state
    {
        let mut stored_state = AUTH_STATE.lock().map_err(|e| e.to_string())?;
        *stored_state = None;
    }

    // Save tokens
    let auth_state = auth_service::handle_auth_callback(
        &access_token,
        &refresh_token,
        expires_in,
        &user_id,
        &email,
    ).await?;

    Ok(AuthStatus {
        is_authenticated: true,
        email: auth_state.email,
        user_id: auth_state.user_id,
    })
}

/// Check if user is currently authenticated
#[tauri::command]
pub fn check_auth_status() -> Result<AuthStatus, String> {
    let auth_state = auth_service::load_auth_state()?;

    Ok(AuthStatus {
        is_authenticated: auth_state.access_token.is_some(),
        email: auth_state.email,
        user_id: auth_state.user_id,
    })
}

/// Get the current user info
#[tauri::command]
pub async fn get_current_user() -> Result<auth_service::UserInfo, String> {
    let access_token = auth_service::get_valid_access_token().await?;
    auth_service::get_user_info(&access_token).await
}

/// Sign out the current user
#[tauri::command]
pub fn signout() -> Result<(), String> {
    auth_service::sign_out()
}

/// Fetch the API key from the server (call on app startup if authenticated)
#[tauri::command]
pub async fn fetch_api_key_from_server() -> Result<String, String> {
    auth_service::fetch_and_store_api_key().await
}

/// Handle deep link URL (called by Tauri when app receives a deep link)
pub fn handle_deep_link(app: &tauri::AppHandle, url: &str) {
    // Parse the URL to extract tokens
    // Expected format: liminal://auth/callback?access_token=xxx&refresh_token=yyy&expires_in=zzz&user_id=aaa&email=bbb&state=ccc

    if !url.starts_with("liminal://auth/callback") {
        return;
    }

    // Parse query parameters
    let url_parsed = match url::Url::parse(url) {
        Ok(u) => u,
        Err(_) => return,
    };

    let mut access_token: Option<String> = None;
    let mut refresh_token: Option<String> = None;
    let mut expires_in: Option<i64> = None;
    let mut user_id: Option<String> = None;
    let mut email: Option<String> = None;
    let mut state: Option<String> = None;

    for (key, value) in url_parsed.query_pairs() {
        match key.as_ref() {
            "access_token" => access_token = Some(value.to_string()),
            "refresh_token" => refresh_token = Some(value.to_string()),
            "expires_in" => expires_in = value.parse().ok(),
            "user_id" => user_id = Some(value.to_string()),
            "email" => email = Some(value.to_string()),
            "state" => state = Some(value.to_string()),
            _ => {}
        }
    }

    // Emit an event to the frontend to handle the callback
    if let (Some(access_token), Some(refresh_token), Some(expires_in), Some(user_id), Some(state)) =
        (access_token, refresh_token, expires_in, user_id, state)
    {
        let _ = app.emit("auth-callback", serde_json::json!({
            "accessToken": access_token,
            "refreshToken": refresh_token,
            "expiresIn": expires_in,
            "userId": user_id,
            "email": email.unwrap_or_default(),
            "state": state
        }));
    }
}
