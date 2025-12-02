// Tauri API wrapper
const { invoke } = window.__TAURI__.core;

// Config commands
export async function getApiKey() {
  return await invoke('get_api_key');
}

export async function setApiKey(key) {
  return await invoke('set_api_key', { key });
}

export async function getBaseUrl() {
  return await invoke('get_base_url');
}

export async function setBaseUrl(url) {
  return await invoke('set_base_url', { url });
}

export async function getModel() {
  return await invoke('get_model');
}

export async function setModel(model) {
  return await invoke('set_model', { model });
}

export async function getConfig() {
  return await invoke('get_config');
}

// Project commands
export async function listProjects() {
  return await invoke('list_projects');
}

export async function getProject(id) {
  return await invoke('get_project', { id });
}

export async function createProject(title, description) {
  return await invoke('create_project', { title, description });
}

export async function deleteProject(id) {
  return await invoke('delete_project', { id });
}

// Page commands
export async function getPageContent(projectId, pageName) {
  return await invoke('get_page_content', { projectId, pageName });
}

export async function savePageContent(projectId, pageName, content) {
  return await invoke('save_page_content', { projectId, pageName, content });
}

export async function addPage(projectId, title) {
  return await invoke('add_page', { projectId, title });
}

export async function reorderPages(projectId, order) {
  return await invoke('reorder_pages', { projectId, order });
}

export async function importFolder(folderPath, title, description) {
  return await invoke('import_folder', { folderPath, title, description });
}

// AI commands (stubs - you will implement these)
export async function generateLearning(topic, depth) {
  return await invoke('generate_learning', { topic, depth });
}

export async function expandSelection(projectId, pageName, selection, question) {
  return await invoke('expand_selection', { projectId, pageName, selection, question });
}

export async function removeExpansion(projectId, pageName, expansionId) {
  return await invoke('remove_expansion', { projectId, pageName, expansionId });
}

export async function answerQuestion(selection, question) {
  return await invoke('answer_question', { selection, question });
}

// Chat commands
export async function listChatSessions(projectId) {
  return await invoke('list_chat_sessions', { projectId });
}

export async function createChatSession(projectId) {
  return await invoke('create_chat_session', { projectId });
}

export async function getChatSession(projectId, sessionId) {
  return await invoke('get_chat_session', { projectId, sessionId });
}

export async function deleteChatSession(projectId, sessionId) {
  return await invoke('delete_chat_session', { projectId, sessionId });
}

export async function sendChatMessage(projectId, sessionId, message) {
  return await invoke('send_chat_message', { projectId, sessionId, message });
}

// Export commands
export async function exportToPdf(projectId, outputPath) {
  return await invoke('export_to_pdf', { projectId, outputPath });
}

export async function getExportsDir() {
  return await invoke('get_exports_dir');
}
