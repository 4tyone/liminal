use std::fs;
use std::path::PathBuf;
use crate::models::{ProjectMeta, ProjectListItem, ChatSession, ChatSessionListItem, ChatMessage};
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

/// Import a folder of markdown files as a new project
pub fn import_folder_as_project(folder_path: &str, title: &str, description: &str) -> Result<ProjectMeta, String> {
    let folder = std::path::Path::new(folder_path);

    if !folder.exists() || !folder.is_dir() {
        return Err("Invalid folder path".to_string());
    }

    // Collect all markdown files
    let mut md_files: Vec<_> = fs::read_dir(folder)
        .map_err(|e| format!("Failed to read folder: {}", e))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let path = entry.path();
            path.is_file() && path.extension().map_or(false, |ext| ext == "md")
        })
        .collect();

    if md_files.is_empty() {
        return Err("No markdown files found in folder".to_string());
    }

    // Sort by filename to maintain order
    md_files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    // Create new project
    let meta = create_new_project(title, description)?;

    // Import each markdown file
    for (index, entry) in md_files.iter().enumerate() {
        let file_path = entry.path();
        let content = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read file {:?}: {}", file_path, e))?;

        // Extract title from filename or first heading
        let file_stem = file_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled");

        // Try to extract title from first # heading, otherwise use filename
        let page_title = content.lines()
            .find(|line| line.starts_with("# "))
            .map(|line| line.trim_start_matches("# ").to_string())
            .unwrap_or_else(|| {
                // Clean up filename: remove leading numbers and dashes
                let cleaned = file_stem
                    .trim_start_matches(|c: char| c.is_numeric() || c == '-' || c == '_')
                    .replace('-', " ")
                    .replace('_', " ");
                if cleaned.is_empty() { file_stem.to_string() } else { cleaned }
            });

        // Generate page filename with order prefix
        let page_name = format!("{:02}-{}.md", index + 1, slug::slugify(&page_title));

        // Save page content
        let pages_dir = get_project_dir(&meta.id)?.join("pages");
        if !pages_dir.exists() {
            fs::create_dir_all(&pages_dir).map_err(|e| e.to_string())?;
        }
        fs::write(pages_dir.join(&page_name), &content)
            .map_err(|e| format!("Failed to write page: {}", e))?;

        // Update project meta with page
        let mut updated_meta = load_project(&meta.id)?;
        updated_meta.page_order.push(page_name);
        updated_meta.updated_at = Utc::now();
        save_project(&updated_meta)?;
    }

    // Return the final project meta
    load_project(&meta.id)
}

// ============================================================================
// Chat Session Functions
// ============================================================================

pub fn get_chats_dir(project_id: &str) -> Result<PathBuf, String> {
    let chats_dir = get_project_dir(project_id)?.join("chats");

    if !chats_dir.exists() {
        fs::create_dir_all(&chats_dir).map_err(|e| e.to_string())?;
    }

    Ok(chats_dir)
}

pub fn list_chat_sessions(project_id: &str) -> Result<Vec<ChatSessionListItem>, String> {
    let chats_dir = get_chats_dir(project_id)?;
    let mut sessions = Vec::new();

    if let Ok(entries) = fs::read_dir(&chats_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<ChatSession>(&content) {
                        sessions.push(ChatSessionListItem::from(&session));
                    }
                }
            }
        }
    }

    // Sort by updated_at descending
    sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    Ok(sessions)
}

pub fn create_chat_session(project_id: &str, title: &str) -> Result<ChatSession, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();

    let session = ChatSession {
        id: id.clone(),
        project_id: project_id.to_string(),
        title: title.to_string(),
        messages: Vec::new(),
        created_at: now,
        updated_at: now,
    };

    save_chat_session(&session)?;
    Ok(session)
}

pub fn load_chat_session(project_id: &str, session_id: &str) -> Result<ChatSession, String> {
    let session_path = get_chats_dir(project_id)?.join(format!("{}.json", session_id));
    let content = fs::read_to_string(&session_path)
        .map_err(|e| format!("Failed to read chat session: {}", e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse chat session: {}", e))
}

pub fn save_chat_session(session: &ChatSession) -> Result<(), String> {
    let chats_dir = get_chats_dir(&session.project_id)?;
    let session_path = chats_dir.join(format!("{}.json", session.id));

    let content = serde_json::to_string_pretty(session)
        .map_err(|e| format!("Failed to serialize chat session: {}", e))?;
    fs::write(&session_path, content)
        .map_err(|e| format!("Failed to write chat session: {}", e))?;

    Ok(())
}

pub fn add_message_to_session(
    project_id: &str,
    session_id: &str,
    role: &str,
    content: &str
) -> Result<ChatSession, String> {
    let mut session = load_chat_session(project_id, session_id)?;

    session.messages.push(ChatMessage {
        role: role.to_string(),
        content: content.to_string(),
        timestamp: Utc::now(),
    });
    session.updated_at = Utc::now();

    // Update title from first user message if it's still "New Chat"
    if session.title == "New Chat" && role == "user" {
        let title = content.chars().take(50).collect::<String>();
        session.title = if content.len() > 50 {
            format!("{}...", title)
        } else {
            title
        };
    }

    save_chat_session(&session)?;
    Ok(session)
}

pub fn delete_chat_session(project_id: &str, session_id: &str) -> Result<(), String> {
    let session_path = get_chats_dir(project_id)?.join(format!("{}.json", session_id));
    if session_path.exists() {
        fs::remove_file(&session_path).map_err(|e| e.to_string())?;
    }
    Ok(())
}
