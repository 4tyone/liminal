use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMeta {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub page_order: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectListItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub page_count: usize,
    pub updated_at: DateTime<Utc>,
}

impl From<ProjectMeta> for ProjectListItem {
    fn from(meta: ProjectMeta) -> Self {
        Self {
            id: meta.id.clone(),
            title: meta.title,
            description: meta.description,
            page_count: meta.page_order.len(),
            updated_at: meta.updated_at,
        }
    }
}
