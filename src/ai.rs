use crate::config;
use reqwest::Client;
use serde_json::json;
use serde::Deserialize;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use crate::utils::get_project_files;

#[derive(Deserialize)]
struct RemoteSkill {
    name: String,
    description: String,
    github_url: String,
}

macro_rules! log_verbose {
    ($verbose:expr, $spinner:expr, $($arg:tt)*) => {
        if $verbose {
            $spinner.suspend(|| {
                println!("  {} {}", "→".dimmed(), format!($($arg)*).dimmed());
            });
        }
    };
}

pub async fn run_recommendation(verbose: bool) -> Result<(), String> {
    let conf = config::get_config()?;
    let client = Client::builder()
        .user_agent("Skills-Pal-CLI/1.0")
        .timeout(Duration::from_secs(30)) // Timeout explicite
        .build()
        .map_err(|e| e.to_string())?;

    let remote_skills = fetch_remote_skills(&client, &conf, verbose).await;
    let stack = analyze_project_stack();
    let (prompt_system, prompt_user) = build_ai_prompt(&remote_skills, &stack);
    
    let reply = call_llm_api(&client, &conf, &prompt_system, &prompt_user, verbose).await?;

    if reply.is_empty() {
        println!("\n{}", "L'IA a répondu avec un format inattendu.".yellow());
    } else {
        println!("\n{}\n{}", "💡 Recommandations de l'IA :".bold().cyan(), reply);
    }

    Ok(())
}

async fn fetch_remote_skills(client: &Client, conf: &config::SkillsPalConfig, verbose: bool) -> Vec<RemoteSkill> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}").unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✔"]),
    );
    spinner.set_message("Contact du registre Railway...");
    spinner.enable_steady_tick(Duration::from_millis(80));

    let mut registry = "https://skillspal-production-0511.up.railway.app".to_string();
    if let Some(url) = &conf.registry_url {
        if !url.is_empty() {
            registry = url.clone();
        }
    }

    let registry_url = format!("{}/api/skills", registry.trim_end_matches('/'));
    log_verbose!(verbose, spinner, "URL du registre : {}", registry_url);

    match client.get(&registry_url).send().await {
        Ok(res) if res.status().is_success() => {
            if let Ok(skills) = res.json::<Vec<RemoteSkill>>().await {
                spinner.finish_with_message(format!("{} {} skills récupérés !", "✔".green(), skills.len().to_string().cyan()));
                return skills;
            }
        },
        Ok(res) => spinner.finish_with_message(format!("{} Erreur du registre: {}", "⚠".yellow(), res.status())),
        Err(e) => spinner.finish_with_message(format!("{} Impossible de contacter le registre: {}", "⚠".yellow(), e)),
    }
    
    Vec::new()
}

