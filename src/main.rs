

use axum::{routing::{get, post, delete}, Router};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tower_http::cors::CorsLayer;

mod config;
mod models;
mod database;
mod errors;
mod handlers;
mod analyzer;
mod downloader;

#[tokio::main]
async fn main() 
{
    println!("Démarrage de Skills Pal...");

    // Configuration de l'API (Surface level)
    let app = Router::new()
        .route("/api/skills", get(handlers::get_skills).post(handlers::add_skill))
        .route("/api/skills/:id", delete(handlers::delete_skill))
        .route("/api/settings/keys", post(handlers::save_api_key))
        .route("/api/analyze", post(handlers::analyze_project))
        .fallback_service(ServeDir::new("static")) // Sert le dossier frontend
        .layer(CorsLayer::permissive()); // Permet à l'IDE et au navigateur d'appeler l'API

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Serveur API en écoute sur http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
