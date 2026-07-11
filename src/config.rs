use std::fs;
use serde::{Deserialize, Serialize};

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

pub fn init_config() -> Result<(), String> {
    if fs::metadata(CONFIG_FILE).is_ok() {
        println!("Le fichier {} existe déjà.", CONFIG_FILE);
        return Ok(());
    }

    let config = SkillsPalConfig {
        provider: "openai_compatible".to_string(),
        model: "mistral-large-latest".to_string(),
        api_key: Some("VOTRE_CLE_MISTRAL_ICI".to_string()),
        base_url: Some("https://api.mistral.ai/v1/chat/completions".to_string()), 
        registry_url: Some("https://skillspal-production-0511.up.railway.app".to_string()), // Serveur public par défaut
    };

    let toml_string = toml::to_string(&config).map_err(|e| e.to_string())?;
    fs::write(CONFIG_FILE, toml_string).map_err(|e| e.to_string())?;

    println!("Fichier {} généré avec succès ! Vous pouvez y configurer n'importe quel labo d'IA.", CONFIG_FILE);
    Ok(())
}

pub fn get_config() -> Result<SkillsPalConfig, String> {
    let content = fs::read_to_string(CONFIG_FILE).map_err(|_| format!("Impossible de lire {}. Lancez `skills_pal init`", CONFIG_FILE))?;
    let config: SkillsPalConfig = toml::from_str(&content).map_err(|_| "Fichier de configuration mal formaté".to_string())?;
    
    Ok(config)
}

pub struct AppConfig
{
    pub port: u16,
    pub host: String,
}

impl AppConfig
{
    pub fn new() -> Self
    {
        Self {
            port: 3000,
            host: String::from("127.0.0.1"),
        }
    }
}
