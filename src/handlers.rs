use axum::{
    http::StatusCode,
    response::IntoResponse,
};

pub async fn get_skills() -> impl IntoResponse
{
    StatusCode::OK
}

pub async fn add_skill() -> impl IntoResponse
{
    StatusCode::CREATED
}

pub async fn delete_skill() -> impl IntoResponse
{
    StatusCode::NO_CONTENT
}

pub async fn save_api_key() -> impl IntoResponse
{
    StatusCode::OK
}
