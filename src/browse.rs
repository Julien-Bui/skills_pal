use crate::config;
use reqwest::Client;
use serde::Deserialize;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use dialoguer::{FuzzySelect, theme::ColorfulTheme};

#[derive(Deserialize)]
struct RemoteSkill {
    name: String,
    description: String,
    github_url: String,
}

pub async fn run_browse() -> Result<(), String> {
    // Spinner pendant le chargement
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✔"]),
    );
    spinner.set_message("Récupération des skills depuis le registre...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let client = Client::builder()
        .user_agent("Skills-Pal-CLI/1.0")
        .build()
        .map_err(|e| e.to_string())?;

    // Déterminer l'URL du registre
    let mut registry = "https://skillspal-production-0511.up.railway.app".to_string();
    if let Ok(conf) = config::get_config() {
        if let Some(url) = &conf.registry_url {
            if !url.is_empty() {
                registry = url.clone();
            }
        }
    }

    let url = format!("{}/api/skills", registry.trim_end_matches('/'));
    let res = client.get(&url).send().await.map_err(|e| format!("Erreur réseau: {}", e))?;

    if !res.status().is_success() {
        spinner.finish_and_clear();
        return Err(format!("{} Le registre a retourné une erreur: {}", "❌", res.status()));
    }

    let skills: Vec<RemoteSkill> = res.json().await.map_err(|e| format!("Erreur JSON: {}", e))?;

    if skills.is_empty() {
        spinner.finish_and_clear();
        println!("{} Aucun skill disponible dans le registre.", "⚠".yellow());
        return Ok(());
    }

    spinner.finish_with_message(format!("{} {} skills chargés !", "✔".green(), skills.len().to_string().cyan()));

    // Construire les options du menu
    let items: Vec<String> = skills.iter()
        .map(|s| format!("{} — {}", s.name, s.description))
        .collect();

    println!("\n{}\n", "Navigue avec ↑↓, tape pour filtrer, Entrée pour sélectionner, Échap pour quitter.".dimmed());

    // Menu interactif avec recherche fuzzy
    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Sélectionne un skill")
        .items(&items)
        .default(0)
        .interact_opt()
        .map_err(|e| format!("Erreur du menu interactif: {}", e))?;

    match selection {
        Some(idx) => {
            let skill = &skills[idx];
            println!("\n{} {}", "📦 Skill sélectionné :".bold(), skill.name.cyan().bold());
            println!("  {} {}", "Description :".dimmed(), skill.description);
            println!("  {} {}", "GitHub :".dimmed(), skill.github_url.underline());

            // Confirmation pour ouvrir dans le navigateur
            let open = dialoguer::Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Ouvrir dans le navigateur ?")
                .default(true)
                .interact()
                .unwrap_or(false);

            if open {
                if let Err(e) = open_url(&skill.github_url) {
                    println!("{} Impossible d'ouvrir le navigateur: {}", "⚠".yellow(), e);
                    println!("  {} Copie ce lien : {}", "→".dimmed(), skill.github_url.cyan());
                }
            }
        },
        None => {
            println!("{}", "Aucune sélection.".dimmed());
        }
    }

    Ok(())
}

fn open_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", url])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
