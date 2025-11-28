use crate::services::config_service;

#[tauri::command]
pub fn get_api_key() -> Result<Option<String>, String> {
    config_service::get_api_key()
}

#[tauri::command]
pub fn set_api_key(key: String) -> Result<(), String> {
    config_service::set_api_key(&key)
}

#[tauri::command]
pub fn get_base_url() -> Result<Option<String>, String> {
    config_service::get_base_url()
}

#[tauri::command]
pub fn set_base_url(url: String) -> Result<(), String> {
    config_service::set_base_url(&url)
}

#[tauri::command]
pub fn get_model() -> Result<Option<String>, String> {
    config_service::get_model()
}

#[tauri::command]
pub fn set_model(model: String) -> Result<(), String> {
    config_service::set_model(&model)
}

#[tauri::command]
pub fn get_config() -> Result<config_service::Config, String> {
    config_service::get_full_config()
}
