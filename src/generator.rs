use crate::{config::ServerConfig, template_engine::TemplateEngine};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};
use tera::Context;

#[derive(Serialize, Deserialize)]
struct Link {
    content: String,
    url: String,
}

#[derive(Serialize, Deserialize)]
struct Image {
    alt: String,
    file_name: String,
}

#[derive(Serialize, Deserialize)]
struct Item {
    title: String,
    subtitle: String,
    #[serde(with = "date_format")]
    date: NaiveDate,
    date_label: String,
    tags: Vec<String>,
    image: Image,
    content: String,
    links: Vec<Link>,
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
struct ItemListings {
    pub list: Vec<(PathBuf, Item)>,
}

impl ItemListings {
    pub fn new() -> Self {
        Self { list: Vec::new() }
    }

    pub fn sort(&mut self) {
        // Sort ascending
        self.list.sort_by(|a, b| b.1.date.cmp(&a.1.date));
    }
}

impl Generator {
    pub fn generate(
        config: &ServerConfig,
        template_engine: &mut TemplateEngine,
    ) -> Result<Self, String> {
        let mut files = HashMap::new();
        for dir in &config.gen_dirs {
            let mut item_listsings = ItemListings::new();
            let template = fs::read_to_string(config.target_dir.join(&dir.content_template))
                .map_err(|err| format!("Failed to load template file: {}", err.to_string()))?;
            let index_template = fs::read_to_string(config.target_dir.join(&dir.index_template))
                .map_err(|err| {
                    format!("Failed to load index template file: {}", err.to_string())
                })?;
            let source_dir = config.target_dir.join(&dir.source_dir);
            if source_dir.is_dir() {
                for file in fs::read_dir(&source_dir).map_err(|err| err.to_string())? {
                    let file = file.map_err(|err| err.to_string())?;
                    let path = file.path();
                    if !path.is_dir() {
                        let content = fs::read_to_string(&path).map_err(|err| err.to_string())?;
                        let item = ron::from_str::<Item>(&content).map_err(|err| {
                            format!(
                                "Failed to deserialize file '{}': {}",
                                path.display(),
                                err.to_string(),
                            )
                        })?;
                        let context =
                            Context::from_serialize(&item).map_err(|err| err.to_string())?;
                        let result = template_engine.render_string(&template, &context)?;
                        if let Some(file_name) = &path.file_stem() {
                            let result_path = dir.target_url.join(PathBuf::from(file_name));
                            item_listsings.list.push((result_path.clone(), item));
                            files.insert(result_path, result);
                        } else {
                            return Err(format!(
                                "Failed to get file stem from path: {}",
                                path.display()
                            ));
                        }
                    }
                }
            }

            item_listsings.sort();
            let items_ctx =
                Context::from_serialize(item_listsings).map_err(|err| err.to_string())?;
            let index_content = template_engine.render_string(&index_template, &items_ctx)?;
            files.insert(dir.index_url.clone(), index_content);
        }
        Ok(Self { files })
    }

    pub fn get(&self, path: &PathBuf) -> Option<&String> {
        self.files.get(path)
    }
}
