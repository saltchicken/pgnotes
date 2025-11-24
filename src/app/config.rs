use serde::Deserialize;
use std::{fs, path::PathBuf};

pub const CONFIG_DIR_NAME: &str = "pgnotes";
pub const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_database_url")]
    pub database_url: String,
    pub editor: Option<String>,
}

fn default_database_url() -> String {
    "postgresql://saltchicken:password@10.0.0.5/pgnotes".to_string()
}

impl Config {
    pub fn new() -> Self {
        let config_dir_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(CONFIG_DIR_NAME);

        let _ = fs::create_dir_all(&config_dir_path);
        let config_path = config_dir_path.join(CONFIG_FILE_NAME);

        if !config_path.exists() {
            let _ = fs::write(
                &config_path,
                "# Configuration for pgnotes\n\n# PostgreSQL connection string.\ndatabase_url = \"postgresql://user:password@localhost/postgres\"\n\n# editor = \"nvim\"\n",
            );
        }

        Self::load(&config_path)
    }

    fn load(path: &PathBuf) -> Self {
        if let Ok(content) = fs::read_to_string(path) {
            return toml::from_str(&content).unwrap_or(Config::default());
        }
        Config::default()
    }

    pub fn get_editor_command(&self) -> String {
        self.editor
            .clone()
            .or_else(|| std::env::var("EDITOR").ok())
            .unwrap_or_else(|| "nvim".to_string())
    }
}


impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: default_database_url(),
            editor: None,
        }
    }
}