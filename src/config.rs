use std::{fs, path::PathBuf};

use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct GenDirectory {
    pub source_dir: PathBuf,
    pub target_url: PathBuf,
    pub content_template: PathBuf,
    pub index_template: PathBuf,
    pub index_url: PathBuf,
}

#[derive(Deserialize, Clone)]
pub struct ServerConfig {
    pub target_dir: PathBuf,
    pub index_file: PathBuf,
    pub error_template: PathBuf,
    pub content_ext: String,
    pub ignored_paths: Vec<PathBuf>,
    pub gen_dirs: Vec<GenDirectory>,
}

impl ServerConfig {
    pub fn load() -> Result<Self, String> {
        let config_str = fs::read_to_string("server_config.ron")
            .map_err(|err| format!("Failed to read config file:\n{}", err))?;
        let config = ron::from_str::<ServerConfig>(&config_str)
            .map_err(|err| format!("Failed to deserialize config file:\n{}", err))?;

        Ok(config)
    }
}
