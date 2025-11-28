use crate::services::file_service::{load_project, load_page_content, get_app_data_dir};
use crate::services::pdf_service::export_project_to_pdf;

#[tauri::command]
pub async fn export_to_pdf(project_id: String, output_path: String) -> Result<(), String> {
    // Load project metadata
    let project = load_project(&project_id)?;

    // Load all page contents
    let mut pages = Vec::new();
    for page_name in &project.page_order {
        let content = load_page_content(&project_id, page_name)?;
        pages.push(content);
    }

    // Export directly to the user-selected path
    export_project_to_pdf(&project.title, pages, &output_path)?;

    Ok(())
}

#[tauri::command]
pub fn get_exports_dir() -> Result<String, String> {
    let exports_dir = get_app_data_dir()?.join("exports");
    if !exports_dir.exists() {
        std::fs::create_dir_all(&exports_dir)
            .map_err(|e| format!("Failed to create exports directory: {}", e))?;
    }
    Ok(exports_dir.to_string_lossy().to_string())
}
