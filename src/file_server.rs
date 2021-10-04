use crate::template_engine::TEMPLATES;
use rocket::{
    fs::NamedFile,
    http::Status,
    response::{
        content::{self, Html},
        Redirect,
    },
};
use std::path::{Path, PathBuf};
use tera::Context;

pub const CONTENT_FOLDER: &str = "public/";
const INDEX_ORDER: [&str; 2] = ["index", "home"];
const FILE_EXT: &str = "html";

#[derive(Debug, Responder)]
pub enum FileResponse {
    Content(Html<String>),
    File(NamedFile),
    Redirect(Redirect),
    StatusCode(Status),
}

#[get("/<url_path..>")]
pub async fn files(url_path: PathBuf) -> FileResponse {
    let path = Path::new(CONTENT_FOLDER).join(url_path.clone());
    // If path has an extension
    // Redirect if path is html file otherwise response with regular file
    if let Some(ext) = path.extension() {
        if ext == FILE_EXT {
            let cleaned_path = url_path.with_extension("");
            let path_string = format!("/{}", cleaned_path.to_str().unwrap());
            return FileResponse::Redirect(Redirect::to(path_string));
        }
        // Regular file
        if let Some(file) = NamedFile::open(path).await.ok() {
            return FileResponse::File(file);
        }
    }
    // If path has No extension
    // Search for content html file as extension and run template engine
    else {
        let file_path = path.with_extension(FILE_EXT);
        if file_path.exists() {
            return match read_content_file(&file_path).await {
                Ok(content) => FileResponse::Content(content::Html(content)),
                Err(err) => {
                    eprintln!("Error while trying to read content file!\n{}\n", err);
                    FileResponse::StatusCode(Status::InternalServerError)
                }
            };
        }

        if path.is_dir() {
            for index_name in INDEX_ORDER {
                let index_path = path.join(index_name).with_extension(FILE_EXT);
                if index_path.exists() {
                    return match read_content_file(&index_path).await {
                        Ok(content) => FileResponse::Content(content::Html(content)),
                        Err(err) => {
                            eprintln!("Error while trying to read content file!\n{}\n", err);
                            FileResponse::StatusCode(Status::InternalServerError)
                        }
                    };
                }
            }
        }
    }

    // Return 404 if nothing found
    FileResponse::StatusCode(Status::NotFound)
}

async fn read_content_file(path: &Path) -> Result<String, String> {
    let mut child_path = PathBuf::new();
    let mut parent = true;
    for node in path.iter() {
        if parent {
            parent = false;
        } else {
            child_path.push(node);
        }
    }
    return TEMPLATES
        .render(child_path.to_str().unwrap(), &Context::new())
        .map_err(|err| err.to_string());
}
