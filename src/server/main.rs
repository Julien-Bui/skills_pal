mod db;
mod scraper;

use axum::{routing::get, Router, Json};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Serialize, Clone)]
pub struct SkillResponse {
    id: i32,
    name: String,
    description: String,
    github_url: String,
}

pub struct AppState {
    pub db: sqlx::PgPool,
    pub cached_skills: tokio::sync::RwLock<Vec<SkillResponse>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Démarrage du serveur Railway Skills Pal...");

    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/skillspal".to_string());
    
    let pool = db::init_db(&db_url).await?;
    
    // Remplir le cache en mémoire au démarrage
    let initial_skills = db::fetch_all_skills(&pool).await.unwrap_or_default();
    println!("Cache initialisé avec {} skills.", initial_skills.len());
    
    let state = Arc::new(AppState { 
        db: pool,
        cached_skills: tokio::sync::RwLock::new(initial_skills),
    });

    scraper::start_background_scraper(state.clone());

    let app = Router::new()
        .route("/api/skills", get(get_skills))
        .layer(tower::limit::GlobalConcurrencyLimitLayer::new(100))
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    
    println!("Serveur lancé sur http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_skills(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Json<Vec<SkillResponse>> {
    // 0 milliseconde de latence, on lit directement dans la RAM !
    let skills = state.cached_skills.read().await;
    Json(skills.clone())
}
