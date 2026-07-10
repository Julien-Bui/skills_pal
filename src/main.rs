

use axum::{routing::{get, post, delete}, Router};
use std::net::SocketAddr;

mod models;
mod database;
mod handlers;
mod analyzer;

#[tokio::main]
async fn main() 
{
    println!("Démarrage de Skills Pal...");

    // Configuration de l'API (Surface level)
    let app = Router::new()
        .route("/api/skills", get(handlers::get_skills).post(handlers::add_skill))
        .route("/api/skills/:id", delete(handlers::delete_skill))
        .route("/api/settings/keys", post(handlers::save_api_key));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Serveur API en écoute sur http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
