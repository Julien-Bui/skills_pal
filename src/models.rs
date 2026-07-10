use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Skill {
    pub id: Option<i64>,
    pub name: String,
    pub description: String,
    pub github_url: String,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Report {
    pub id: Option<i64>,
    pub skill_id: i64,
    pub file_path: String,
    pub line_number: Option<i32>,
    pub message: String,
    pub severity: String,
    pub details: serde_json::Value,
}
