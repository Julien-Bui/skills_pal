use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "skills_pal")]
#[command(about = "L'assistant IA pour la dette technique et les plugins", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialise le fichier de configuration local (.skillspal.toml)
    Init,
    /// Analyse le projet et demande à l'IA des recommandations de plugins
    Recom,
    /// Scanne le projet avec les plugins installés
    Scan,
    /// Met à jour l'outil vers la dernière version disponible sur Github
    Update,
    /// Nettoie les dossiers générés par l'outil (ex: dossier plugins)
    Clean,
}
