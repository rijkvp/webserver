use std::{fs, net::IpAddr, path::PathBuf};

use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct FeedOutput {
    pub template: PathBuf,
    pub link: PathBuf,
}

#[derive(Deserialize, Clone)]
pub struct FeedConfig {
    // Feed properties
    pub title: String,
    pub description: String,
    pub link: String,
    pub rss_feed_link: Option<PathBuf>,

    // Source
    pub source_dir: PathBuf,
    // Content files
    pub content_output: Option<FeedOutput>,
    // Index file
    pub index_output: Option<FeedOutput>,
}

#[derive(Deserialize, Clone)]
pub struct ServerConfig {
    pub address: IpAddr,
    pub port: u16,
    pub server_name: String,
    pub root_dir: PathBuf,
    pub index: PathBuf,
    pub error_template: Option<PathBuf>,
    pub content_ext: String,
    pub ignored_paths: Vec<PathBuf>,
    pub feeds: Vec<FeedConfig>,
}

impl ServerConfig {
    pub fn load(path: PathBuf) -> Result<Self, String> {
        let config_str = fs::read_to_string(&path).map_err(|err| {
            format!(
                "Failed to read config file: {}\nPath: {}",
                err,
                &path.display()
            )
        })?;
        let config = ron::from_str::<ServerConfig>(&config_str)
            .map_err(|err| format!("Failed to deserialize config file: {}", err))?;
        Ok(config)
    }
}
