use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionRange {
    #[serde(default)]
    pub start_line: usize,
    #[serde(default)]
    pub end_line: usize,
    pub selected_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpansionResult {
    pub expansion_id: String,
    pub updated_markdown: String,
    pub inserted_content: String,
    pub insertion_line: usize,
    pub updated_lines: Vec<usize>,
}
