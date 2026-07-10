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
                        Err(e) => eprintln!("❌ Erreur lors de la mise à jour: {}", e),
                    }
                },
                Err(e) => eprintln!("❌ Erreur de configuration de la mise à jour: {}", e),
            }
        }
    }
}
