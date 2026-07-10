mod cli;
mod ai;
mod config;
mod models;
mod database;
mod analyzer;
mod downloader;

use clap::Parser;

#[tokio::main]
async fn main() {
    let args = cli::Cli::parse();

    // Initialisation silencieuse de la BDD si elle n'existe pas
    let _ = database::init_db(config::DB_PATH);

    match args.command {
        cli::Commands::Init => {
            if let Err(e) = config::init_config() {
                eprintln!("Erreur lors de l'initialisation: {}", e);
            }
        },
        cli::Commands::Recom => {
            if let Err(e) = ai::run_recommendation().await {
                eprintln!("Erreur lors de la recommandation: {}", e);
            }
        },
        cli::Commands::Scan => {
            let current_dir = std::env::current_dir().unwrap().display().to_string();
            println!("Lancement du scan sur {}...", current_dir);
            match analyzer::scan_project(current_dir).await {
                Ok(reports) => {
                    if reports.is_empty() {
                        println!("✅ Aucun TODO ni FIXME trouvé. Ton projet est parfaitement clean !");
                    } else {
                        for r in reports {
                            let path = r.file_path;
                            let line = r.line_number.map(|l| l.to_string()).unwrap_or_default();
                            println!("[{}] {}:{} -> {}", r.severity.to_uppercase(), path, line, r.message);
                        }
                    }
                },
                Err(e) => eprintln!("Erreur lors de l'analyse: {}", e),
            }
        },
        cli::Commands::Update => {
            println!("Recherche de mises à jour sur Github...");
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
                                println!("✅ Skills Pal a été mis à jour avec succès vers la version {} !", status.version());
                            } else {
                                println!("✨ Tu as déjà la dernière version ({}) !", status.version());
                            }
                        },
                        Err(e) => {
                            let err_msg = e.to_string();
                            if err_msg.contains("Permission denied") || err_msg.contains("os error 13") {
                                eprintln!("❌ Erreur : Permission refusée.");
                                eprintln!("👉 L'outil est installé dans un dossier système. Relance la commande avec les droits administrateur :");
                                eprintln!("   sudo skills_pal update");
                            } else {
                                eprintln!("❌ Erreur lors de la mise à jour: {}", e);
                            }
                        }
                    }
                },
                Err(e) => eprintln!("❌ Erreur de configuration de la mise à jour: {}", e),
            }
        },
        cli::Commands::Clean => {
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
                            eprintln!("❌ Erreur lors de la suppression de '{}' : {}", path_str, e);
                        } else {
                            println!("🧹 Dossier '{}' supprimé.", path_str);
                            cleaned_something = true;
                        }
                    } else {
                        if let Err(e) = std::fs::remove_file(path) {
                            eprintln!("❌ Erreur lors de la suppression de '{}' : {}", path_str, e);
                        } else {
                            println!("🧹 Fichier '{}' supprimé.", path_str);
                            cleaned_something = true;
                        }
                    }
                }
            }
            
            if !cleaned_something {
                println!("✨ Rien à nettoyer, aucun fichier généré n'a été trouvé.");
            } else {
                println!("✅ Nettoyage complet terminé !");
            }
        },
    }
}