pub fn analyze_project_stack() -> String {
    println!("\n{} Analyse dynamique du projet...", "🔍".to_string());
    let mut extensions = std::collections::HashMap::new();

    for entry in get_project_files(".") {
        if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
            *extensions.entry(ext.to_string()).or_insert(0) += 1;
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
    stack
}

fn build_ai_prompt(remote_skills: &[RemoteSkill], stack: &str) -> (String, String) {
    let mut prompt_system = "Tu es l'IA de Skills Pal. Propose 2 ou 3 outils, plugins ou compétences d'ingénierie pertinents pour améliorer ce projet dans son ensemble en te basant sur la stack fournie. Réponds de manière concise.".to_string();

    if !remote_skills.is_empty() {
        prompt_system.push_str("\n\nVoici la base de plugins. Priorise ces recommandations si pertinentes (donne l'URL) :\n");
        for skill in remote_skills.iter().take(20) {
            prompt_system.push_str(&format!("- {} ({}): {}\n", skill.name, skill.github_url, skill.description));
        }
    }

    let prompt_user = format!("Le projet utilise : {}", stack);
    (prompt_system, prompt_user)
}

async fn call_llm_api(client: &Client, conf: &config::SkillsPalConfig, prompt_system: &str, prompt_user: &str, verbose: bool) -> Result<String, String> {
    let ai_spinner = ProgressBar::new_spinner();
    ai_spinner.set_style(
        ProgressStyle::with_template("{spinner:.magenta} {msg}").unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✔"]),
    );
    ai_spinner.set_message(format!("Contact de l'IA ({})...", conf.provider.to_uppercase().magenta()));
    ai_spinner.enable_steady_tick(Duration::from_millis(80));

    log_verbose!(verbose, ai_spinner, "Modèle : {}", conf.model);
    log_verbose!(verbose, ai_spinner, "Prompt système : {} caractères", prompt_system.len());

    let reply = match conf.provider.to_lowercase().as_str() {
        "openai" | "openai_compatible" => {
            let api_key = conf.api_key.as_ref().ok_or_else(|| format!("{} Clé API manquante", "❌".red()))?;
            if api_key == "VOTRE_CLE_MISTRAL_ICI" || api_key == "VOTRE_CLE_ICI" || api_key.is_empty() {
                ai_spinner.finish_and_clear();
                return Err(format!("{} Clé API non configurée.", "❌"));
            }

            let endpoint = conf.base_url.clone().unwrap_or_else(|| "https://api.openai.com/v1/chat/completions".to_string());
            log_verbose!(verbose, ai_spinner, "Endpoint : {}", endpoint);

            let res = client.post(&endpoint)
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&json!({
                    "model": conf.model,
                    "messages": [
                        { "role": "system", "content": prompt_system },
                        { "role": "user", "content": prompt_user }
                    ]
                }))
                .send().await.map_err(|e| { ai_spinner.finish_and_clear(); format!("{} Erreur réseau: {}", "❌", e) })?;

            if !res.status().is_success() {
                ai_spinner.finish_and_clear();
                return Err(format!("{} Erreur API {} ({})", "❌", conf.provider, res.status()));
            }

            let data: serde_json::Value = res.json().await.map_err(|e| { ai_spinner.finish_and_clear(); e.to_string() })?;
            data["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string()
        },
        "anthropic" => {
            let api_key = conf.api_key.as_ref().ok_or_else(|| format!("{} Clé API manquante", "❌"))?;
            let res = client.post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .json(&json!({
                    "model": conf.model,
                    "max_tokens": 500,
                    "system": prompt_system,
                    "messages": [ { "role": "user", "content": prompt_user } ]
                }))
                .send().await.map_err(|e| { ai_spinner.finish_and_clear(); format!("{} Erreur réseau: {}", "❌", e) })?;

            if !res.status().is_success() {
                ai_spinner.finish_and_clear();
                return Err(format!("{} Erreur API Anthropic: {}", "❌", res.status()));
            }

            let data: serde_json::Value = res.json().await.map_err(|e| { ai_spinner.finish_and_clear(); e.to_string() })?;
            data["content"][0]["text"].as_str().unwrap_or("").to_string()
        },
        "ollama" => {
            let url = if let Some(base) = &conf.base_url {
                format!("{}/api/chat", base.trim_end_matches('/'))
            } else {
                "http://localhost:11434/api/chat".to_string()
            };

            let res = client.post(&url)
                .json(&json!({
                    "model": conf.model,
                    "messages": [
                        { "role": "system", "content": prompt_system },
                        { "role": "user", "content": prompt_user }
                    ],
                    "stream": false
                }))
                .send().await.map_err(|e| { ai_spinner.finish_and_clear(); format!("{} Erreur réseau: {}", "❌", e) })?;

            if !res.status().is_success() {
                ai_spinner.finish_and_clear();
                return Err(format!("{} Erreur API Ollama: {}", "❌", res.status()));
            }

            let data: serde_json::Value = res.json().await.map_err(|e| { ai_spinner.finish_and_clear(); e.to_string() })?;
            data["message"]["content"].as_str().unwrap_or("").to_string()
        },
        _ => {
            ai_spinner.finish_and_clear();
            return Err(format!("{} Fournisseur IA non supporté: {}", "❌", conf.provider));
        },
    };

    ai_spinner.finish_with_message(format!("{} Réponse reçue !", "✔".green()));
    Ok(reply)
}

pub async fn generate_commit_message(diff: &str, verbose: bool) -> Result<String, String> {
    let conf = config::get_config()?;
    let client = Client::builder()
        .user_agent("Skills-Pal-CLI/1.0")
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let prompt_system = "Tu es l'IA de Skills Pal. Analyse le diff Git fourni et génère un message de commit concis au format Conventional Commits (ex: feat: ..., fix: ..., refactor: ...). Ne renvoie QUE le message, sans rien d'autre. Pas d'introduction, pas de guillemets autour, et pas de conclusion.";
    let prompt_user = format!("Voici le diff Git :\n{}", diff);

    let mut reply = call_llm_api(&client, &conf, prompt_system, &prompt_user, verbose).await?;
    
    // Nettoyage de sécurité si l'IA renvoie quand même des guillemets
    reply = reply.trim().trim_matches('"').trim_matches('\'').to_string();
    
    Ok(reply)
}

pub async fn generate_readme(stack: &str, sample: &str, verbose: bool) -> Result<String, String> {
    let conf = config::get_config()?;
    let client = Client::builder()
        .user_agent("Skills-Pal-CLI/1.0")
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| e.to_string())?;

    let prompt_system = "Tu es l'IA de Skills Pal. Rédige un fichier README.md complet et professionnel pour ce projet. Inclus un titre, une description, une section d'installation et d'utilisation. Ne renvoie QUE le contenu Markdown, sans bloc ```markdown autour.";
    let prompt_user = format!("Stack du projet: {}\n\nAperçu du code source principal :\n{}", stack, sample);

    let reply = call_llm_api(&client, &conf, prompt_system, &prompt_user, verbose).await?;
    
    Ok(reply)
}

