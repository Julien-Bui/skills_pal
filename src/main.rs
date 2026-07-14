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
mod utils;
mod git;

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
                        for r in reports.iter() {
                            let path = &r.file_path;
                            let line = r.line_number.map(|l| l.to_string()).unwrap_or_default();
                            let severity_colored = match r.severity.as_str() {
                                "error" => format!("[{}]", r.severity.to_uppercase()).red().bold().to_string(),
                                "warning" => format!("[{}]", r.severity.to_uppercase()).yellow().bold().to_string(),
                                _ => format!("[{}]", r.severity.to_uppercase()).dimmed().to_string(),
                            };
                            println!("  {} {}:{} → {}", severity_colored, path.dimmed(), line.dimmed(), r.message);
                        }
                    }

                    // Enregistrer l'historique
                    if let Ok(db) = database::init_db(config::DB_PATH) {
                        let _ = database::insert_scan_history(&db, reports.len() as i32);
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
                cli::HookAction::Uninstall | cli::HookAction::Disable => hooks::uninstall_hook(),
            };
            if let Err(e) = result {
                eprintln!("{}", e);
            }
        },
        cli::Commands::Commit => {
            match git::get_staged_diff() {
                Ok(diff) => {
                    println!("{} Récupération du diff ({} caractères)...", "📦".cyan(), diff.len());
                    match ai::generate_commit_message(&diff, verbose).await {
                        Ok(msg) => {
                            println!("\n{} Message de commit généré par l'IA :\n", "✨".bold().yellow());
                            println!("  {}\n", msg.cyan().bold());
                            
                            let confirm = dialoguer::Confirm::new()
                                .with_prompt("Veux-tu utiliser ce message pour le commit ?")
                                .default(true)
                                .interact()
                                .unwrap_or(false);

                            if confirm {
                                if let Err(e) = git::commit(&msg) {
                                    eprintln!("{} {}", "❌".red(), e.red());
                                } else {
                                    println!("{} Commit créé avec succès !", "✔".green());
                                }
                            } else {
                                println!("{} Opération annulée.", "ℹ".dimmed());
                            }
                        },
                        Err(e) => eprintln!("{} Erreur IA : {}", "❌".red(), e.red()),
                    }
                },
                Err(e) => eprintln!("{} {}", "❌".red(), e.red()),
            }
        },
        cli::Commands::Stats => {
            if let Ok(db) = database::init_db(config::DB_PATH) {
                if let Ok(history) = database::get_scan_history(&db) {
                    println!("\n{} Évolution de la dette technique :\n", "📈".bold().cyan());
                    if history.is_empty() {
                        println!("  {}", "Aucun historique disponible. Lance un scan d'abord !".dimmed());
                    } else {
                        for (date, count) in &history {
                            let trend = if *count == 0 {
                                "✨ Parfait".green()
                            } else if *count < 10 {
                                "⚠️ Gérable".yellow()
                            } else {
                                "🚨 Critique".red()
                            };
                            println!("  {} - {} problèmes ({})", date.dimmed(), count.to_string().bold(), trend);
                        }
                    }
                }
            } else {
                eprintln!("{} Impossible d'accéder à la base locale.", "❌".red());
            }
        },
        cli::Commands::Doc => {
            let stack = ai::analyze_project_stack();
            println!("{} Génération du README par l'IA...", "📝".cyan());
            
            // On essaie de lire un fichier principal
            let mut sample = String::new();
            if let Ok(content) = std::fs::read_to_string("src/main.rs") {
                sample = content.chars().take(2000).collect();
            } else if let Ok(content) = std::fs::read_to_string("index.js") {
                sample = content.chars().take(2000).collect();
            }

            match ai::generate_readme(&stack, &sample, verbose).await {
                Ok(doc) => {
                    println!("\n{} README.md généré avec succès :\n", "✨".bold().yellow());
                    let preview: String = doc.lines().take(10).collect::<Vec<_>>().join("\n");
                    println!("{}\n[...]\n", preview.cyan());

                    let confirm = dialoguer::Confirm::new()
                        .with_prompt("Écraser ton README.md actuel avec cette version ?")
                        .default(false)
                        .interact()
                        .unwrap_or(false);

                    if confirm {
                        if let Err(e) = std::fs::write("README.md", doc) {
                            eprintln!("{} Erreur d'écriture: {}", "❌".red(), e);
                        } else {
                            println!("{} README.md mis à jour !", "✔".green());
                        }
                    } else {
                        println!("{} Opération annulée.", "ℹ".dimmed());
                    }
                },
                Err(e) => eprintln!("{} Erreur IA : {}", "❌".red(), e.red()),
            }
        },
        cli::Commands::Ignore => {
            let stack = ai::analyze_project_stack();
            println!("{} Configuration du .gitignore intelligent...", "🧹".cyan());
            
            let mut additions = Vec::new();
            if stack.contains(".rs") { additions.extend(vec!["/target/", "Cargo.lock"]); }
            if stack.contains(".js") || stack.contains(".ts") { additions.push("node_modules/"); }
            if stack.contains(".py") { additions.extend(vec!["__pycache__/", "venv/", ".env"]); }

            if additions.is_empty() {
                println!("{} Aucune règle spécifique détectée pour cette stack.", "ℹ".dimmed());
            } else {
                use std::io::Write;
                let mut file = std::fs::OpenOptions::new().create(true).append(true).open(".gitignore").unwrap();
                writeln!(file, "\n# Skills Pal Smart Gitignore").unwrap();
                for rule in &additions {
                    writeln!(file, "{}", rule).unwrap();
                }
                println!("{} Règles ajoutées au .gitignore : {}", "✔".green(), additions.join(", ").yellow());
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
