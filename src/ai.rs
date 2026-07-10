use crate::config;
use reqwest::Client;
use std::fs;
use serde_json::json;

pub async fn run_recommendation() -> Result<(), String> {
    let conf = config::get_config()?;

    println!("Analyse du contexte du projet local...");
    
    // Détection basique du contexte (fichiers existants)
    let mut context = Vec::new();
    if fs::metadata("Cargo.toml").is_ok() {
        context.push("Rust (Cargo)");
    }
    if fs::metadata("package.json").is_ok() {
        context.push("Node.js (NPM)");
    }
    if fs::metadata("requirements.txt").is_ok() {
        context.push("Python");
    }

    let stack = if context.is_empty() {
        "Stack inconnue".to_string()
    } else {
        context.join(", ")
    };

    println!("Stack détectée : {}", stack);
    println!("Contact de l'IA ({}) pour obtenir des recommandations...", conf.provider.to_uppercase());

    let client = Client::new();
    let prompt_system = "Tu es l'IA de Skills Pal. Propose 2 ou 3 outils/plugins pertinents pour analyser la dette technique du projet basé sur la stack fournie. Réponds de manière concise.";
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
