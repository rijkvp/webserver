use rocket::{
    fs::NamedFile,
    http::Status,
    response::{
        content::{self, Html},
        Redirect,
    },
    State,
};
use std::path::{Path, PathBuf};
use tera::Context;

use crate::{config::ServerConfig, template_engine::TemplateEngine};

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
) -> FileResponse {
    let abs_path = Path::new(&config.target_dir).join(url_path.clone());

    // If url has an extension
    if let Some(ext) = url_path.extension() {
        // Redirect if url has content extension
        if ext.to_str().unwrap() == config.content_ext {
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
            let content_file = abs_path.with_extension(&config.content_ext);
            if content_file.exists() {
                Some(url_path.with_extension(&config.content_ext))
            } else if abs_path.is_dir() {
                let index_path = abs_path.join(&config.index_file);
                if index_path.exists() {
                    Some(url_path.join(&config.index_file))
                } else {
                    None
                }
            } else {
                None
            }
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
