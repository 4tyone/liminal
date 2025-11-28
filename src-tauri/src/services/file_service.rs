use std::fs;
use std::path::PathBuf;
use crate::models::{ProjectMeta, ProjectListItem};
use chrono::Utc;
use uuid::Uuid;

pub fn get_app_data_dir() -> Result<PathBuf, String> {
    let data_dir = dirs::data_dir()
        .ok_or("Could not find data directory")?
        .join("Liminal");

    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    }

    Ok(data_dir)
}

pub fn get_projects_dir() -> Result<PathBuf, String> {
    let projects_dir = get_app_data_dir()?.join("projects");

    if !projects_dir.exists() {
        fs::create_dir_all(&projects_dir).map_err(|e| e.to_string())?;
    }

    Ok(projects_dir)
}

pub fn get_project_dir(project_id: &str) -> Result<PathBuf, String> {
    let project_dir = get_projects_dir()?.join(project_id);
    Ok(project_dir)
}

pub fn list_all_projects() -> Result<Vec<ProjectListItem>, String> {
    let projects_dir = get_projects_dir()?;
    let mut projects = Vec::new();

    if let Ok(entries) = fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let meta_path = entry.path().join("meta.json");
                if meta_path.exists() {
                    if let Ok(content) = fs::read_to_string(&meta_path) {
                        if let Ok(meta) = serde_json::from_str::<ProjectMeta>(&content) {
                            projects.push(meta.into());
                        }
                    }
                }
            }
        }
    }

    // Sort by updated_at descending
    projects.sort_by(|a: &ProjectListItem, b: &ProjectListItem| b.updated_at.cmp(&a.updated_at));

    Ok(projects)
}

pub fn load_project(project_id: &str) -> Result<ProjectMeta, String> {
    let meta_path = get_project_dir(project_id)?.join("meta.json");
    let content = fs::read_to_string(&meta_path)
        .map_err(|e| format!("Failed to read project: {}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse project: {}", e))
}

pub fn save_project(meta: &ProjectMeta) -> Result<(), String> {
    let project_dir = get_project_dir(&meta.id)?;

    if !project_dir.exists() {
        fs::create_dir_all(&project_dir).map_err(|e| e.to_string())?;
    }

    let pages_dir = project_dir.join("pages");
    if !pages_dir.exists() {
        fs::create_dir_all(&pages_dir).map_err(|e| e.to_string())?;
    }

    let meta_path = project_dir.join("meta.json");
    let content = serde_json::to_string_pretty(meta)
        .map_err(|e| format!("Failed to serialize project: {}", e))?;
    fs::write(&meta_path, content)
        .map_err(|e| format!("Failed to write project: {}", e))?;

    Ok(())
}

pub fn delete_project_dir(project_id: &str) -> Result<(), String> {
    let project_dir = get_project_dir(project_id)?;
    if project_dir.exists() {
        fs::remove_dir_all(&project_dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn load_page_content(project_id: &str, page_name: &str) -> Result<String, String> {
    let page_path = get_project_dir(project_id)?.join("pages").join(page_name);
    fs::read_to_string(&page_path)
        .map_err(|e| format!("Failed to read page: {}", e))
}

pub fn save_page_content(project_id: &str, page_name: &str, content: &str) -> Result<(), String> {
    let pages_dir = get_project_dir(project_id)?.join("pages");

    if !pages_dir.exists() {
        fs::create_dir_all(&pages_dir).map_err(|e| e.to_string())?;
    }

    let page_path = pages_dir.join(page_name);
    fs::write(&page_path, content)
        .map_err(|e| format!("Failed to write page: {}", e))?;

    // Update project's updated_at
    if let Ok(mut meta) = load_project(project_id) {
        meta.updated_at = Utc::now();
        let _ = save_project(&meta);
    }

    Ok(())
}

pub fn create_new_project(title: &str, description: &str) -> Result<ProjectMeta, String> {
    // Use UUID for project ID to avoid filename length issues with long prompts
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();

    let meta = ProjectMeta {
        id: id.clone(),
        title: title.to_string(),
        description: description.to_string(),
        created_at: now,
        updated_at: now,
        page_order: Vec::new(),
    };

    save_project(&meta)?;
    Ok(meta)
}

pub fn add_page_to_project(project_id: &str, title: &str, content: &str) -> Result<String, String> {
    let mut meta = load_project(project_id)?;

    // Generate page filename
    let page_num = meta.page_order.len() + 1;
    let page_name = format!("{:02}-{}.md", page_num, slug::slugify(title));

    // Save the page content
    save_page_content(project_id, &page_name, content)?;

    // Update page order
    meta.page_order.push(page_name.clone());
    meta.updated_at = Utc::now();
    save_project(&meta)?;

    Ok(page_name)
}
