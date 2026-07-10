use sqlx::{PgPool, postgres::PgPoolOptions, Row};
use crate::SkillResponse;

pub async fn init_db(db_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;

    // Create table if not exists
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS remote_skills (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            description TEXT NOT NULL,
            github_url VARCHAR(255) UNIQUE NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );"
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

pub async fn fetch_all_skills(pool: &PgPool) -> Result<Vec<SkillResponse>, sqlx::Error> {
    let records = sqlx::query(
        "SELECT id, name, description, github_url FROM remote_skills ORDER BY id DESC"
    )
    .fetch_all(pool)
    .await?;

    let mut skills = Vec::new();
    for r in records {
        skills.push(SkillResponse {
            id: r.get("id"),
            name: r.get("name"),
            description: r.get("description"),
            github_url: r.get("github_url"),
        });
    }

    Ok(skills)
}

pub async fn insert_skill(pool: &PgPool, name: &str, description: &str, github_url: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO remote_skills (name, description, github_url) 
         VALUES ($1, $2, $3) 
         ON CONFLICT (github_url) DO NOTHING"
    )
    .bind(name)
    .bind(description)
    .bind(github_url)
    .execute(pool)
    .await?;

    Ok(())
}
