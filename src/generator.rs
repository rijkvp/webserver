use crate::{config::ServerConfig, rss::feed_items_to_rss, template_engine::TemplateEngine};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};
use tera::Context;

#[derive(Serialize, Deserialize, Clone)]
struct FeedLink {
    content: String,
    url: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct FeedImage {
    alt: String,
    file_name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FeedItem {
    pub title: String,
    subtitle: String,
    #[serde(with = "date_format")]
    pub date: NaiveDate,
    date_label: String,
    tags: Vec<String>,
    image: FeedImage,
    pub content: String,
    links: Vec<FeedLink>,
}

mod date_format {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%d";

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
        Ok(dt)
    }
}

#[derive(Clone)]
pub struct Generator {
    files: HashMap<PathBuf, String>,
}

#[derive(Serialize)]
struct EmptyFeedIndex {
    pub feed_url: Option<PathBuf>,
    pub items: Vec<FeedItem>,
}

#[derive(Serialize)]
struct ContentFeedIndex {
    pub feed_url: Option<PathBuf>,
    pub items: Vec<(PathBuf, FeedItem)>,
}

impl Generator {
    pub fn generate(
        config: &ServerConfig,
        template_engine: &mut TemplateEngine,
    ) -> Result<Self, String> {
        let mut files = HashMap::new();

        for feed_cfg in &config.feeds {
            let source_dir = config.target_dir.join(&feed_cfg.source_dir);
            let mut feed_items = Vec::<(String, FeedItem)>::new();

            if source_dir.is_dir() {
                for file in fs::read_dir(&source_dir).map_err(|err| err.to_string())? {
                    let file = file.map_err(|err| err.to_string())?;
                    let path = file.path();
                    if !path.is_dir() {
                        let content = fs::read_to_string(&path).map_err(|err| err.to_string())?;
                        let item = serde_yaml::from_str::<FeedItem>(&content).map_err(|err| {
                            format!(
                                "Failed to deserialize file '{}': {}",
                                path.display(),
                                err.to_string(),
                            )
                        })?;

                        if let Some(file_name) = &path.file_stem() {
                            feed_items.push((file_name.to_string_lossy().to_string(), item));
                        } else {
                            return Err(format!(
                                "Failed to get file stem from path: {}",
                                path.display()
                            ));
                        }
                    }
                }
            }

            // Sort the feed ascending by
            feed_items.sort_by(|a, b| b.1.date.cmp(&a.1.date));

            // Generate content
            if let Some(content_output) = &feed_cfg.content_output {
                let template_path = config.target_dir.join(&content_output.template);
                let content_template = fs::read_to_string(&template_path).map_err(|err| {
                    format!(
                        "Failed to load template file '{}': {}",
                        &template_path.display(),
                        err.to_string()
                    )
                })?;
                for (file_name, item) in &feed_items {
                    let context = Context::from_serialize(&item).map_err(|err| err.to_string())?;
                    let rendered_content = template_engine
                        .render_string(&content_template, &context)
                        .map_err(|err| {
                            format!(
                                "Failed to render content template '{}': {}",
                                &content_output.template.display(),
                                err.to_string()
                            )
                        })?;
                    let path = content_output.link.join(file_name);
                    files.insert(path, rendered_content);
                }
            }

            // Generate index
            if let Some(index_output) = &feed_cfg.index_output {
                let template_path = config.target_dir.join(&index_output.template);
                let index_template = fs::read_to_string(&template_path).map_err(|err| {
                    format!(
                        "Failed to load index template file '{}': {}",
                        &template_path.display(),
                        err.to_string()
                    )
                })?;

                let index_context = {
                    if let Some(_) = feed_cfg.content_output {
                        let items: Vec<(PathBuf, FeedItem)> = feed_items
                            .iter()
                            .map(|(file_name, item)| {
                                let path = index_output.link.join(file_name);
                                (path, item.clone())
                            })
                            .collect();
                        let index = ContentFeedIndex {
                            feed_url: feed_cfg.rss_feed_url.clone(),
                            items,
                        };
                        Context::from_serialize(&index).map_err(|err| err.to_string())?
                    } else {
                        let items: Vec<FeedItem> =
                            feed_items.iter().map(|(_, item)| item.clone()).collect();
                        let index = EmptyFeedIndex {
                            feed_url: feed_cfg.rss_feed_url.clone(),
                            items,
                        };
                        Context::from_serialize(&index).map_err(|err| err.to_string())?
                    }
                };
                let index_content = template_engine
                    .render_string(&index_template, &index_context)
                    .map_err(|err| {
                        format!(
                            "Failed to render index template '{}': {}",
                            &index_output.template.display(),
                            err.to_string()
                        )
                    })?;
                files.insert(index_output.link.clone(), index_content);
            }

            // Generate RSS feed
            if let Some(rss_feed_url) = &feed_cfg.rss_feed_url {
                let full_rss_url = config.server_name.clone()
                    + "/"
                    + rss_feed_url
                        .to_str()
                        .ok_or("Failed to convert feed url pathbuf to string!".to_string())?;

                let rss_str = feed_items_to_rss(&feed_items, &feed_cfg, &full_rss_url);

                files.insert(rss_feed_url.clone(), rss_str);
            }
        }
        Ok(Self { files })
    }

    pub fn get(&self, path: &PathBuf) -> Option<&String> {
        self.files.get(path)
    }
}
