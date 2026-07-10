use crate::config;
use reqwest::Client;
use std::fs;
use serde_json::json;
use serde::Deserialize;

#[derive(Deserialize)]
struct RemoteSkill {
    name: String,
    description: String,
    github_url: String,
}

pub async fn run_recommendation() -> Result<(), String> {
    let conf = config::get_config()?;
    let client = Client::builder()
        .user_agent("Skills-Pal-CLI/1.0")
        .build()
        .map_err(|e| e.to_string())?;

    println!("Contact du registre Railway pour récupérer la liste des skills...");
    let mut remote_skills = Vec::new();
    
    let mut registry = "https://skillspal-production-0511.up.railway.app".to_string();
    if let Some(url) = &conf.registry_url {
        if !url.is_empty() {
            registry = url.clone();
        }
    }

    let registry_url = format!("{}/api/skills", registry.trim_end_matches('/'));
    match client.get(&registry_url).send().await {
        Ok(res) => {
            if res.status().is_success() {
                if let Ok(skills) = res.json::<Vec<RemoteSkill>>().await {
                    remote_skills = skills;
                    println!("✅ {} skills récupérés depuis le registre !", remote_skills.len());
                }
            } else {
                println!("⚠️ Erreur du registre: {}", res.status());
            }
        },
        Err(e) => println!("⚠️ Impossible de contacter le registre: {}", e),
    }

    println!("Analyse dynamique du projet (Agonostique au langage)...");
    
    let mut extensions = std::collections::HashMap::new();
    
    // On scanne les 200 premiers fichiers pertinents pour deviner le langage
    for entry in walkdir::WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
        if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
            // Ignorer les extensions inutiles
            if !["git", "json", "md", "txt", "lock", "toml", "yml", "yaml", "png", "svg", "jpg"].contains(&ext) {
                *extensions.entry(ext.to_string()).or_insert(0) += 1;
            }
        }
    }

    let mut sorted_exts: Vec<_> = extensions.into_iter().collect();
    sorted_exts.sort_by(|a, b| b.1.cmp(&a.1)); // Trier par fréquence d'apparition

    let top_extensions: Vec<String> = sorted_exts.into_iter().take(5).map(|(ext, count)| format!(".{} ({})", ext, count)).collect();

    let stack = if top_extensions.is_empty() {
        "Projet vide ou langages non identifiés".to_string()
    } else {
        format!("Extensions principales détectées : {}", top_extensions.join(", "))
    };

    println!("Stack détectée : {}", stack);
    println!("Contact de l'IA ({}) pour obtenir des recommandations...", conf.provider.to_uppercase());

    let mut prompt_system = "Tu es l'IA de Skills Pal. Propose 2 ou 3 outils/plugins pertinents pour analyser la dette technique du projet basé sur la stack fournie. Réponds de manière concise.".to_string();
    
    if !remote_skills.is_empty() {
        prompt_system.push_str("\n\nVoici la base de données de plugins existants. Tu DOIS prioriser les recommandations de CES plugins si tu penses qu'ils sont pertinents (fournis leur URL) :\n");
        for skill in remote_skills.iter().take(20) {
            prompt_system.push_str(&format!("- {} ({}): {}\n", skill.name, skill.github_url, skill.description));
        }
    }

    let prompt_user = format!("Le projet utilise : {}", stack);
    
    let reply = match conf.provider.to_lowercase().as_str() {
        "openai" | "openai_compatible" => {
            let api_key = conf.api_key.ok_or("Clé API manquante")?;
            if api_key == "VOTRE_CLE_ICI" || api_key.is_empty() {
                return Err("La clé API n'a pas été configurée.".to_string());
            }
            
            let mut endpoint = "https://api.openai.com/v1/chat/completions".to_string();
            if let Some(url) = &conf.base_url {
                if !url.is_empty() {
                    endpoint = url.clone();
                }
            }
            
            let res = client.post(&endpoint)
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&json!({
                    "model": conf.model,
                    "messages": [
                        { "role": "system", "content": prompt_system },
                        { "role": "user", "content": prompt_user }
                    ]
                }))
                .send().await.map_err(|e| e.to_string())?;
                
            if !res.status().is_success() {
                let status = res.status();
                let err_text = res.text().await.unwrap_or_default();
                return Err(format!("Erreur API {}: {} - Détails: {}", conf.provider, status, err_text));
            }
            
            let data: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
            data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string()
        },
        "anthropic" => {
            let api_key = conf.api_key.ok_or("Clé API manquante")?;
            
            let res = client.post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .json(&json!({
                    "model": conf.model,
                    "max_tokens": 500,
                    "system": prompt_system,
                    "messages": [
                        { "role": "user", "content": prompt_user }
                    ]
                }))
                .send().await.map_err(|e| e.to_string())?;
                
            if !res.status().is_success() {
                return Err(format!("Erreur API Anthropic: {}", res.status()));
            }
            
            let data: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
            data["content"][0]["text"].as_str().unwrap_or("").to_string()
        },
        "ollama" => {
            let mut url = "http://localhost:11434/api/chat".to_string();
            if let Some(base) = &conf.base_url {
                if !base.is_empty() {
                    url = format!("{}/api/chat", base.trim_end_matches('/'));
                }
            }
            
            let res = client.post(&url)
                .json(&json!({
                    "model": conf.model,
                    "messages": [
                        { "role": "system", "content": prompt_system },
                        { "role": "user", "content": prompt_user }
                    ],
                    "stream": false
                }))
                .send().await.map_err(|e| e.to_string())?;
                
            if !res.status().is_success() {
                return Err(format!("Erreur API Ollama: {}", res.status()));
            }
            
            let data: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
            data["message"]["content"].as_str().unwrap_or("").to_string()
        },
        _ => return Err(format!("Fournisseur IA non supporté: {}. Choisissez parmi: openai, openai_compatible, anthropic, ollama.", conf.provider)),
    };

    if reply.is_empty() {
        println!("L'IA a répondu avec un format inattendu.");
    } else {
        println!("\n💡 Recommandations de l'IA :\n{}", reply);
    }

    Ok(())
}
