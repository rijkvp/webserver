use crate::{config::ServerConfig, template_engine::TemplateEngine};
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
    image: Image,
    content: String,
    links: Vec<Link>,
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
        for dir in &config.gen_dirs {
            let template = fs::read_to_string(config.target_dir.join(&dir.template_file))
                .map_err(|err| format!("Failed to load template file: {}", err.to_string()))?;
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
                            Context::from_serialize(item).map_err(|err| err.to_string())?;
                        let result = template_engine.render_string(&template, &context)?;
                        if let Some(file_name) = &path.file_stem() {
                            let result_path = dir.target_dir.join(PathBuf::from(file_name));
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
        }
        Ok(Self { files })
    }

    pub fn get(&self, path: &PathBuf) -> Option<&String> {
        println!("GET {}", path.display());
        for (p, s) in &self.files {
            println!("P {} S {}", p.display(), s);
        }
        self.files.get(path)
    }
}
