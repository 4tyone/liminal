use crate::models::{ProjectMeta, ProjectListItem, Page};
use crate::services::file_service;

#[tauri::command]
pub fn list_projects() -> Result<Vec<ProjectListItem>, String> {
    file_service::list_all_projects()
}

#[tauri::command]
pub fn get_project(id: String) -> Result<ProjectMeta, String> {
    file_service::load_project(&id)
}

#[tauri::command]
pub fn create_project(title: String, description: String) -> Result<ProjectMeta, String> {
    file_service::create_new_project(&title, &description)
}

#[tauri::command]
pub fn delete_project(id: String) -> Result<(), String> {
    file_service::delete_project_dir(&id)
}

#[tauri::command]
pub fn get_page_content(project_id: String, page_name: String) -> Result<String, String> {
    file_service::load_page_content(&project_id, &page_name)
}

#[tauri::command]
pub fn save_page_content(project_id: String, page_name: String, content: String) -> Result<(), String> {
    file_service::save_page_content(&project_id, &page_name, &content)
}

#[tauri::command]
pub fn add_page(project_id: String, title: String) -> Result<Page, String> {
    let page_name = file_service::add_page_to_project(&project_id, &title, "# New Page\n\nStart writing here...")?;
    Ok(Page {
        name: page_name.clone(),
        title,
    })
}

#[tauri::command]
pub fn reorder_pages(project_id: String, order: Vec<String>) -> Result<(), String> {
    let mut meta = file_service::load_project(&project_id)?;
    meta.page_order = order;
    file_service::save_project(&meta)
}

#[tauri::command]
pub fn import_folder(folder_path: String, title: String, description: String) -> Result<ProjectMeta, String> {
    file_service::import_folder_as_project(&folder_path, &title, &description)
}
