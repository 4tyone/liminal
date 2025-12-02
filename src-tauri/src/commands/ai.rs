use crate::models::{ProjectMeta, SelectionRange, ExpansionResult};
use crate::services::{ai_service, config_service};
use tauri::AppHandle;

#[tauri::command]
pub async fn generate_learning(app: AppHandle, topic: String, depth: String) -> Result<ProjectMeta, String> {
    let api_key = config_service::get_api_key()?
        .ok_or("API key not configured")?;

    ai_service::generate_learning_material(&topic, &depth, &api_key, app).await
}

#[tauri::command]
pub async fn expand_selection(
    project_id: String,
    page_name: String,
    selection: SelectionRange,
    question: String,
) -> Result<ExpansionResult, String> {
    let api_key = config_service::get_api_key()?
        .ok_or("API key not configured")?;

    ai_service::expand_selection_with_ai(
        &project_id,
        &page_name,
        &selection,
        &question,
        &api_key,
    ).await
}

#[tauri::command]
pub fn remove_expansion(
    project_id: String,
    page_name: String,
    expansion_id: String,
) -> Result<String, String> {
    use crate::services::file_service::{load_page_content, save_page_content};

    let content = load_page_content(&project_id, &page_name)?;

    // Remove the expansion block by finding and removing the details element
    let pattern = format!(r#"<details class="ai-expansion" data-expansion-id="{}".*?</details>"#, expansion_id);
    let re = regex::Regex::new(&pattern).map_err(|e| e.to_string())?;
    let updated = re.replace(&content, "").to_string();

    save_page_content(&project_id, &page_name, &updated)?;

    Ok(updated)
}

#[tauri::command]
pub async fn answer_question(
    selection: SelectionRange,
    question: String,
) -> Result<String, String> {
    let api_key = config_service::get_api_key()?
        .ok_or("API key not configured")?;

    ai_service::answer_question(&selection, &question, &api_key).await
}
