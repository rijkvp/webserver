use crate::{config::ServerConfig, generator::Generator, template_engine::TemplateEngine};
use actix_files::{file_extension_to_mime, NamedFile};
use actix_web::{http::header::ContentType, web, HttpRequest, HttpResponse};
use std::{ffi::OsString, path::PathBuf};
use tera::Context;

pub async fn files(
    req: HttpRequest,
    config: web::Data<ServerConfig>,
    template_engine: web::Data<TemplateEngine>,
    generator: web::Data<Generator>,
) -> HttpResponse {
    let uri_path_str = req.uri().path().to_string();
    let uri_path = PathBuf::from(&uri_path_str[1..]);

    // Check if file doesn't start with an ignored path
    for dir in &config.ignored_paths {
        if uri_path.starts_with(dir) {
            return HttpResponse::NotFound().finish();
        }
    }

    // Check if url is a generated template
    if let Some(content) = generator.get(&uri_path) {
        let mime_type: mime::Mime = {
            if let Some(ext) = uri_path.extension() {
                if let Some(ext_str) = ext.to_str() {
                    file_extension_to_mime(&ext_str)
                } else {
                    mime::TEXT_PLAIN
                }
            } else {
                mime::TEXT_HTML
            }
        };
        return HttpResponse::Ok().set(ContentType(mime_type)).body(content);
    }

    let abs_path = config.root_dir.join(uri_path.clone());

    // If url has an extension
    if let Some(ext) = uri_path.extension() {
        // Redirect if url has content extension
        if ext == OsString::from(&config.content_ext) {
            let clean_url = uri_path.with_extension("");
            let url_string = format!("/{}", clean_url.display());
            return HttpResponse::PermanentRedirect()
                .set_header("Location", url_string)
                .finish();
        }
        // Regular file
        if let Some(file) = NamedFile::open(abs_path).ok() {
            if let Ok(response) = file.into_response(&req) {
                return response;
            } else {
                return HttpResponse::BadRequest().finish();
            }
        }
    }
    // If url has no extension
    else {
        // Return rendered file if exists
        if let Some(relative_path) = {
            let mut result = None;
            let content_file = abs_path.with_extension(&config.content_ext);
            if content_file.exists() {
                result = Some(uri_path.with_extension(&config.content_ext))
            } else if abs_path.is_dir() {
                let index_path = abs_path.join(&config.index);
                if index_path.exists() {
                    result = Some(uri_path.join(&config.index))
                }
            }
            result
        } {
            return match template_engine.render_file(relative_path, &Context::new()) {
                Ok(content) => HttpResponse::Ok().content_type("text/html").body(content),
                Err(err) => {
                    eprintln!("Error while rendering file!\n{}", err);
                    HttpResponse::InternalServerError().finish()
                }
            };
        }
    }

    // Return a 404 if nothing found
    HttpResponse::NotFound().finish()
}
