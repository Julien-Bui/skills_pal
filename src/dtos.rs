use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AddSkillRequest
{
    pub github_url: String,
}

#[derive(Deserialize)]
pub struct SaveApiKeyRequest
{
    pub service_name: String,
    pub key_value: String,
}

#[derive(Deserialize)]
pub struct AnalyzeProjectRequest
{
    pub project_path: String,
}
