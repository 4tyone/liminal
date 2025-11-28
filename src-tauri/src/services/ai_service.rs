use crate::models::{ProjectMeta, SelectionRange, ExpansionResult};
use crate::services::file_service::{
    create_new_project, add_page_to_project, load_page_content,
    save_page_content, load_project
};
use crate::services::llm_client::LlmClient;
use crate::services::config_service;
use uuid::Uuid;
use regex::Regex;

/// System prompt for the learning content generation agent
const GENERATION_SYSTEM_PROMPT: &str = r#"You are an expert educational content creator. Your task is to generate comprehensive, book-like learning material on any topic.

## Your Capabilities
- Create well-structured educational content with clear chapters/sections
- Explain complex concepts in an accessible, engaging manner
- Use examples, analogies, and practical applications
- Structure content progressively from fundamentals to advanced topics

## Output Format
You will generate content in a special patch format. Each chapter/page should be created as a separate markdown file.

### Patch Format
Use this EXACT format to create files:

*** Begin Patch
*** Add File: pages/01-introduction.md
+# Chapter Title
+
+Content goes here...
+Each line must start with + for new content
*** Add File: pages/02-fundamentals.md
+# Another Chapter
+
+More content...
*** End Patch

## Guidelines
1. Each chapter should be self-contained but build on previous ones
2. Use markdown formatting (headers, bold, lists, code blocks)
3. Include practical examples and exercises where appropriate
4. Write in an engaging, conversational yet professional tone
5. Structure like a real book with clear progression
6. NEVER use emojis in any content
7. Let the depth level and topic complexity determine how many chapters you need
    - it's ok to have many chapters or or just a few depending on the topic and depth and the prompt

## Depth Levels
- beginner: Cover the basics only. High-level overview without going into details.
- intermediate: Go deeper into concepts. Include examples and explain the "why" behind things.
- advanced: Comprehensive coverage. Include technical details, edge cases, best practices, and advanced patterns.

IMPORTANT: Always output ONLY the patch format. No other text before or after. NEVER use emojis."#;

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

/// Generate learning material for a topic using the AI agent
pub async fn generate_learning_material(
    topic: &str,
    depth: &str,
    api_key: &str,
) -> Result<ProjectMeta, String> {
    // Get configuration
    let config = config_service::get_full_config()?;
    let base_url = config.base_url.unwrap_or_default();
    let model = config.model.unwrap_or_else(|| "gpt-4o-mini".to_string());

    let client = LlmClient::new(&base_url, api_key, &model);

    // Create the project first
    let project = create_new_project(topic, &format!("Learning {} at {} level", topic, depth))?;

    // Build the generation prompt
    let user_prompt = format!(
        "Topic: {}\nDepth: {}\n\nGenerate the content now as a patch.",
        topic, depth
    );

    // Call the LLM
    let messages = vec![
        LlmClient::system_message(GENERATION_SYSTEM_PROMPT),
        LlmClient::user_message(&user_prompt),
    ];

    let response = client.chat_completion(messages, Some(0.7), Some(8000)).await?;

    // Parse the patch from the response
    let operations = parse_patch(&response)?;

    // Track the first page title to use as project title
    let mut first_page_title: Option<String> = None;

    if operations.is_empty() {
        // Fallback: create a basic introduction if no patch was generated
        let intro_content = format!(
            "# Introduction to {}\n\n\
            Welcome to your learning journey about **{}**!\n\n\
            ## Overview\n\n\
            This learning material was generated at the **{}** level.\n\n\
            ## Getting Started\n\n\
            {}\n",
            topic, topic, depth, response
        );
        add_page_to_project(&project.id, "Introduction", &intro_content)?;
    } else {
        // Apply the patch operations
        for op in operations {
            match op {
                PatchOperation::AddFile { path, content } => {
                    // Extract title from the path or content
                    let title = extract_title_from_content(&content)
                        .unwrap_or_else(|| path_to_title(&path));

                    // Capture the first page title for the project
                    if first_page_title.is_none() {
                        first_page_title = Some(title.clone());
                    }

                    add_page_to_project(&project.id, &title, &content)?;
                }
                _ => {} // Ignore update and delete for generation
            }
        }
    }

    // Update project title with the first page's title
    if let Some(title) = first_page_title {
        let mut updated_project = load_project(&project.id)?;
        updated_project.title = title;
        crate::services::file_service::save_project(&updated_project)?;
    }

    // Reload to get updated page order
    load_project(&project.id)
}

/// Extract a title from markdown content (first H1 header)
fn extract_title_from_content(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.starts_with("# ") {
            return Some(line[2..].trim().to_string());
        }
    }
    None
}

/// Convert a file path to a title
fn path_to_title(path: &str) -> String {
    let filename = path.split('/').last().unwrap_or(path);
    let name = filename.strip_suffix(".md").unwrap_or(filename);
    // Remove leading numbers like "01-"
    let re = Regex::new(r"^\d+-").unwrap();
    let clean_name = re.replace(name, "").to_string();
    // Convert to title case
    clean_name
        .split('-')
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Expand a selection with AI-generated content using Codex-style patches
pub async fn expand_selection_with_ai(
    project_id: &str,
    page_name: &str,
    selection: &SelectionRange,
    question: &str,
    api_key: &str,
) -> Result<ExpansionResult, String> {
    // Get configuration
    let config = config_service::get_full_config()?;
    let base_url = config.base_url.unwrap_or_default();
    let model = config.model.unwrap_or_else(|| "gpt-4o-mini".to_string());

    let client = LlmClient::new(&base_url, api_key, &model);

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

    let response = client.chat_completion(messages, Some(0.7), Some(2000)).await?;

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

// Old helper functions removed - now using Codex-style patch application
