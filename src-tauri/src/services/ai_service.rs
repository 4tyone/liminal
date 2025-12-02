use crate::models::{ProjectMeta, SelectionRange, ExpansionResult, ChatMessage};
use crate::services::file_service::{
    create_new_project, add_page_to_project, load_page_content,
    save_page_content, load_project, load_chat_session, save_chat_session
};
use crate::services::llm_client::LlmClient;

use uuid::Uuid;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use chrono::Utc;

// ============================================================================
// AGENT TOOL DEFINITIONS
// ============================================================================

/// Tool call parsed from agent response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Result of executing a tool
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub tool_name: String,
    pub success: bool,
    pub output: String,
}

/// Agent state during generation
pub struct AgentState {
    pub project_id: String,
    pub pages: Vec<PageInfo>,
    pub book_title: Option<String>,
    pub is_finished: bool,
    pub iteration: u32,
    pub max_iterations: u32,
}

#[derive(Debug, Clone)]
pub struct PageInfo {
    pub filename: String,
    pub title: String,
}

/// Event payload for agent status updates
#[derive(Debug, Clone, Serialize)]
pub struct AgentStatusEvent {
    pub message: String,
    pub iteration: u32,
    pub tool_name: Option<String>,
}

// ============================================================================
// SYSTEM PROMPT FOR TOOL-USING AGENT
// ============================================================================

const AGENT_SYSTEM_PROMPT: &str = r##"You are an expert educational content creator agent. Your task is to generate comprehensive, book-like learning material on any topic.

## Your Tools

You have access to the following tools to create learning materials:

### 1. create_file
Creates a new markdown page/chapter.
```json
{
  "tool": "create_file",
  "arguments": {
    "title": "Chapter Title",
    "content": "# Chapter Title\n\nYour markdown content here..."
  }
}
```

### 2. edit_file
Edits an existing page by replacing content.
```json
{
  "tool": "edit_file",
  "arguments": {
    "filename": "01-introduction.md",
    "old_content": "Text to find and replace",
    "new_content": "New text to insert"
  }
}
```

### 3. read_file
Reads the content of an existing page.
```json
{
  "tool": "read_file",
  "arguments": {
    "filename": "01-introduction.md"
  }
}
```

### 4. list_files
Lists all pages in the current project.
```json
{
  "tool": "list_files",
  "arguments": {}
}
```

### 5. set_book_info
Sets the title and description of the book. Call this first to give your book a proper name and subtitle.
```json
{
  "tool": "set_book_info",
  "arguments": {
    "title": "A Creative Book Title",
    "description": "A brief, elegant one-sentence description of what the reader will learn"
  }
}
```

### 6. finish
Call this when you have completed creating all the learning material.
```json
{
  "tool": "finish",
  "arguments": {
    "summary": "Brief summary of what was created"
  }
}
```

## How to Respond

Each response should contain exactly ONE tool call in JSON format. Think step by step about what to create next.

Format your response as:
<thinking>
Your reasoning about what to do next...
</thinking>

<tool_call>
{
  "tool": "tool_name",
  "arguments": { ... }
}
</tool_call>

## Guidelines for Content Creation

1. **Structure like a real book**: Create chapters that build on each other progressively
2. **Start with fundamentals**: Begin with an introduction/overview chapter
3. **Use markdown formatting**: Headers, bold, lists, code blocks, etc.
4. **Include practical examples**: Real-world applications and exercises
5. **Maintain consistent style**: Professional yet engaging tone
6. **NEVER use emojis**: Keep content clean and professional
7. **One chapter at a time**: Create chapters sequentially, reviewing structure as you go

## Depth Levels

- **beginner**: Cover basics only. High-level overview without excessive details.
- **intermediate**: Go deeper. Include examples and explain the "why" behind concepts.
- **advanced**: Comprehensive coverage. Technical details, edge cases, best practices.

## Workflow

1. First, set a creative book title and description using set_book_info
2. Create the introduction chapter
3. Create subsequent chapters one by one
4. Review and edit if needed
5. Call finish when complete

IMPORTANT: Always respond with exactly one tool call. Never output raw content without a tool call wrapper."##;

// ============================================================================
// TOOL EXECUTION
// ============================================================================

