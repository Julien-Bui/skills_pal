use crate::config;
use reqwest::Client;
use serde_json::json;
use serde::Deserialize;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Deserialize)]
struct RemoteSkill {
    name: String,
    description: String,
    github_url: String,
}

pub async fn run_recommendation(verbose: bool) -> Result<(), String> {
    let conf = config::get_config()?;
    let client = Client::builder()
        .user_agent("Skills-Pal-CLI/1.0")
        .build()
        .map_err(|e| e.to_string())?;

    // --- Spinner : Contact du registre ---
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✔"]),
    );
    spinner.set_message("Contact du registre Railway...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let mut remote_skills = Vec::new();

    let mut registry = "https://skillspal-production-0511.up.railway.app".to_string();
    if let Some(url) = &conf.registry_url {
        if !url.is_empty() {
            registry = url.clone();
        }
    }

    let registry_url = format!("{}/api/skills", registry.trim_end_matches('/'));
    
    if verbose {
        spinner.suspend(|| {
            println!("  {} URL du registre : {}", "→".dimmed(), registry_url.dimmed());
        });
    }

    match client.get(&registry_url).send().await {
        Ok(res) => {
            if res.status().is_success() {
                if let Ok(skills) = res.json::<Vec<RemoteSkill>>().await {
                    remote_skills = skills;
                    spinner.finish_with_message(
                        format!("{} {} skills récupérés depuis le registre !", "✔".green(), remote_skills.len().to_string().cyan())
                    );
                }
            } else {
                spinner.finish_with_message(format!("{} Erreur du registre: {}", "⚠".yellow(), res.status()));
            }
        },
        Err(e) => {
            spinner.finish_with_message(format!("{} Impossible de contacter le registre: {}", "⚠".yellow(), e));
        },
    }

    // --- Analyse du projet ---
    println!("\n{} Analyse dynamique du projet...", "🔍".to_string());

    let mut extensions = std::collections::HashMap::new();

    for entry in walkdir::WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
        if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
            if !["git", "json", "md", "txt", "lock", "toml", "yml", "yaml", "png", "svg", "jpg"].contains(&ext) {
                *extensions.entry(ext.to_string()).or_insert(0) += 1;
            }
        }
    }

    let mut sorted_exts: Vec<_> = extensions.into_iter().collect();
    sorted_exts.sort_by(|a, b| b.1.cmp(&a.1));

    let top_extensions: Vec<String> = sorted_exts.into_iter().take(5).map(|(ext, count)| format!(".{} ({})", ext, count)).collect();

    let stack = if top_extensions.is_empty() {
        "Projet vide ou langages non identifiés".to_string()
    } else {
        format!("Extensions principales : {}", top_extensions.join(", "))
    };

    println!("  {} {}", "Stack détectée :".bold(), stack.cyan());

    // --- Spinner : Contact de l'IA ---
    let ai_spinner = ProgressBar::new_spinner();
    ai_spinner.set_style(
        ProgressStyle::with_template("{spinner:.magenta} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✔"]),
    );
    ai_spinner.set_message(format!("Contact de l'IA ({})...", conf.provider.to_uppercase().magenta()));
    ai_spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    if verbose {
        ai_spinner.suspend(|| {
            println!("  {} Modèle : {}", "→".dimmed(), conf.model.dimmed());
        });
    }

    let mut prompt_system = "Tu es l'IA de Skills Pal. Propose 2 ou 3 outils, plugins ou compétences d'ingénierie pertinents pour améliorer ce projet dans son ensemble (ex: architecture, CI/CD, productivité, qualité de code) en te basant sur la stack fournie. Ne te limite pas à la dette technique. Réponds de manière concise.".to_string();

    if !remote_skills.is_empty() {
        prompt_system.push_str("\n\nVoici la base de données de plugins existants. Tu DOIS prioriser les recommandations de CES plugins si tu penses qu'ils sont pertinents (fournis leur URL) :\n");
        for skill in remote_skills.iter().take(20) {
            prompt_system.push_str(&format!("- {} ({}): {}\n", skill.name, skill.github_url, skill.description));
        }
    }

    let prompt_user = format!("Le projet utilise : {}", stack);

    if verbose {
        ai_spinner.suspend(|| {
            println!("  {} Prompt système : {} caractères", "→".dimmed(), prompt_system.len().to_string().dimmed());
        });
    }

    let reply = match conf.provider.to_lowercase().as_str() {
        "openai" | "openai_compatible" => {
            let api_key = conf.api_key.ok_or(format!("{} Clé API manquante dans la configuration.", "❌".red()))?;
            if api_key == "VOTRE_CLE_MISTRAL_ICI" || api_key == "VOTRE_CLE_ICI" || api_key.is_empty() {
                ai_spinner.finish_and_clear();
                return Err(format!("{} La clé API n'a pas été configurée.\n  {} Éditez votre fichier de configuration ou relancez : skills_pal init --api-key <VOTRE_CLE>", "❌", "→".dimmed()));
            }

            let mut endpoint = "https://api.openai.com/v1/chat/completions".to_string();
            if let Some(url) = &conf.base_url {
                if !url.is_empty() {
                    endpoint = url.clone();
                }
            }

            if verbose {
                ai_spinner.suspend(|| {
                    println!("  {} Endpoint : {}", "→".dimmed(), endpoint.dimmed());
                });
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
                .send().await.map_err(|e| {
                    ai_spinner.finish_and_clear();
                    format!("{} Erreur réseau: {}", "❌", e)
                })?;

            if !res.status().is_success() {
                let status = res.status();
                let err_text = res.text().await.unwrap_or_default();
                ai_spinner.finish_and_clear();
                return Err(format!("{} Erreur API {} ({})\n  {}", "❌", conf.provider, status, err_text.dimmed()));
            }

            let data: serde_json::Value = res.json().await.map_err(|e| {
                ai_spinner.finish_and_clear();
                e.to_string()
            })?;
            data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string()
        },
        "anthropic" => {
            let api_key = conf.api_key.ok_or(format!("{} Clé API manquante", "❌"))?;

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
                .send().await.map_err(|e| {
                    ai_spinner.finish_and_clear();
                    format!("{} Erreur réseau: {}", "❌", e)
                })?;

            if !res.status().is_success() {
                ai_spinner.finish_and_clear();
                return Err(format!("{} Erreur API Anthropic: {}", "❌", res.status()));
            }

            let data: serde_json::Value = res.json().await.map_err(|e| {
                ai_spinner.finish_and_clear();
                e.to_string()
            })?;
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
                .send().await.map_err(|e| {
                    ai_spinner.finish_and_clear();
                    format!("{} Erreur réseau: {}", "❌", e)
                })?;

            if !res.status().is_success() {
                ai_spinner.finish_and_clear();
                return Err(format!("{} Erreur API Ollama: {}", "❌", res.status()));
            }

            let data: serde_json::Value = res.json().await.map_err(|e| {
                ai_spinner.finish_and_clear();
                e.to_string()
            })?;
            data["message"]["content"].as_str().unwrap_or("").to_string()
        },
        _ => {
            ai_spinner.finish_and_clear();
            return Err(format!("{} Fournisseur IA non supporté: {}.\n  {} Choisissez parmi: openai, openai_compatible, anthropic, ollama.", "❌", conf.provider.red(), "→".dimmed()));
        },
    };

    ai_spinner.finish_with_message(format!("{} Réponse reçue !", "✔".green()));

    if reply.is_empty() {
        println!("\n{}", "L'IA a répondu avec un format inattendu.".yellow());
    } else {
        println!("\n{}\n{}", "💡 Recommandations de l'IA :".bold().cyan(), reply);
    }

    Ok(())
}
