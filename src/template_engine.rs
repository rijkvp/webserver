use std::path::PathBuf;

use crate::config::ServerConfig;
use tera::{Context, Tera};

#[derive(Clone)]
pub struct TemplateEngine {
    tera: Tera,
}

impl TemplateEngine {
    pub fn load(config: &ServerConfig) -> Result<Self, String> {
        let mut tera = Tera::new(&format!(
            "{}**/*.{}",
            config.target_dir.to_string_lossy(),
            config.content_ext
        ))
        .map_err(|err| format!("Failed to compile templates!\nParsing error(s): {}", err))?;

        tera.autoescape_on(vec![]);

        Ok(Self { tera })
    }

    // TODO: Add a way to reload
    pub fn _reload(&mut self) -> Result<(), String> {
        self.tera.full_reload().map_err(|err| err.to_string())
    }

    pub fn render_file(&self, path: PathBuf, context: &Context) -> Result<String, String> {
        if let Some(path_str) = path.to_str() {
            self.tera
                .render(path_str, context)
                .map_err(|err| format!("Template rendering error (File): {}", err.to_string()))
        } else {
            Err("Failed to convert path to string!".to_string())
        }
    }

    pub fn render_string(
        &mut self,
        template: &str,
        context: &Context,
    ) -> Result<String, String> {
        self
            .tera
            .render_str(template, context)
            .map_err(|err| format!("Template rendering error (String): {}", err.to_string()))
    }
}
