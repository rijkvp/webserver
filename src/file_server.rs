use rocket::{
    fs::NamedFile,
    http::Status,
    response::{
        content::{self, Html},
        Redirect,
    },
    State,
};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};
use tera::Context;

use crate::{config::ServerConfig, generator::Generator, template_engine::TemplateEngine};

#[derive(Debug, Responder)]
pub enum FileResponse {
    Content(Html<String>),
    File(NamedFile),
    Redirect(Redirect),
    StatusCode(Status),
}

#[get("/<url_path..>")]
pub async fn files(
    url_path: PathBuf,
    config: &State<ServerConfig>,
    template_engine: &State<TemplateEngine>,
    generator: &State<Generator>,
) -> FileResponse {
    // Check if file doesn't start with an ignored path
    for dir in &config.ignored_paths {
        if url_path.starts_with(dir) {
            return FileResponse::StatusCode(Status::NotFound);
        }
    }

    // Check if url is a generated template
    if let Some(content) = generator.get(&url_path) {
        return FileResponse::Content(content::Html(content.to_string()));
    }

    let abs_path = Path::new(&config.target_dir).join(url_path.clone());

    // If url has an extension
    if let Some(ext) = url_path.extension() {
        // Redirect if url has content extension
        if ext == OsString::from(&config.content_ext) {
            let clean_url = url_path.with_extension("");
            let url_string = format!("/{}", clean_url.display());
            return FileResponse::Redirect(Redirect::to(url_string));
        }
        // Regular file
        if let Some(file) = NamedFile::open(abs_path).await.ok() {
            return FileResponse::File(file);
        }
    }
    // If url has no extension
    else {
        // Return rendered file if exists
        if let Some(relative_path) = {
            let mut result = None;
            let content_file = abs_path.with_extension(&config.content_ext);
            if content_file.exists() {
                result = Some(url_path.with_extension(&config.content_ext))
            } else if abs_path.is_dir() {
                let index_path = abs_path.join(&config.index_file);
                if index_path.exists() {
                    result = Some(url_path.join(&config.index_file))
                }
            }
            result
        } {
            return match template_engine.render_file(relative_path, &Context::new()) {
                Ok(content) => FileResponse::Content(content::Html(content)),
                Err(err) => {
                    eprintln!("Error while rendering file!\n{}", err);
                    FileResponse::StatusCode(Status::InternalServerError)
                }
            };
        }
    }

    // Return a 404 if nothing found
    FileResponse::StatusCode(Status::NotFound)
}
