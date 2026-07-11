use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "skills_pal")]
#[command(version)]
#[command(about = "L'assistant IA pour la dette technique et les plugins", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Affiche les détails techniques (debug)
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialise le fichier de configuration (.skillspal.toml)
    Init {
        /// Fournisseur IA (ex: openai_compatible, openai, anthropic, ollama)
        #[arg(long)]
        provider: Option<String>,

        /// Modèle IA à utiliser (ex: mistral-large-latest, gpt-4o)
        #[arg(long)]
        model: Option<String>,

        /// Clé API du fournisseur
        #[arg(long)]
        api_key: Option<String>,

        /// Créer la configuration dans le dossier global (~/.config/skills_pal/)
        #[arg(long)]
        global: bool,
    },
    /// Analyse le projet et demande à l'IA des recommandations de plugins
    Recom,
    /// Scanne le projet pour détecter la dette technique (TODO, FIXME, clippy)
    Scan {
        /// Chemin du dossier à scanner (par défaut: dossier courant)
        #[arg(short, long, default_value = ".")]
        path: String,
    },
    /// Met à jour l'outil vers la dernière version disponible sur Github
    Update,
    /// Supprime tous les fichiers générés par l'outil (config, db, plugins)
    Clean,
}
