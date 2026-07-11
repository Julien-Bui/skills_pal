mod db;
mod scraper;
mod dashboard;

use axum::{routing::get, Router, Json};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Serialize, Clone, sqlx::FromRow)]
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

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        println!("⚠️  ATTENTION: DATABASE_URL non définie. Utilisation du fallback local.");
        "postgres://postgres:postgres@localhost/skillspal".to_string()
    });
    
    let pool = db::init_db(&db_url).await?;
    
    // Remplir le cache en mémoire au démarrage avec log d'erreur
    let initial_skills = match db::fetch_all_skills(&pool).await {
        Ok(skills) => {
            println!("Cache initialisé avec {} skills.", skills.len());
            skills
        },
        Err(e) => {
            eprintln!("⚠️  ERREUR CRITIQUE: Impossible de charger les skills depuis la BDD au démarrage: {}", e);
            Vec::new()
        }
    };
    
    let state = Arc::new(AppState { 
        db: pool,
        cached_skills: tokio::sync::RwLock::new(initial_skills),
    });

    scraper::start_background_scraper(state.clone());

    let app = Router::new()
        .route("/", get(dashboard::serve_dashboard))
        .route("/api/skills", get(get_skills))
        .layer(tower::limit::GlobalConcurrencyLimitLayer::new(100))
        .with_state(state.clone());

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    
    println!("Serveur lancé sur http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    // Graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    println!("Serveur arrêté proprement. Fermeture des connexions DB...");
    state.db.close().await;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Echec de l'installation du handler Ctrl+C");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Echec de l'installation du handler SIGTERM")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    
    println!("\nSignal d'arrêt reçu, lancement du graceful shutdown...");
}

async fn get_skills(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Json<Vec<SkillResponse>> {
    // 0 milliseconde de latence, on lit directement dans la RAM !
    let skills = state.cached_skills.read().await;
    Json(skills.clone())
}
