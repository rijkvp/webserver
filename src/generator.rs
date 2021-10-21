use crate::{config::ServerConfig, rss::generate_rss_xml, template_engine::TemplateEngine};
use chrono::NaiveDate;
use comrak::{markdown_to_html, ComrakOptions};
use serde::{self, Deserialize, Serialize};
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
enum FeedContentType {
    #[serde(rename = "html")]
    Html,
    #[serde(rename = "md")]
    Markdown,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FeedMeta {
    pub title: String,
    subtitle: Option<String>,
    #[serde(with = "date_format")]
    pub date: NaiveDate,
    date_label: Option<String>,
    tags: Option<Vec<String>>,
    image: Option<FeedImage>,
    links: Option<Vec<FeedLink>>,
    content_type: FeedContentType,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FeedItem {
    pub file_name: String,
    pub meta: FeedMeta,
    pub content: String,
    pub link: Option<String>,
}

impl FeedItem {
    pub fn new(id: String, meta: FeedMeta, content: String) -> Self {
        Self {
            file_name: id,
            meta,
            content,
            link: None,
        }
    }
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

fn render_content(input: &str, content_type: &FeedContentType) -> Result<String, String> {
    let md_options = ComrakOptions::default();

    match content_type {
        FeedContentType::Html => Ok(input.to_string()),
        FeedContentType::Markdown => {
            let rendered = markdown_to_html(&input, &md_options);
            Ok(rendered)
        }
    }
}

#[derive(Serialize)]
struct FeedIndex {
    pub feed_link: Option<PathBuf>,
    pub items: Vec<FeedItem>,
}

#[derive(Clone)]
pub struct Generator {
    files: HashMap<PathBuf, String>,
}

impl Generator {
    pub fn generate(
        config: &ServerConfig,
        template_engine: &mut TemplateEngine,
    ) -> Result<Self, String> {
        let mut files = HashMap::new();
        for feed_cfg in &config.feeds {
            let mut feed_items = Vec::<FeedItem>::new();

            let source_dir = config.root_dir.join(&feed_cfg.source_dir);
            if source_dir.is_dir() {
                for file in fs::read_dir(&source_dir).map_err(|err| err.to_string())? {
                    let file = file.map_err(|err| err.to_string())?;
                    let path = file.path();
                    if let Some(ext) = path.extension() {
                        if ext.to_string_lossy().to_string() == "md" {
                            let file_str =
                                fs::read_to_string(&path).map_err(|err| err.to_string())?;
                            if let Some((meta, content)) = file_str.split_once("___") {
                                let meta =
                                    serde_yaml::from_str::<FeedMeta>(&meta).map_err(|err| {
                                        format!(
                                            "Failed to deserialize file '{}': {}",
                                            path.display(),
                                            err.to_string(),
                                        )
                                    })?;
                                let html = render_content(content, &meta.content_type)?;

                                if let Some(file_name) = &path.file_stem() {
                                    feed_items.push(FeedItem::new(
                                        file_name.to_string_lossy().to_string(),
                                        meta,
                                        html,
                                    ));
                                } else {
                                    return Err(format!(
                                        "Failed to get file stem from path: {}",
                                        path.display()
                                    ));
                                }
                            } else {
                                return Err(format!(
                                    "Invalid syntax in '{}'. Make sure there is exactly one meta seperator!",
                                    path.display()
                                ));
                            }
                        }
                    }
                }
            }

            // Sort the feed ascending by date
            feed_items.sort_by(|a, b| b.meta.date.cmp(&a.meta.date));

            // Generate content
            if let Some(content_output) = &feed_cfg.content_output {
                let template_path = config.root_dir.join(&content_output.template);
                let content_template = fs::read_to_string(&template_path).map_err(|err| {
                    format!(
                        "Failed to load template file '{}': {}",
                        &template_path.display(),
                        err.to_string()
                    )
                })?;
                for feed_item in &mut feed_items {
                    let context =
                        Context::from_serialize(&feed_item).map_err(|err| err.to_string())?;
                    let rendered_content = template_engine
                        .render_string(&content_template, &context)
                        .map_err(|err| {
                            format!(
                                "Failed to render content template '{}': {}",
                                &content_output.template.display(),
                                err.to_string()
                            )
                        })?;
                    let link = content_output.link.join(feed_item.file_name.clone());
                    feed_item.link = Some(link.to_string_lossy().to_string());
                    files.insert(link, rendered_content);
                }
            }

            // Generate index
            if let Some(index_output) = &feed_cfg.index_output {
                let template_path = config.root_dir.join(&index_output.template);
                let index_template = fs::read_to_string(&template_path).map_err(|err| {
                    format!(
                        "Failed to load index template file '{}': {}",
                        &template_path.display(),
                        err.to_string()
                    )
                })?;

                let index = FeedIndex {
                    feed_link: feed_cfg.rss_feed_link.clone(),
                    items: feed_items.clone(),
                };
                let index_ctx = Context::from_serialize(&index).map_err(|err| err.to_string())?;
                let index_content = template_engine
                    .render_string(&index_template, &index_ctx)
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
            if let Some(rss_feed_link) = &feed_cfg.rss_feed_link {
                let index_output = &feed_cfg.index_output.clone().ok_or(format!(
                    "An index output is required to generate an RSS feed for '{}'!",
                    feed_cfg.title
                ))?;
                let rss_str = generate_rss_xml(
                    &feed_items,
                    &feed_cfg,
                    &config.server_name,
                    &index_output,
                    &rss_feed_link.to_string_lossy().to_string(),
                )?;
                files.insert(rss_feed_link.clone(), rss_str);
            }
        }
        Ok(Self { files })
    }

    pub fn get(&self, path: &PathBuf) -> Option<&String> {
        self.files.get(path)
    }
}
