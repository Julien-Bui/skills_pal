use std::sync::Arc;
use tokio::time::{sleep, Duration};
use reqwest::Client;
use crate::AppState;
use serde::Deserialize;

#[derive(Deserialize)]
struct GithubSearchResponse {
    items: Vec<GithubRepo>,
}

#[derive(Deserialize)]
struct GithubRepo {
    name: String,
    description: Option<String>,
    html_url: String,
}

pub fn start_background_scraper(state: Arc<AppState>) {
    tokio::spawn(async move {
        println!("Background scraper lancé ! Il tournera toutes les 12 heures.");
        let client = Client::builder()
            .user_agent("Skills-Pal-Scraper/1.0")
            .build()
            .unwrap();

        loop {
            println!("Lancement du scraping Github...");
            if let Err(e) = scrape_github(&client, &state).await {
                eprintln!("Erreur lors du scraping: {}", e);
            }
            // Scrape toutes les 12 heures
            sleep(Duration::from_secs(12 * 3600)).await;
        }
    });
}

async fn scrape_github(client: &Client, state: &Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    let query = "topic:skills-pal";
    let url = format!("https://api.github.com/search/repositories?q={}", query);
    
    let res = client.get(&url).send().await?;
    if !res.status().is_success() {
        eprintln!("Github a retourné une erreur: {}", res.status());
        return Ok(());
    }

    let search_res: GithubSearchResponse = res.json().await?;
    
    for repo in search_res.items {
        let desc = repo.description.unwrap_or_else(|| "Aucune description".to_string());
        println!("Nouveau skill détecté : {} ({})", repo.name, repo.html_url);
        
        crate::db::insert_skill(&state.db, &repo.name, &desc, &repo.html_url).await?;
    }

    // Mettre à jour le cache en mémoire après le scraping
    if let Ok(new_skills) = crate::db::fetch_all_skills(&state.db).await {
        let mut cache = state.cached_skills.write().await;
        *cache = new_skills;
        println!("Cache mis à jour en mémoire ({} skills).", cache.len());
    }

    println!("Scraping Github terminé !");
    Ok(())
}
