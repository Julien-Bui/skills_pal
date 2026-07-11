use crate::config;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

pub async fn run_doctor() -> Result<(), String> {
    println!("\n{}\n", "🩺 Skills Pal — Diagnostic".bold().cyan());

    let mut ok_count = 0;
    let mut warn_count = 0;
    let mut err_count = 0;

    // 1. Version
    let version = env!("CARGO_PKG_VERSION");
    println!("  {} Version installée : {}", "ℹ".cyan(), format!("v{}", version).bold());
    println!();

    // 2. Fichier de configuration
    print!("  ");
    match config::get_config() {
        Ok(conf) => {
            println!("{} Fichier de configuration trouvé", "✔".green());
            ok_count += 1;

            // 3. Provider configuré
            print!("  ");
            println!("{} Provider : {}", "ℹ".cyan(), conf.provider.bold());

            // 4. Clé API
            print!("  ");
            match &conf.api_key {
                Some(key) if !key.is_empty() 
                    && key != "VOTRE_CLE_MISTRAL_ICI" 
                    && key != "VOTRE_CLE_ICI" => {
                    let masked = format!("{}...{}", &key[..4.min(key.len())], &key[key.len().saturating_sub(4)..]);
                    println!("{} Clé API configurée ({})", "✔".green(), masked.dimmed());
                    ok_count += 1;
                },
                Some(_) => {
                    println!("{} Clé API par défaut — pensez à la configurer", "⚠".yellow());
                    warn_count += 1;
                },
                None => {
                    println!("{} Clé API manquante", "✖".red());
                    err_count += 1;
                },
            }

            // 5. Registry URL
            print!("  ");
            if let Some(url) = &conf.registry_url {
                if !url.is_empty() {
                    println!("{} Registry URL : {}", "ℹ".cyan(), url.dimmed());
                } else {
                    println!("{} Registry URL non configurée (utilisation du défaut)", "⚠".yellow());
                    warn_count += 1;
                }
            }
        },
        Err(_) => {
            println!("{} Aucun fichier de configuration trouvé", "✖".red());
            println!("    {} Lancez {} ou {}", "→".dimmed(), "skills_pal init".cyan(), "skills_pal init --global".cyan());
            err_count += 1;
        }
    }

    println!();

    // 6. Connectivité au serveur Railway
    print!("  ");
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("  {spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✔"]),
    );
    spinner.set_message("Test de connectivité au registre Railway...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let client = reqwest::Client::builder()
        .user_agent("Skills-Pal-CLI/1.0")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let registry_url = "https://skillspal-production-0511.up.railway.app/api/skills";
    match client.get(registry_url).send().await {
        Ok(res) if res.status().is_success() => {
            if let Ok(skills) = res.json::<Vec<serde_json::Value>>().await {
                spinner.finish_and_clear();
                println!("  {} Serveur Railway en ligne ({} skills dans le registre)", "✔".green(), skills.len().to_string().cyan());
                ok_count += 1;
            } else {
                spinner.finish_and_clear();
                println!("  {} Serveur Railway répond mais format inattendu", "⚠".yellow());
                warn_count += 1;
            }
        },
        Ok(res) => {
            spinner.finish_and_clear();
            println!("  {} Serveur Railway a répondu avec une erreur: {}", "⚠".yellow(), res.status());
            warn_count += 1;
        },
        Err(e) => {
            spinner.finish_and_clear();
            println!("  {} Serveur Railway injoignable: {}", "✖".red(), e.to_string().dimmed());
            err_count += 1;
        }
    }

    // 7. Dossier plugins
    print!("  ");
    if std::path::Path::new(config::PLUGINS_DIR).exists() {
        let count = std::fs::read_dir(config::PLUGINS_DIR)
            .map(|d| d.count())
            .unwrap_or(0);
        println!("{} Dossier plugins/ présent ({} éléments)", "✔".green(), count);
        ok_count += 1;
    } else {
        println!("{} Dossier plugins/ absent (sera créé au besoin)", "ℹ".cyan());
    }

    // 8. Git repo
    print!("  ");
    if std::path::Path::new(".git").exists() {
        println!("{} Dépôt Git détecté", "✔".green());
        ok_count += 1;
    } else {
        println!("{} Pas de dépôt Git dans ce dossier", "ℹ".cyan());
    }

    // Résumé
    println!("\n{}", "─".repeat(50).dimmed());
    println!(
        "  {} {} réussi(s)  {} {} avertissement(s)  {} {} erreur(s)\n",
        "✔".green(), ok_count,
        "⚠".yellow(), warn_count,
        "✖".red(), err_count,
    );

    if err_count == 0 && warn_count == 0 {
        println!("  {} {}\n", "🎉", "Tout est parfaitement configuré !".green().bold());
    } else if err_count == 0 {
        println!("  {} {}\n", "👍", "Fonctionnel, mais quelques points à améliorer.".yellow());
    } else {
        println!("  {} {}\n", "🔧", "Des corrections sont nécessaires pour utiliser Skills Pal.".red());
    }

    Ok(())
}