/// Parse tool call from agent response
fn parse_tool_call(response: &str) -> Result<ToolCall, String> {
    // Try to find JSON in <tool_call> tags first
    let tool_call_re = Regex::new(r"<tool_call>\s*(\{[\s\S]*?\})\s*</tool_call>").unwrap();

    if let Some(captures) = tool_call_re.captures(response) {
        let json_str = captures.get(1).map(|m| m.as_str()).unwrap_or("");
        return parse_tool_json(json_str);
    }

    // Fallback: find any JSON object with "tool" key
    let json_re = Regex::new(r#"\{[^{}]*"tool"[^{}]*\}"#).unwrap();
    if let Some(mat) = json_re.find(response) {
        return parse_tool_json(mat.as_str());
    }

    // Try to find JSON in code blocks
    let code_block_re = Regex::new(r"```(?:json)?\s*(\{[\s\S]*?\})\s*```").unwrap();
    if let Some(captures) = code_block_re.captures(response) {
        let json_str = captures.get(1).map(|m| m.as_str()).unwrap_or("");
        return parse_tool_json(json_str);
    }

    Err(format!("No valid tool call found in response: {}", &response[..response.len().min(200)]))
}

fn parse_tool_json(json_str: &str) -> Result<ToolCall, String> {
    // Parse the JSON
    let parsed: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse tool JSON: {} - Input: {}", e, json_str))?;

    let tool_name = parsed.get("tool")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'tool' field")?
        .to_string();

    let arguments = parsed.get("arguments")
        .cloned()
        .unwrap_or(serde_json::json!({}));

    Ok(ToolCall {
        name: tool_name,
        arguments,
    })
}

/// Execute a tool call and return the result
fn execute_tool(tool_call: &ToolCall, state: &mut AgentState) -> ToolResult {
    match tool_call.name.as_str() {
        "create_file" => execute_create_file(tool_call, state),
        "edit_file" => execute_edit_file(tool_call, state),
        "read_file" => execute_read_file(tool_call, state),
        "list_files" => execute_list_files(state),
        "set_book_info" => execute_set_book_info(tool_call, state),
        "finish" => execute_finish(tool_call, state),
        _ => ToolResult {
            tool_name: tool_call.name.clone(),
            success: false,
            output: format!("Unknown tool: {}", tool_call.name),
        },
    }
}

fn execute_create_file(tool_call: &ToolCall, state: &mut AgentState) -> ToolResult {
    let title = tool_call.arguments.get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled");

    let content = tool_call.arguments.get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    match add_page_to_project(&state.project_id, title, content) {
        Ok(filename) => {
            state.pages.push(PageInfo {
                filename: filename.clone(),
                title: title.to_string(),
            });
            ToolResult {
                tool_name: "create_file".to_string(),
                success: true,
                output: format!("Created page '{}' as {}", title, filename),
            }
        }
        Err(e) => ToolResult {
            tool_name: "create_file".to_string(),
            success: false,
            output: format!("Failed to create page: {}", e),
        },
    }
}

fn execute_edit_file(tool_call: &ToolCall, state: &mut AgentState) -> ToolResult {
    let filename = tool_call.arguments.get("filename")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let old_content = tool_call.arguments.get("old_content")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let new_content = tool_call.arguments.get("new_content")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Load existing content
    let current_content = match load_page_content(&state.project_id, filename) {
        Ok(c) => c,
        Err(e) => return ToolResult {
            tool_name: "edit_file".to_string(),
            success: false,
            output: format!("Failed to read file '{}': {}", filename, e),
        },
    };

    // Perform replacement
    if !current_content.contains(old_content) {
        return ToolResult {
            tool_name: "edit_file".to_string(),
            success: false,
            output: format!("Could not find the specified text in '{}'. Make sure old_content matches exactly.", filename),
        };
    }

    let updated_content = current_content.replacen(old_content, new_content, 1);

    match save_page_content(&state.project_id, filename, &updated_content) {
        Ok(()) => ToolResult {
            tool_name: "edit_file".to_string(),
            success: true,
            output: format!("Successfully edited '{}'", filename),
        },
        Err(e) => ToolResult {
            tool_name: "edit_file".to_string(),
            success: false,
            output: format!("Failed to save edits to '{}': {}", filename, e),
        },
    }
}

fn execute_read_file(tool_call: &ToolCall, state: &AgentState) -> ToolResult {
    let filename = tool_call.arguments.get("filename")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    match load_page_content(&state.project_id, filename) {
        Ok(content) => ToolResult {
            tool_name: "read_file".to_string(),
            success: true,
            output: format!("Content of '{}':\n\n{}", filename, content),
        },
        Err(e) => ToolResult {
            tool_name: "read_file".to_string(),
            success: false,
            output: format!("Failed to read '{}': {}", filename, e),
        },
    }
}

fn execute_list_files(state: &AgentState) -> ToolResult {
    if state.pages.is_empty() {
        return ToolResult {
            tool_name: "list_files".to_string(),
            success: true,
            output: "No pages created yet.".to_string(),
        };
    }

    let file_list: Vec<String> = state.pages.iter()
        .map(|p| format!("- {} ({})", p.filename, p.title))
        .collect();

    ToolResult {
        tool_name: "list_files".to_string(),
        success: true,
        output: format!("Pages in project:\n{}", file_list.join("\n")),
    }
}

fn execute_set_book_info(tool_call: &ToolCall, state: &mut AgentState) -> ToolResult {
    let title = tool_call.arguments.get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled");

    let description = tool_call.arguments.get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    state.book_title = Some(title.to_string());

    // Update the project title and description immediately
    match load_project(&state.project_id) {
        Ok(mut project) => {
            project.title = title.to_string();
            project.description = description.to_string();
            if let Err(e) = crate::services::file_service::save_project(&project) {
                return ToolResult {
                    tool_name: "set_book_info".to_string(),
                    success: false,
                    output: format!("Failed to save book info: {}", e),
                };
            }
            ToolResult {
                tool_name: "set_book_info".to_string(),
                success: true,
                output: format!("Book info set - Title: '{}', Description: '{}'", title, description),
            }
        }
        Err(e) => ToolResult {
            tool_name: "set_book_info".to_string(),
            success: false,
            output: format!("Failed to load project: {}", e),
        },
    }
}

fn execute_finish(tool_call: &ToolCall, state: &mut AgentState) -> ToolResult {
    state.is_finished = true;

    let summary = tool_call.arguments.get("summary")
        .and_then(|v| v.as_str())
        .unwrap_or("Content generation complete.");

    ToolResult {
        tool_name: "finish".to_string(),
        success: true,
        output: format!("Finished: {}", summary),
    }
}

// ============================================================================
// AGENT LOOP
// ============================================================================

/// Extract thinking/reasoning from agent response
fn extract_agent_thinking(response: &str) -> Option<String> {
    // Try to find content in <thinking> tags
    let thinking_re = Regex::new(r"<thinking>\s*([\s\S]*?)\s*</thinking>").unwrap();
    if let Some(captures) = thinking_re.captures(response) {
        if let Some(thinking) = captures.get(1) {
            let text = thinking.as_str().trim();
            // Get first sentence or first 100 chars
            let first_line = text.lines().next().unwrap_or(text);
            let truncated = truncate_text(first_line, 80);
            if !truncated.is_empty() {
                return Some(truncated);
            }
        }
    }

    // Fallback: get first meaningful line that's not a tool call or tag
    for line in response.lines() {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Skip JSON objects
        if trimmed.starts_with('{') || trimmed.starts_with('}') {
            continue;
        }

        // Skip code blocks
        if trimmed.starts_with("```") {
            continue;
        }

        // Skip XML-style tags (but not their content)
        if trimmed.starts_with('<') && (trimmed.ends_with('>') || trimmed.contains("</")) {
            continue;
        }

        // Skip lines that look like JSON keys
        if trimmed.starts_with('"') && trimmed.contains(':') {
            continue;
        }

        // This looks like actual content
        let truncated = truncate_text(trimmed, 80);
        if !truncated.is_empty() {
            return Some(truncated);
        }
    }

    None
}

/// Truncate text to max length, adding ellipsis if needed
fn truncate_text(text: &str, max_len: usize) -> String {
    let trimmed = text.trim();
    if trimmed.len() <= max_len {
        trimmed.to_string()
    } else {
        // Try to break at a word boundary
        let truncated = &trimmed[..max_len];
        if let Some(last_space) = truncated.rfind(' ') {
            if last_space > max_len / 2 {
                return format!("{}...", &trimmed[..last_space]);
            }
        }
        format!("{}...", truncated)
    }
}

/// Emit agent status event to frontend
fn emit_agent_status(app: &AppHandle, message: &str, iteration: u32, tool_name: Option<&str>) {
    let event = AgentStatusEvent {
        message: message.to_string(),
        iteration,
        tool_name: tool_name.map(|s| s.to_string()),
    };
    let _ = app.emit("agent-status", event);
}

/// Generate learning material using an iterative agent loop
pub async fn generate_learning_material(
    topic: &str,
    depth: &str,
    _api_key: &str,
    app: AppHandle,
) -> Result<ProjectMeta, String> {
    // Create LLM client from config
    let client = LlmClient::from_config()?;

    // Emit initial status
    emit_agent_status(&app, "Starting content generation...", 0, None);

    // Create the project first (title and description will be set by the agent)
    let project = create_new_project(topic, "")?;

    // Initialize agent state
    let mut state = AgentState {
        project_id: project.id.clone(),
        pages: Vec::new(),
        book_title: None,
        is_finished: false,
        iteration: 0,
        max_iterations: 30, // Safety limit
    };

    // Build initial user message
    let initial_prompt = format!(
        "Create comprehensive learning material about: {}\n\nDepth level: {}\n\nStart by creating the first chapter (introduction/overview). Then continue creating chapters until you have covered the topic thoroughly at the specified depth level. Call the finish tool when done.",
        topic, depth
    );

    // Message history for the agent
    let mut messages = vec![
        LlmClient::system_message(AGENT_SYSTEM_PROMPT),
        LlmClient::user_message(&initial_prompt),
    ];

    // Agent loop
    while !state.is_finished && state.iteration < state.max_iterations {
        state.iteration += 1;

        // Call the LLM
        let response = client.chat_completion(messages.clone(), Some(0.7)).await?;

        // Extract and emit agent's thinking (if any)
        if let Some(thinking) = extract_agent_thinking(&response) {
            emit_agent_status(&app, &thinking, state.iteration, None);
        }

        // Add assistant response to history
        messages.push(LlmClient::assistant_message(&response));

        // Parse tool call from response
        let tool_call = match parse_tool_call(&response) {
            Ok(tc) => tc,
            Err(e) => {
                // If parsing fails, add error message and continue
                let error_msg = format!("Error parsing your response: {}. Please respond with a valid tool call.", e);
                messages.push(LlmClient::user_message(&error_msg));
                continue;
            }
        };

        // Emit tool execution status
        let tool_status = match tool_call.name.as_str() {
            "set_book_info" => {
                let title = tool_call.arguments.get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("book");
                format!("Naming: {}", title)
            }
            "create_file" => {
                let title = tool_call.arguments.get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("chapter");
                format!("Creating: {}", title)
            }
            "edit_file" => {
                let filename = tool_call.arguments.get("filename")
                    .and_then(|v| v.as_str())
                    .unwrap_or("file");
                format!("Editing: {}", filename)
            }
            "read_file" => {
                let filename = tool_call.arguments.get("filename")
                    .and_then(|v| v.as_str())
                    .unwrap_or("file");
                format!("Reading: {}", filename)
            }
            "list_files" => "Reviewing structure...".to_string(),
            "finish" => "Finalizing content...".to_string(),
            _ => format!("Executing: {}", tool_call.name),
        };
        emit_agent_status(&app, &tool_status, state.iteration, Some(&tool_call.name));

        // Execute the tool
        let result = execute_tool(&tool_call, &mut state);

        // Add tool result to message history
        let result_msg = if result.success {
            format!("Tool '{}' executed successfully:\n{}", result.tool_name, result.output)
        } else {
            format!("Tool '{}' failed:\n{}", result.tool_name, result.output)
        };
        messages.push(LlmClient::user_message(&result_msg));

        // If finished, break the loop
        if state.is_finished {
            emit_agent_status(&app, "Content generation complete!", state.iteration, Some("finish"));
            break;
        }
    }

    // If we hit max iterations without finishing, that's okay - we likely have content
    if state.iteration >= state.max_iterations && !state.is_finished {
        eprintln!("Agent reached max iterations ({}) without calling finish", state.max_iterations);
        emit_agent_status(&app, "Wrapping up...", state.iteration, None);
    }

    // Reload to get updated page order
    load_project(&project.id)
}

// ============================================================================
// EXPANSION AGENT (for inline Q&A)
// ============================================================================

/// System prompt for the content expansion/question-answering agent
const EXPANSION_SYSTEM_PROMPT: &str = r#"You are an expert tutor editing learning material. The student has highlighted text and asked a question. Your task is to UPDATE the document by adding a brief, helpful explanation.

## Your Task
Output a patch that modifies the document to include your explanation. The new content must blend seamlessly with existing text - same style, same formatting, no special markers.

## Patch Format
Use this EXACT format:

*** Begin Patch
*** Update File: content.md
@@ context line from the document
 line to keep unchanged (space prefix)
 another line to keep (space prefix)
+new line to add (plus prefix)
+another new line (plus prefix)
*** End Patch

## Line Prefixes
- Space prefix " " = keep this line unchanged (context)
- Plus prefix "+" = add this new line
- Minus prefix "-" = remove this line

## Rules
1. Find the selected text in the document
2. Add your explanation AFTER the relevant paragraph/section
3. Keep explanations SHORT - 2-4 sentences max
4. Match the document style exactly
5. NO question/answer format
6. NO blockquotes
7. NO special markers or headers for your additions
8. NO emojis
9. Content should look like it was always part of the document

## Example
If the document has:
"Photosynthesis is how plants make food."

And user asks "How does it work?", output:
*** Begin Patch
*** Update File: content.md
@@ Photosynthesis is how plants make food.
 Photosynthesis is how plants make food.
+Plants absorb sunlight through chlorophyll in their leaves. This energy converts carbon dioxide and water into glucose and oxygen.
*** End Patch"#;

/// Parse the patch format and extract file operations
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PatchOperation {
    AddFile { path: String, content: String },
    UpdateFile { path: String, chunks: Vec<UpdateFileChunk> },
    DeleteFile { path: String },
}

/// A chunk within an UpdateFile operation - mirrors Codex's UpdateFileChunk
#[derive(Debug, Clone)]
pub struct UpdateFileChunk {
    /// A single line of context used to narrow down position (usually after @@)
    pub change_context: Option<String>,
    /// Lines that should be replaced (prefixed with - or space in patch)
    pub old_lines: Vec<String>,
    /// Lines that will replace old_lines (prefixed with + or space in patch)
    pub new_lines: Vec<String>,
    /// If true, old_lines must occur at end of file
    pub is_end_of_file: bool,
}

/// Parse a patch string into operations (Codex-style patch format)
pub fn parse_patch(patch: &str) -> Result<Vec<PatchOperation>, String> {
    let mut operations = Vec::new();
    let lines: Vec<&str> = patch.lines().collect();

    let mut i = 0;

    // Find the start of the patch
    while i < lines.len() && !lines[i].trim().starts_with("*** Begin Patch") {
        i += 1;
    }

    if i >= lines.len() {
        return Err("No patch found in output".to_string());
    }

    i += 1; // Skip "*** Begin Patch"

    while i < lines.len() {
        let line = lines[i].trim();

        if line.starts_with("*** End Patch") {
            break;
        }

        if line.starts_with("*** Add File:") {
            let path = line.strip_prefix("*** Add File:").unwrap().trim().to_string();
            let mut content = String::new();
            i += 1;

            while i < lines.len() {
                let content_line = lines[i];
                if content_line.starts_with("***") {
                    break;
                }
                if let Some(stripped) = content_line.strip_prefix('+') {
                    content.push_str(stripped);
                    content.push('\n');
                }
                i += 1;
            }

            operations.push(PatchOperation::AddFile { path, content });
        } else if line.starts_with("*** Update File:") {
            let path = line.strip_prefix("*** Update File:").unwrap().trim().to_string();
            let mut chunks = Vec::new();
            i += 1;

            while i < lines.len() {
                let update_line = lines[i];
                let trimmed = update_line.trim();

                // Stop at next file operation
                if trimmed.starts_with("*** Add File:") ||
                   trimmed.starts_with("*** Update File:") ||
                   trimmed.starts_with("*** Delete File:") ||
                   trimmed.starts_with("*** End Patch") {
                    break;
                }

                // Skip blank lines between chunks
                if trimmed.is_empty() {
                    i += 1;
                    continue;
                }

                // Parse a chunk (starts with @@ or directly with diff lines)
                let (chunk, lines_consumed) = parse_update_chunk(&lines[i..]);
                if let Some(chunk) = chunk {
                    chunks.push(chunk);
                }
                i += lines_consumed.max(1);
            }

            operations.push(PatchOperation::UpdateFile { path, chunks });
        } else if line.starts_with("*** Delete File:") {
            let path = line.strip_prefix("*** Delete File:").unwrap().trim().to_string();
            operations.push(PatchOperation::DeleteFile { path });
            i += 1;
        } else {
            i += 1;
        }
    }

    Ok(operations)
}

/// Parse a single update chunk from lines
fn parse_update_chunk(lines: &[&str]) -> (Option<UpdateFileChunk>, usize) {
    if lines.is_empty() {
        return (None, 0);
    }

    let first_line = lines[0];
    let trimmed = first_line.trim();

    // Check if this is a context marker
    let (change_context, start_idx) = if trimmed == "@@" {
        (None, 1)
    } else if let Some(ctx) = trimmed.strip_prefix("@@ ") {
        (Some(ctx.to_string()), 1)
    } else if trimmed.starts_with("@@") {
        // Handle "@@ context" without space
        let ctx = trimmed.strip_prefix("@@").unwrap().trim();
        if ctx.is_empty() {
            (None, 1)
        } else {
            (Some(ctx.to_string()), 1)
        }
    } else if first_line.starts_with(' ') || first_line.starts_with('+') || first_line.starts_with('-') {
        // No context marker, starts directly with diff lines
        (None, 0)
    } else {
        // Not a valid chunk start
        return (None, 1);
    };

    let mut old_lines = Vec::new();
    let mut new_lines = Vec::new();
    let mut is_end_of_file = false;
    let mut parsed_lines = start_idx;

    for line in &lines[start_idx..] {
        let trimmed = line.trim();

        // Stop conditions
        if trimmed.starts_with("@@") ||
           trimmed.starts_with("*** Add File:") ||
           trimmed.starts_with("*** Update File:") ||
           trimmed.starts_with("*** Delete File:") ||
           trimmed.starts_with("*** End Patch") {
            break;
        }

        if trimmed == "*** End of File" {
            is_end_of_file = true;
            parsed_lines += 1;
            break;
        }

        // Parse diff line
        if line.is_empty() {
            // Empty line counts as context
            old_lines.push(String::new());
            new_lines.push(String::new());
        } else if let Some(rest) = line.strip_prefix(' ') {
            // Context line (keep)
            old_lines.push(rest.to_string());
            new_lines.push(rest.to_string());
        } else if let Some(rest) = line.strip_prefix('+') {
            // Added line
            new_lines.push(rest.to_string());
        } else if let Some(rest) = line.strip_prefix('-') {
            // Removed line
            old_lines.push(rest.to_string());
        } else {
            // Unknown format - might be unprefixed content, treat as end of chunk
            break;
        }

        parsed_lines += 1;
    }

    if old_lines.is_empty() && new_lines.is_empty() {
        return (None, parsed_lines);
    }

    (Some(UpdateFileChunk {
        change_context,
        old_lines,
        new_lines,
        is_end_of_file,
    }), parsed_lines)
}

/// Seek sequence - find pattern in lines starting at position (Codex-style)
fn seek_sequence(lines: &[String], pattern: &[String], start: usize, eof: bool) -> Option<usize> {
    if pattern.is_empty() {
        return Some(start);
    }

    if pattern.len() > lines.len() {
        return None;
    }

    let search_start = if eof && lines.len() >= pattern.len() {
        lines.len() - pattern.len()
    } else {
        start.min(lines.len().saturating_sub(pattern.len()))
    };

    // Exact match first
    for i in search_start..=lines.len().saturating_sub(pattern.len()) {
        if lines[i..i + pattern.len()] == *pattern {
            return Some(i);
        }
    }

    // Then trim match (more lenient)
    for i in search_start..=lines.len().saturating_sub(pattern.len()) {
        let mut ok = true;
        for (p_idx, pat) in pattern.iter().enumerate() {
            if lines[i + p_idx].trim() != pat.trim() {
                ok = false;
                break;
            }
        }
        if ok {
            return Some(i);
        }
    }

    // Try searching from beginning if not found
    if search_start > 0 {
        for i in 0..search_start {
            let mut ok = true;
            for (p_idx, pat) in pattern.iter().enumerate() {
                if i + p_idx >= lines.len() || lines[i + p_idx].trim() != pat.trim() {
                    ok = false;
                    break;
                }
            }
            if ok {
                return Some(i);
            }
        }
    }

    None
}

/// Find a single line anywhere in the document (fuzzy)
fn find_line_fuzzy(lines: &[String], target: &str) -> Option<usize> {
    let target_trimmed = target.trim();

    // Exact match first
    for (i, line) in lines.iter().enumerate() {
        if line.trim() == target_trimmed {
            return Some(i);
        }
    }

    // Contains match (for partial lines)
    for (i, line) in lines.iter().enumerate() {
        if line.contains(target_trimmed) || target_trimmed.contains(line.trim()) {
            return Some(i);
        }
    }

    // Substring match (first significant words)
    let target_words: Vec<&str> = target_trimmed.split_whitespace().take(3).collect();
    if !target_words.is_empty() {
        for (i, line) in lines.iter().enumerate() {
            let line_words: Vec<&str> = line.trim().split_whitespace().take(3).collect();
            if line_words == target_words {
                return Some(i);
            }
        }
    }

    None
}

/// Compute replacements from chunks (Codex-style with fuzzy fallback)
fn compute_replacements(
    original_lines: &[String],
    chunks: &[UpdateFileChunk],
) -> Result<Vec<(usize, usize, Vec<String>)>, String> {
    let mut replacements: Vec<(usize, usize, Vec<String>)> = Vec::new();
    let mut line_index: usize = 0;

    for chunk in chunks {
        // If chunk has change_context, find it first
        if let Some(ctx_line) = &chunk.change_context {
            if let Some(idx) = seek_sequence(
                original_lines,
                &[ctx_line.clone()],
                line_index,
                false,
            ) {
                line_index = idx + 1;
            } else if let Some(idx) = find_line_fuzzy(original_lines, ctx_line) {
                // Fuzzy fallback for context
                line_index = idx + 1;
            }
            // If still not found, continue anyway
        }

        if chunk.old_lines.is_empty() {
            // Pure addition - add at current position or end
            let insertion_idx = if chunk.is_end_of_file {
                original_lines.len()
            } else {
                line_index.min(original_lines.len())
            };
            replacements.push((insertion_idx, 0, chunk.new_lines.clone()));
            continue;
        }

        // Try to find old_lines in the file
        let pattern = &chunk.old_lines[..];
        let mut found = seek_sequence(original_lines, pattern, line_index, chunk.is_end_of_file);

        let new_slice = &chunk.new_lines[..];

        // Retry without trailing empty line if needed
        if found.is_none() && pattern.last().map(|s| s.is_empty()).unwrap_or(false) {
            let shorter_pattern = &pattern[..pattern.len() - 1];
            found = seek_sequence(original_lines, shorter_pattern, line_index, chunk.is_end_of_file);
        }

        // Fuzzy fallback: try to find just the first non-empty line
        if found.is_none() {
            let first_significant = pattern.iter().find(|s| !s.trim().is_empty());
            if let Some(first_line) = first_significant {
                if let Some(idx) = find_line_fuzzy(original_lines, first_line) {
                    // Found the first line - use it as anchor
                    // Count how many old_lines we can match from here
                    let mut match_count = 0;
                    for (i, old_line) in pattern.iter().enumerate() {
                        if idx + i < original_lines.len() {
                            let orig = original_lines[idx + i].trim();
                            let old = old_line.trim();
                            if orig == old || orig.contains(old) || old.contains(orig) {
                                match_count += 1;
                            } else {
                                break;
                            }
                        }
                    }
                    if match_count > 0 {
                        found = Some(idx);
                    }
                }
            }
        }

        // Last resort: if we have context, insert after context position
        if found.is_none() && line_index > 0 {
            // Insert as pure addition after context
            replacements.push((line_index, 0, new_slice.to_vec()));
            continue;
        }

        if let Some(start_idx) = found {
            replacements.push((start_idx, pattern.len(), new_slice.to_vec()));
            line_index = start_idx + pattern.len();
        } else {
            // Ultimate fallback: append to end of document
            replacements.push((original_lines.len(), 0, new_slice.to_vec()));
        }
    }

    replacements.sort_by(|(lhs, _, _), (rhs, _, _)| lhs.cmp(rhs));
    Ok(replacements)
}

/// Apply replacements to lines (Codex-style - reverse order)
/// Returns (updated lines, line numbers, inserted content)
fn apply_replacements(
    mut lines: Vec<String>,
    replacements: &[(usize, usize, Vec<String>)],
) -> (Vec<String>, Vec<usize>, String) {
    let mut updated_line_numbers = Vec::new();
    let mut all_inserted_content = Vec::new();

    // Apply in reverse order so positions stay valid
    for (start_idx, old_len, new_segment) in replacements.iter().rev() {
        let start_idx = *start_idx;
        let old_len = *old_len;

        // Remove old lines
        for _ in 0..old_len {
            if start_idx < lines.len() {
                lines.remove(start_idx);
            }
        }

        // Insert new lines and track the inserted content
        for (offset, new_line) in new_segment.iter().enumerate() {
            lines.insert(start_idx + offset, new_line.clone());
            updated_line_numbers.push(start_idx + offset + 1); // 1-indexed
        }

        // Collect the inserted content
        all_inserted_content.extend(new_segment.clone());
    }

    let inserted_content = all_inserted_content.join("\n");
    (lines, updated_line_numbers, inserted_content)
}

/// Apply update chunks to content (main entry point)
/// Returns (updated content, line numbers, inserted content)
fn apply_update_chunks(original: &str, chunks: &[UpdateFileChunk]) -> Result<(String, Vec<usize>, String), String> {
    let mut original_lines: Vec<String> = original.split('\n').map(String::from).collect();

    // Drop trailing empty element from final newline
    if original_lines.last().map(|s| s.is_empty()).unwrap_or(false) {
        original_lines.pop();
    }

    let replacements = compute_replacements(&original_lines, chunks)?;
    let (mut new_lines, updated_lines, inserted_content) = apply_replacements(original_lines, &replacements);

    // Ensure trailing newline
    if !new_lines.last().map(|s| s.is_empty()).unwrap_or(false) {
        new_lines.push(String::new());
    }

    Ok((new_lines.join("\n"), updated_lines, inserted_content))
}

/// Expand a selection with AI-generated content using Codex-style patches
pub async fn expand_selection_with_ai(
    project_id: &str,
    page_name: &str,
    selection: &SelectionRange,
    question: &str,
    _api_key: &str,
) -> Result<ExpansionResult, String> {
    // Get configuration
    // Create LLM client from config
    let client = LlmClient::from_config()?;

    // Load the current page content
    let content = load_page_content(project_id, page_name)?;

    // Build the expansion prompt with full document context
    let user_prompt = format!(
        "## Current Document\n```\n{}\n```\n\n## Selected Text\n\"{}\"\n\n## Question\n{}",
        content, selection.selected_text, question
    );

    let messages = vec![
        LlmClient::system_message(EXPANSION_SYSTEM_PROMPT),
        LlmClient::user_message(&user_prompt),
    ];

    let response = client.chat_completion(messages, Some(0.7)).await?;

    // Parse the patch from the AI response
    let operations = parse_patch(&response)?;

    // Find UpdateFile operation and apply it
    let (updated_markdown, updated_lines, inserted_content) = {
        let mut result_content = content.clone();
        let mut result_lines = Vec::new();
        let mut result_inserted = String::new();

        for op in operations {
            match op {
                PatchOperation::UpdateFile { chunks, .. } => {
                    if !chunks.is_empty() {
                        let (new_content, lines, inserted) = apply_update_chunks(&result_content, &chunks)?;
                        result_content = new_content;
                        result_lines = lines;
                        result_inserted = inserted;
                    }
                }
                _ => {} // Ignore Add/Delete for expansion
            }
        }

        (result_content, result_lines, result_inserted)
    };

    // Generate expansion ID
    let expansion_id = format!("exp_{}", Uuid::new_v4().to_string().split('-').next().unwrap());

    // Determine insertion line from updated lines
    let insertion_line = updated_lines.first().copied().unwrap_or(1);

    // Save the updated content
    save_page_content(project_id, page_name, &updated_markdown)?;

    Ok(ExpansionResult {
        expansion_id,
        updated_markdown: updated_markdown.clone(),
        inserted_content,
        insertion_line,
        updated_lines,
    })
}

// ============================================================================
// ANSWER-ONLY MODE (no document modification)
// ============================================================================

/// System prompt for answer-only mode
const ANSWER_SYSTEM_PROMPT: &str = r#"You are an expert tutor helping a student understand learning material. The student has highlighted some text and asked a question about it.

Your task is to provide a clear, concise answer to their question.

## Guidelines
1. Keep answers SHORT - 2-4 sentences max
2. Be direct and educational
3. Match the tone of learning material
4. NO markdown formatting (plain text only)
5. NO emojis
6. Do not reference the document or say things like "as mentioned" - just answer directly"#;

/// Answer a question about selected text without modifying the document
pub async fn answer_question(
    selection: &SelectionRange,
    question: &str,
    _api_key: &str,
) -> Result<String, String> {
    // Create LLM client from config
    let client = LlmClient::from_config()?;

    // Build the prompt
    let user_prompt = format!(
        "Selected text: \"{}\"\n\nQuestion: {}",
        selection.selected_text, question
    );

    let messages = vec![
        LlmClient::system_message(ANSWER_SYSTEM_PROMPT),
        LlmClient::user_message(&user_prompt),
    ];

    let response = client.chat_completion(messages, Some(0.7)).await?;

    Ok(response.trim().to_string())
}

// ============================================================================
// EDITING AGENT (Chat-based project editing)
// ============================================================================

/// System prompt for the editing agent
const EDITING_AGENT_SYSTEM_PROMPT: &str = r##"You are an expert educational content editor. You help users modify, improve, and expand their learning materials through conversation.

## Your Tools

You have access to the following tools to edit the learning material:

### 1. create_file
Creates a new markdown page/chapter.
```json
{
  "tool": "create_file",
  "arguments": {
    "title": "Chapter Title",
    "content": "# Chapter Title\n\nYour markdown content here..."
  }
}
```

### 2. edit_file
Edits an existing page by replacing content. Use this to modify specific sections.
```json
{
  "tool": "edit_file",
  "arguments": {
    "filename": "01-introduction.md",
    "old_content": "Text to find and replace",
    "new_content": "New text to insert"
  }
}
```

### 3. read_file
Reads the content of an existing page.
```json
{
  "tool": "read_file",
  "arguments": {
    "filename": "01-introduction.md"
  }
}
```

### 4. list_files
Lists all pages in the current project.
```json
{
  "tool": "list_files",
  "arguments": {}
}
```

### 5. set_book_info
Updates the title and description of the book.
```json
{
  "tool": "set_book_info",
  "arguments": {
    "title": "New Book Title",
    "description": "Updated description"
  }
}
```

### 6. delete_file
Deletes a page from the project.
```json
{
  "tool": "delete_file",
  "arguments": {
    "filename": "03-unwanted-chapter.md"
  }
}
```

### 7. respond
Use this when you want to respond to the user without making changes, or to ask clarifying questions.
```json
{
  "tool": "respond",
  "arguments": {
    "message": "Your response to the user..."
  }
}
```

## How to Respond

Each response should contain exactly ONE tool call in JSON format. Think step by step about what to do.

Format your response as:
<thinking>
Your reasoning about what to do...
</thinking>

<tool_call>
{
  "tool": "tool_name",
  "arguments": { ... }
}
</tool_call>

## Guidelines

1. **Understand first**: If unsure what the user wants, use read_file or list_files to understand the current state
2. **Be helpful**: Suggest improvements when you see opportunities
3. **Preserve style**: Match the existing writing style and formatting
4. **Confirm big changes**: For major restructuring, explain what you'll do first using respond
5. **NEVER use emojis**: Keep content clean and professional
6. **One action at a time**: Execute one tool per response

## Common Tasks

- "Add more examples to chapter 3" → read_file to see content, then edit_file to add examples
- "Create a new chapter about X" → create_file with comprehensive content
- "What chapters do we have?" → list_files
- "Improve the introduction" → read_file first, then edit_file with improvements
- "Delete the last chapter" → list_files to confirm, then delete_file

IMPORTANT: Always respond with exactly one tool call. Use 'respond' tool when you need to communicate with the user."##;

/// State for the editing agent
pub struct EditingAgentState {
    pub project_id: String,
    pub pages: Vec<PageInfo>,
    pub iteration: u32,
    pub max_iterations: u32,
    pub response_to_user: Option<String>,
}

/// Event payload for chat agent status
#[derive(Debug, Clone, Serialize)]
pub struct ChatAgentEvent {
    pub session_id: String,
    pub status: String,
    pub message: Option<String>,
    pub tool_name: Option<String>,
}

/// Execute delete_file tool
fn execute_delete_file(tool_call: &ToolCall, state: &mut EditingAgentState) -> ToolResult {
    let filename = tool_call.arguments.get("filename")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Load project and remove from page order
    match load_project(&state.project_id) {
        Ok(mut project) => {
            if !project.page_order.contains(&filename.to_string()) {
                return ToolResult {
                    tool_name: "delete_file".to_string(),
                    success: false,
                    output: format!("File '{}' not found in project", filename),
                };
            }

            // Remove from page order
            project.page_order.retain(|p| p != filename);
            project.updated_at = Utc::now();

            // Delete the actual file
            let pages_dir = match crate::services::file_service::get_project_dir(&state.project_id) {
                Ok(dir) => dir.join("pages"),
                Err(e) => return ToolResult {
                    tool_name: "delete_file".to_string(),
                    success: false,
                    output: format!("Failed to get project directory: {}", e),
                },
            };

            let file_path = pages_dir.join(filename);
            if file_path.exists() {
                if let Err(e) = std::fs::remove_file(&file_path) {
                    return ToolResult {
                        tool_name: "delete_file".to_string(),
                        success: false,
                        output: format!("Failed to delete file: {}", e),
                    };
                }
            }

            // Save updated project
            if let Err(e) = crate::services::file_service::save_project(&project) {
                return ToolResult {
                    tool_name: "delete_file".to_string(),
                    success: false,
                    output: format!("Failed to update project: {}", e),
                };
            }

            // Update state
            state.pages.retain(|p| p.filename != filename);

            ToolResult {
                tool_name: "delete_file".to_string(),
                success: true,
                output: format!("Deleted '{}'", filename),
            }
        }
        Err(e) => ToolResult {
            tool_name: "delete_file".to_string(),
            success: false,
            output: format!("Failed to load project: {}", e),
        },
    }
}

/// Execute respond tool (just returns message to user)
fn execute_respond(tool_call: &ToolCall, state: &mut EditingAgentState) -> ToolResult {
    let message = tool_call.arguments.get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("I'm here to help with your learning material.");

    state.response_to_user = Some(message.to_string());

    ToolResult {
        tool_name: "respond".to_string(),
        success: true,
        output: message.to_string(),
    }
}

/// Execute a tool call for the editing agent
fn execute_editing_tool(tool_call: &ToolCall, state: &mut EditingAgentState) -> ToolResult {
    match tool_call.name.as_str() {
        "create_file" => {
            // Reuse existing create_file logic but adapt for EditingAgentState
            let title = tool_call.arguments.get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled");

            let content = tool_call.arguments.get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match add_page_to_project(&state.project_id, title, content) {
                Ok(filename) => {
                    state.pages.push(PageInfo {
                        filename: filename.clone(),
                        title: title.to_string(),
                    });
                    ToolResult {
                        tool_name: "create_file".to_string(),
                        success: true,
                        output: format!("Created page '{}' as {}", title, filename),
                    }
                }
                Err(e) => ToolResult {
                    tool_name: "create_file".to_string(),
                    success: false,
                    output: format!("Failed to create page: {}", e),
                },
            }
        }
        "edit_file" => {
            let filename = tool_call.arguments.get("filename")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let old_content = tool_call.arguments.get("old_content")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let new_content = tool_call.arguments.get("new_content")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let current_content = match load_page_content(&state.project_id, filename) {
                Ok(c) => c,
                Err(e) => return ToolResult {
                    tool_name: "edit_file".to_string(),
                    success: false,
                    output: format!("Failed to read file '{}': {}", filename, e),
                },
            };

            if !current_content.contains(old_content) {
                return ToolResult {
                    tool_name: "edit_file".to_string(),
                    success: false,
                    output: format!("Could not find the specified text in '{}'. Make sure old_content matches exactly.", filename),
                };
            }

            let updated_content = current_content.replacen(old_content, new_content, 1);

            match save_page_content(&state.project_id, filename, &updated_content) {
                Ok(()) => ToolResult {
                    tool_name: "edit_file".to_string(),
                    success: true,
                    output: format!("Successfully edited '{}'", filename),
                },
                Err(e) => ToolResult {
                    tool_name: "edit_file".to_string(),
                    success: false,
                    output: format!("Failed to save edits to '{}': {}", filename, e),
                },
            }
        }
        "read_file" => {
            let filename = tool_call.arguments.get("filename")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match load_page_content(&state.project_id, filename) {
                Ok(content) => ToolResult {
                    tool_name: "read_file".to_string(),
                    success: true,
                    output: format!("Content of '{}':\n\n{}", filename, content),
                },
                Err(e) => ToolResult {
                    tool_name: "read_file".to_string(),
                    success: false,
                    output: format!("Failed to read '{}': {}", filename, e),
                },
            }
        }
        "list_files" => {
            if state.pages.is_empty() {
                // Refresh pages from project
                if let Ok(project) = load_project(&state.project_id) {
                    state.pages = project.page_order.iter()
                        .map(|f| PageInfo {
                            filename: f.clone(),
                            title: f.clone(),
                        })
                        .collect();
                }
            }

            if state.pages.is_empty() {
                return ToolResult {
                    tool_name: "list_files".to_string(),
                    success: true,
                    output: "No pages in this project yet.".to_string(),
                };
            }

            let file_list: Vec<String> = state.pages.iter()
                .map(|p| format!("- {}", p.filename))
                .collect();

            ToolResult {
                tool_name: "list_files".to_string(),
                success: true,
                output: format!("Pages in project:\n{}", file_list.join("\n")),
            }
        }
        "set_book_info" => {
            let title = tool_call.arguments.get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled");

            let description = tool_call.arguments.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match load_project(&state.project_id) {
                Ok(mut project) => {
                    project.title = title.to_string();
                    project.description = description.to_string();
                    project.updated_at = Utc::now();

                    if let Err(e) = crate::services::file_service::save_project(&project) {
                        return ToolResult {
                            tool_name: "set_book_info".to_string(),
                            success: false,
                            output: format!("Failed to save book info: {}", e),
                        };
                    }
                    ToolResult {
                        tool_name: "set_book_info".to_string(),
                        success: true,
                        output: format!("Updated book - Title: '{}', Description: '{}'", title, description),
                    }
                }
                Err(e) => ToolResult {
                    tool_name: "set_book_info".to_string(),
                    success: false,
                    output: format!("Failed to load project: {}", e),
                },
            }
        }
        "delete_file" => execute_delete_file(tool_call, state),
        "respond" => execute_respond(tool_call, state),
        _ => ToolResult {
            tool_name: tool_call.name.clone(),
            success: false,
            output: format!("Unknown tool: {}", tool_call.name),
        },
    }
}

/// Result from chat agent
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatAgentResult {
    pub response: String,
    pub tool_used: Option<String>,
    pub pages_changed: bool,
}

/// Run the editing agent for a chat message
pub async fn run_editing_agent(
    project_id: &str,
    session_id: &str,
    user_message: &str,
    _api_key: &str,
    app: AppHandle,
) -> Result<ChatAgentResult, String> {
    // Create LLM client from config
    let client = LlmClient::from_config()?;

    // Load the session to get history
    let mut session = load_chat_session(project_id, session_id)?;

    // Add user message to session
    session.messages.push(ChatMessage {
        role: "user".to_string(),
        content: user_message.to_string(),
        timestamp: Utc::now(),
    });

    // Update title from first user message if still "New Chat"
    if session.title == "New Chat" {
        let title = user_message.chars().take(50).collect::<String>();
        session.title = if user_message.len() > 50 {
            format!("{}...", title)
        } else {
            title
        };
    }

    // Load project info
    let project = load_project(project_id)?;

    // Initialize agent state
    let mut state = EditingAgentState {
        project_id: project_id.to_string(),
        pages: project.page_order.iter()
            .map(|f| PageInfo {
                filename: f.clone(),
                title: f.clone(),
            })
            .collect(),
        iteration: 0,
        max_iterations: 10, // Limit iterations for chat
        response_to_user: None,
    };

    // Build messages for LLM from session history
    let mut messages = vec![
        LlmClient::system_message(EDITING_AGENT_SYSTEM_PROMPT),
    ];

    // Add conversation history (last 20 messages to avoid token limits)
    let history_start = if session.messages.len() > 20 {
        session.messages.len() - 20
    } else {
        0
    };

    for msg in &session.messages[history_start..] {
        if msg.role == "user" {
            messages.push(LlmClient::user_message(&msg.content));
        } else {
            messages.push(LlmClient::assistant_message(&msg.content));
        }
    }

    let mut final_response = String::new();
    let mut tool_used: Option<String> = None;
    let mut pages_changed = false;

    // Emit starting status
    let _ = app.emit("chat-agent-status", ChatAgentEvent {
        session_id: session_id.to_string(),
        status: "thinking".to_string(),
        message: None,
        tool_name: None,
    });

    // Agent loop - continue until we get a response to user or hit limit
    while state.iteration < state.max_iterations && state.response_to_user.is_none() {
        state.iteration += 1;

        // Call the LLM
        let response = client.chat_completion(messages.clone(), Some(0.7)).await?;

        // Add assistant response to messages
        messages.push(LlmClient::assistant_message(&response));

        // Parse tool call
        let tool_call = match parse_tool_call(&response) {
            Ok(tc) => tc,
            Err(_) => {
                // If no tool call found, treat the response as a direct message
                final_response = response.clone();
                break;
            }
        };

        // Emit tool execution status
        let _ = app.emit("chat-agent-status", ChatAgentEvent {
            session_id: session_id.to_string(),
            status: "executing".to_string(),
            message: Some(format!("Using {}", tool_call.name)),
            tool_name: Some(tool_call.name.clone()),
        });

        // Track if pages might have changed
        if matches!(tool_call.name.as_str(), "create_file" | "edit_file" | "delete_file") {
            pages_changed = true;
        }

        tool_used = Some(tool_call.name.clone());

        // Execute the tool
        let result = execute_editing_tool(&tool_call, &mut state);

        // If it was a respond tool, we're done
        if tool_call.name == "respond" {
            final_response = result.output.clone();
            break;
        }

        // Add tool result to messages for next iteration
        let result_msg = if result.success {
            format!("Tool '{}' executed successfully:\n{}", result.tool_name, result.output)
        } else {
            format!("Tool '{}' failed:\n{}", result.tool_name, result.output)
        };
        messages.push(LlmClient::user_message(&result_msg));
    }

    // If we have a response from respond tool, use that
    if let Some(resp) = state.response_to_user {
        final_response = resp;
    }

    // If still no response, ask agent to summarize what it did
    if final_response.is_empty() {
        messages.push(LlmClient::user_message(
            "Now use the respond tool to tell the user what you did."
        ));

        if let Ok(summary_response) = client.chat_completion(messages, Some(0.7)).await {
            if let Ok(tool_call) = parse_tool_call(&summary_response) {
                if tool_call.name == "respond" {
                    if let Some(msg) = tool_call.arguments.get("message").and_then(|v| v.as_str()) {
                        final_response = msg.to_string();
                    }
                }
            }
            if final_response.is_empty() {
                final_response = "I've made the requested changes to your learning material.".to_string();
            }
        } else {
            final_response = "I've made the requested changes to your learning material.".to_string();
        }
    }

    // Add assistant response to session
    session.messages.push(ChatMessage {
        role: "assistant".to_string(),
        content: final_response.clone(),
        timestamp: Utc::now(),
    });
    session.updated_at = Utc::now();

    // Save session
    save_chat_session(&session)?;

    // Emit completion status
    let _ = app.emit("chat-agent-status", ChatAgentEvent {
        session_id: session_id.to_string(),
        status: "complete".to_string(),
        message: None,
        tool_name: None,
    });

    Ok(ChatAgentResult {
        response: final_response,
        tool_used,
        pages_changed,
    })
}
