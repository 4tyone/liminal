use std::fs;
use serde::{Deserialize, Serialize};
use super::file_service::get_app_data_dir;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub theme: String,
}

fn get_config_path() -> Result<std::path::PathBuf, String> {
    Ok(get_app_data_dir()?.join("config.json"))
}

pub fn load_config() -> Result<Config, String> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        return Ok(Config::default());
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse config: {}", e))
}

pub fn save_config(config: &Config) -> Result<(), String> {
    let config_path = get_config_path()?;
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config: {}", e))?;
    Ok(())
}

pub fn get_api_key() -> Result<Option<String>, String> {
    let config = load_config()?;
    Ok(config.api_key)
}

pub fn set_api_key(key: &str) -> Result<(), String> {
    let mut config = load_config().unwrap_or_default();
    config.api_key = Some(key.to_string());
    save_config(&config)
}

pub fn get_base_url() -> Result<Option<String>, String> {
    let config = load_config()?;
    Ok(config.base_url)
}

pub fn set_base_url(url: &str) -> Result<(), String> {
    let mut config = load_config().unwrap_or_default();
    config.base_url = Some(url.to_string());
    save_config(&config)
}

pub fn get_model() -> Result<Option<String>, String> {
    let config = load_config()?;
    Ok(config.model)
}

pub fn set_model(model: &str) -> Result<(), String> {
    let mut config = load_config().unwrap_or_default();
    config.model = Some(model.to_string());
    save_config(&config)
}

pub fn get_full_config() -> Result<Config, String> {
    load_config()
}
