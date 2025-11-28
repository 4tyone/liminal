mod commands;
mod models;
mod services;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            // Config commands
            get_api_key,
            set_api_key,
            get_base_url,
            set_base_url,
            get_model,
            set_model,
            get_config,
            // Project commands
            list_projects,
            get_project,
            create_project,
            delete_project,
            // Page commands
            get_page_content,
            save_page_content,
            add_page,
            reorder_pages,
            // AI commands
            generate_learning,
            expand_selection,
            remove_expansion,
            // Export commands
            export_to_pdf,
            get_exports_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
