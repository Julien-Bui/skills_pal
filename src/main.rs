mod cli;
mod ai;
mod config;
mod models;
mod database;
mod analyzer;
mod downloader;
mod browse;
mod doctor;
mod hooks;

use clap::Parser;
use colored::Colorize;

#[tokio::main]
async fn main() {
    let args = cli::Cli::parse();
    let verbose = args.verbose;

    // Initialisation silencieuse de la BDD si elle n'existe pas
    let _ = database::init_db(config::DB_PATH);

    match args.command {
        cli::Commands::Init { provider, model, api_key, global } => {
            match config::init_config(provider, model, api_key, global) {
                Ok(()) => {},
                Err(e) => eprintln!("{} {}", "❌".red(), e.red()),
            }
        },
        cli::Commands::Recom => {
            if let Err(e) = ai::run_recommendation(verbose).await {
                eprintln!("{}", e);
            }
            // Notification passive de mise à jour
            check_update_passive().await;
        },
        cli::Commands::Scan { path } => {
            println!("{} Lancement du scan sur {}...", "🔍", path.cyan());
            match analyzer::scan_project(path.clone()).await {
                Ok(reports) => {
                    if reports.is_empty() {
                        println!("{} {}", "✅".green(), "Aucun TODO ni FIXME trouvé. Ton projet est parfaitement clean !".green());
                    } else {
                        println!("\n{} {} problème(s) trouvé(s) :\n", "📋".to_string(), reports.len().to_string().bold());
                        for r in reports {
                            let path = r.file_path;
                            let line = r.line_number.map(|l| l.to_string()).unwrap_or_default();
                            let severity_colored = match r.severity.as_str() {
                                "error" => format!("[{}]", r.severity.to_uppercase()).red().bold().to_string(),
                                "warning" => format!("[{}]", r.severity.to_uppercase()).yellow().bold().to_string(),
                                _ => format!("[{}]", r.severity.to_uppercase()).dimmed().to_string(),
                            };
                            println!("  {} {}:{} → {}", severity_colored, path.dimmed(), line.dimmed(), r.message);
                        }
                    }
                },
                Err(e) => eprintln!("{} {}", "❌".red(), e.red()),
            }
            // Notification passive de mise à jour
            check_update_passive().await;
        },
        cli::Commands::Update => {
            println!("{} Recherche de mises à jour sur Github...", "🔄".to_string());
            let status = self_update::backends::github::Update::configure()
                .repo_owner("Julien-Bui")
                .repo_name("skills_pal")
                .bin_name("skills_pal")
                .show_download_progress(true)
                .current_version(env!("CARGO_PKG_VERSION"))
                .build();

            match status {
                Ok(updater) => {
                    match updater.update() {
                        Ok(status) => {
                            if status.updated() {
                                println!("{} Skills Pal mis à jour avec succès vers la version {} !", "✅".green(), status.version().to_string().cyan().bold());
                            } else {
                                println!("{} Tu as déjà la dernière version ({}) !", "✨".to_string(), status.version().to_string().green());
                            }
                        },
                        Err(e) => {
                            let err_msg = e.to_string();
                            if err_msg.contains("Permission denied") || err_msg.contains("os error 13") {
                                eprintln!("{} {}", "❌".red(), "Permission refusée.".red());
                                eprintln!("  {} L'outil est installé dans un dossier système. Relance avec :", "→".dimmed());
                                eprintln!("  {}", "sudo skills_pal update".cyan().bold());
                            } else {
                                eprintln!("{} Erreur lors de la mise à jour: {}", "❌".red(), e.to_string().red());
                            }
                        }
                    }
                },
                Err(e) => eprintln!("{} Erreur de configuration: {}", "❌".red(), e.to_string().red()),
            }
        },
        cli::Commands::Clean => {
            println!("{} Nettoyage des fichiers générés...\n", "🧹".to_string());
            let files_to_clean = vec![
                config::PLUGINS_DIR,
                config::DB_PATH,
                config::CONFIG_FILE,
                "skills_pal.zip",
                "skills_pal.tar.gz"
            ];

            let mut cleaned_something = false;

            for path_str in files_to_clean {
                let path = std::path::Path::new(path_str);
                if path.exists() {
                    if path.is_dir() {
                        if let Err(e) = std::fs::remove_dir_all(path) {
                            eprintln!("  {} Erreur suppression '{}': {}", "❌".red(), path_str, e);
                        } else {
                            println!("  {} Dossier '{}' supprimé.", "✔".green(), path_str.dimmed());
                            cleaned_something = true;
                        }
                    } else {
                        if let Err(e) = std::fs::remove_file(path) {
                            eprintln!("  {} Erreur suppression '{}': {}", "❌".red(), path_str, e);
                        } else {
                            println!("  {} Fichier '{}' supprimé.", "✔".green(), path_str.dimmed());
                            cleaned_something = true;
                        }
                    }
                }
            }

            if !cleaned_something {
                println!("{} Rien à nettoyer, aucun fichier généré n'a été trouvé.", "✨".to_string());
            } else {
                println!("\n{} {}", "✅".green(), "Nettoyage complet terminé !".green().bold());
            }
        },
        cli::Commands::Browse => {
            if let Err(e) = browse::run_browse().await {
                eprintln!("{}", e);
            }
        },
        cli::Commands::Doctor => {
            if let Err(e) = doctor::run_doctor().await {
                eprintln!("{}", e);
            }
        },
        cli::Commands::Hook { action } => {
            let result = match action {
                cli::HookAction::Install => hooks::install_hook(),
                cli::HookAction::Uninstall => hooks::uninstall_hook(),
            };
            if let Err(e) = result {
                eprintln!("{}", e);
            }
        },
    }
}

/// Vérifie silencieusement s'il existe une nouvelle version sur Github.
/// Affiche un petit message jaune si oui, ne fait rien sinon.
async fn check_update_passive() {
    let current = env!("CARGO_PKG_VERSION");

    // On utilise l'API Github pour récupérer la dernière release
    let client = reqwest::Client::builder()
        .user_agent("Skills-Pal-CLI/1.0")
        .build();

    let client = match client {
        Ok(c) => c,
        Err(_) => return,
    };

    let url = "https://api.github.com/repos/Julien-Bui/skills_pal/releases/latest";
    if let Ok(res) = client.get(url).send().await {
        if res.status().is_success() {
            if let Ok(data) = res.json::<serde_json::Value>().await {
                if let Some(tag) = data["tag_name"].as_str() {
                    let latest = tag.trim_start_matches('v');
                    if latest != current {
                        println!(
                            "\n{} {} → {} disponible ! Lance {}",
                            "💡".to_string(),
                            format!("v{}", current).dimmed(),
                            format!("v{}", latest).yellow().bold(),
                            "sudo skills_pal update".cyan().bold()
                        );
                    }
                }
            }
        }
    }
}
