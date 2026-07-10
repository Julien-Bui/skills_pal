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
                    for r in reports {
                        let path = r.file_path;
                        let line = r.line_number.map(|l| l.to_string()).unwrap_or_default();
                        println!("[{}] {}:{} -> {}", r.severity.to_uppercase(), path, line, r.message);
                    }
                },
                Err(e) => eprintln!("Erreur lors de l'analyse: {}", e),
            }
        }
    }
}
