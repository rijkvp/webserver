use std::{
    path::{PathBuf},
    str::FromStr,
};

use serde::Deserialize;
use tokio::fs;

#[derive(Deserialize)]
pub struct ServerConfigFile {
    pub target_dir: String,
    pub index_file: String,
    pub content_ext: String,
}

pub struct ServerConfig {
    pub target_dir: PathBuf,
    pub index_file: PathBuf,
    pub content_ext: String,
}

impl ServerConfig {
    pub async fn load() -> Result<Self, String> {
        let config_str = fs::read_to_string("server_config.ron")
            .await
            .map_err(|err| format!("Failed to read config file:\n{}", err))?;
        let config_file = ron::from_str::<ServerConfigFile>(&config_str)
            .map_err(|err| format!("Failed to deserialize config file:\n{}", err))?;

        let config = ServerConfig {
            target_dir: PathBuf::from_str(&config_file.target_dir)
                .map_err(|err| err.to_string())?,
            index_file: PathBuf::from_str(&config_file.index_file)
                .map_err(|err| err.to_string())?,
            content_ext: config_file.content_ext,
        };

        Ok(config)
    }
}
