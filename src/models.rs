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
    pub timestamp: String,
    pub grade: String,
    pub total_issues: i32,
    pub score_details: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiKey {
    pub id: Option<i64>,
    pub service_name: String,
    pub key_value: String,
}
