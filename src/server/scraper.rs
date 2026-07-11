use std::sync::Arc;
use tokio::time::{sleep, Duration};
use reqwest::Client;
use crate::AppState;
use serde::Deserialize;

#[derive(Deserialize)]
struct GithubContentItem {
    name: String,
    #[serde(rename = "type")]
    item_type: String,
}

pub fn start_background_scraper(state: Arc<AppState>) {
    tokio::spawn(async move {
        println!("Background scraper lancé ! Il tournera toutes les 12 heures.");
        // Délai de grâce initial de 60s pour éviter le spam API GitHub en cas de crash loop
        sleep(Duration::from_secs(60)).await;
        
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
    let mut url = "https://api.github.com/repos/Julien-Bui/skills-registry/contents/skills".to_string();
    
    loop {
        let res = client.get(&url).send().await?;
        if !res.status().is_success() {
            eprintln!("Github a retourné une erreur lors de la lecture du dossier: {}", res.status());
            break;
        }

        let next_url = get_next_link(res.headers());
        let items: Vec<GithubContentItem> = res.json().await?;
        
        for item in items {
            if item.item_type == "dir" {
                let skill_name = item.name.clone();
                let raw_url = format!("https://raw.githubusercontent.com/Julien-Bui/skills-registry/main/skills/{}/SKILL.md", skill_name);
                
                match client.get(&raw_url).send().await {
                    Ok(raw_res) => {
                        if raw_res.status().is_success() {
                            let content = raw_res.text().await.unwrap_or_default();
                            if let Some((name, desc)) = parse_skill_frontmatter(&content) {
                                let github_url = format!("https://github.com/Julien-Bui/skills-registry/tree/main/skills/{}", skill_name);
                                println!("Nouveau skill détecté : {} ({})", name, github_url);
                                let _ = crate::db::insert_skill(&state.db, &name, &desc, &github_url).await;
                            }
                        }
                    },
                    Err(e) => eprintln!("Erreur lors du téléchargement de {} : {}", skill_name, e),
                }
            }
        }

        if let Some(next) = next_url {
            url = next;
        } else {
            break;
        }
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

fn parse_skill_frontmatter(content: &str) -> Option<(String, String)> {
    if !content.starts_with("---") {
        return None;
    }
    
    let parts: Vec<&str> = content.split("---").collect();
    if parts.len() < 3 {
        return None;
    }
    
    let frontmatter = parts[1];
    let mut name = String::new();
    let mut desc = String::new();
    
    for line in frontmatter.lines() {
        let line = line.trim();
        if line.starts_with("name:") {
            name = line.trim_start_matches("name:").trim()
                       .trim_matches(|c| c == '"' || c == '\'')
                       .to_string();
        } else if line.starts_with("description:") {
            desc = line.trim_start_matches("description:").trim()
                       .trim_matches(|c| c == '"' || c == '\'')
                       .to_string();
        }
    }
    
    if !name.is_empty() && !desc.is_empty() {
        Some((name, desc))
    } else {
        None
    }
}

fn get_next_link(headers: &reqwest::header::HeaderMap) -> Option<String> {
    if let Some(link_header) = headers.get(reqwest::header::LINK) {
        if let Ok(link_str) = link_header.to_str() {
            for link in link_str.split(',') {
                if link.contains("rel=\"next\"") {
                    if let Some(start) = link.find('<') {
                        if let Some(end) = link.find('>') {
                            return Some(link[start + 1..end].to_string());
                        }
                    }
                }
            }
        }
    }
    None
}
