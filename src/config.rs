pub const DB_PATH: &str = "skills_pal.db";
pub const PLUGINS_DIR: &str = "plugins";

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
