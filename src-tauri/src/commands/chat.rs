use crate::models::{ChatSession, ChatSessionListItem};
use crate::services::{file_service, ai_service, config_service};
use tauri::AppHandle;

#[tauri::command]
pub fn list_chat_sessions(project_id: String) -> Result<Vec<ChatSessionListItem>, String> {
    file_service::list_chat_sessions(&project_id)
}

#[tauri::command]
pub fn create_chat_session(project_id: String) -> Result<ChatSession, String> {
    file_service::create_chat_session(&project_id, "New Chat")
}

#[tauri::command]
pub fn get_chat_session(project_id: String, session_id: String) -> Result<ChatSession, String> {
    file_service::load_chat_session(&project_id, &session_id)
}

#[tauri::command]
pub fn delete_chat_session(project_id: String, session_id: String) -> Result<(), String> {
    file_service::delete_chat_session(&project_id, &session_id)
}

#[tauri::command]
pub async fn send_chat_message(
    app: AppHandle,
    project_id: String,
    session_id: String,
    message: String,
) -> Result<ai_service::ChatAgentResult, String> {
    let api_key = config_service::get_api_key()?
        .ok_or("API key not configured")?;

    ai_service::run_editing_agent(
        &project_id,
        &session_id,
        &message,
        &api_key,
        app,
    ).await
}
