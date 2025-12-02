use std::fs;
use serde::{Deserialize, Serialize};
use super::file_service::get_app_data_dir;

// Default values (OpenAI as the most common provider)
pub const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
pub const DEFAULT_MODEL: &str = "gpt-5.1";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
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
    if key.is_empty() {
        config.api_key = None;
    } else {
        config.api_key = Some(key.to_string());
    }
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

pub fn get_provider() -> Result<Option<String>, String> {
    let config = load_config()?;
    Ok(config.provider)
}

pub fn set_provider(provider: &str) -> Result<(), String> {
    let mut config = load_config().unwrap_or_default();
    config.provider = Some(provider.to_string());
    save_config(&config)
}

pub fn get_full_config() -> Result<Config, String> {
    load_config()
}

/// Get effective config values with defaults applied
pub fn get_effective_config() -> Result<(String, String, String, String), String> {
    let config = load_config()?;

    let provider = config.provider.unwrap_or_else(|| "openai".to_string());
    let api_key = config.api_key.unwrap_or_default();
    let base_url = config.base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
    let model = config.model.unwrap_or_else(|| DEFAULT_MODEL.to_string());

    Ok((provider, base_url, model, api_key))
}
