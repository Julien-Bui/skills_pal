use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use directories::ProjectDirs;

pub const DB_PATH: &str = "skills_pal.db";
pub const PLUGINS_DIR: &str = "plugins/";
pub const CONFIG_FILE: &str = ".skillspal.toml";

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct SkillsPalConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub registry_url: Option<String>,
}

/// Retourne le chemin du dossier de configuration global (~/.config/skills_pal/)
pub fn get_global_config_dir() -> Option<PathBuf> {
    ProjectDirs::from("com", "skills-pal", "skills_pal")
        .map(|dirs| dirs.config_dir().to_path_buf())
}

/// Retourne le chemin du fichier de configuration global
fn get_global_config_path() -> Option<PathBuf> {
    get_global_config_dir().map(|dir| dir.join("config.toml"))
}

fn default_config() -> SkillsPalConfig {
    SkillsPalConfig {
        provider: "openai_compatible".to_string(),
        model: "mistral-large-latest".to_string(),
        api_key: Some("VOTRE_CLE_MISTRAL_ICI".to_string()),
        base_url: Some("https://api.mistral.ai/v1/chat/completions".to_string()),
        registry_url: Some("https://skillspal-production-0511.up.railway.app".to_string()),
    }
}

pub fn init_config(
    provider: Option<String>,
    model: Option<String>,
    api_key: Option<String>,
    global: bool,
) -> Result<(), String> {
    let target_path = if global {
        let dir = get_global_config_dir()
            .ok_or("Impossible de déterminer le dossier de configuration global.")?;
        fs::create_dir_all(&dir).map_err(|e| format!("Impossible de créer le dossier {:?}: {}", dir, e))?;
        dir.join("config.toml")
    } else {
        PathBuf::from(CONFIG_FILE)
    };

    if target_path.exists() {
        println!("Le fichier {} existe déjà.", target_path.display());
        return Ok(());
    }

    let mut config = default_config();

    // Surcharger avec les paramètres passés en ligne de commande
    if let Some(p) = provider {
        config.provider = p;
    }
    if let Some(m) = model {
        config.model = m;
    }
    if let Some(k) = api_key {
        config.api_key = Some(k);
    }

    let toml_string = toml::to_string(&config).map_err(|e| e.to_string())?;
    fs::write(&target_path, toml_string).map_err(|e| e.to_string())?;

    println!("Fichier {} généré avec succès !", target_path.display());
    Ok(())
}

pub fn get_config() -> Result<SkillsPalConfig, String> {
    // 1. Chercher d'abord un fichier local (.skillspal.toml dans le dossier courant)
    if let Ok(content) = fs::read_to_string(CONFIG_FILE) {
        let config: SkillsPalConfig = toml::from_str(&content)
            .map_err(|_| "Fichier de configuration local mal formaté".to_string())?;
        return Ok(config);
    }

    // 2. Fallback sur le fichier global (~/.config/skills_pal/config.toml)
    if let Some(global_path) = get_global_config_path() {
        if let Ok(content) = fs::read_to_string(&global_path) {
            let config: SkillsPalConfig = toml::from_str(&content)
                .map_err(|_| "Fichier de configuration global mal formaté".to_string())?;
            return Ok(config);
        }
    }

    Err(format!(
        "Aucun fichier de configuration trouvé.\n  → Lancez `skills_pal init` (local) ou `skills_pal init --global` (global) pour en créer un."
    ))
}
