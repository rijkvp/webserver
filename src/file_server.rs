use rocket::{
    fs::NamedFile,
    response::{
        content::{self, Html},
        Redirect,
    },
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::{concatenator::concatenate_rhc, file_util::read_file};

#[derive(Debug, Responder)]
pub enum FileResponse {
    Content(Html<String>),
    Template(NamedFile),
    Redirect(Redirect),
}

const CONTENT_FOLDER: &str = "public/";
const INDEX_ORDER: [&str; 2] = ["index", "home"];
const EXTENSION_ORDER: [&str; 2] = ["rhc", "html"];

#[get("/<url_path..>")]
pub async fn files(url_path: PathBuf) -> Option<FileResponse> {
    let path = Path::new(CONTENT_FOLDER).join(url_path.clone());
    // If path has an extension 
    // Redirect if path is an rhc or html file otherwise response with regular file
    if let Some(ext) = path.extension() {
        if EXTENSION_ORDER.contains(&ext.to_str().unwrap()) {
            let cleaned_path = path.with_extension("");
            let path_string = format!("/{}", cleaned_path.to_str().unwrap());
            return Some(FileResponse::Redirect(Redirect::to(path_string)));
        }
        // Regular file
        if let Some(file) = NamedFile::open(path).await.ok() {
            return Some(FileResponse::Template(file));
        }
    }
    // If path has No extension 
    // Search for content (rhc, html) file as extension
    else {
        for ext in EXTENSION_ORDER {
            let file_path = path.with_extension(ext);
            if file_path.exists() {
                return match read_content_file(&file_path).await {
                    Ok(content) => Some(FileResponse::Content(content::Html(content))),
                    Err(err) => {
                        println!("Error while trying to read content file!\n{}\n", err);
                        None
                    }
                };
            }
        }

        if path.is_dir() {
            for index_name in INDEX_ORDER {
                for ext in EXTENSION_ORDER {
                    let index_path = path.join(index_name).with_extension(ext);
                    if index_path.exists() {
                        return match read_content_file(&index_path).await {
                            Ok(content) => Some(FileResponse::Content(content::Html(content))),
                            Err(err) => {
                                println!("Error while trying to read content file!\n{}\n", err);
                                None
                            }
                        };
                    }
                }
            }
        }
    }
    None
}

async fn read_content_file(path: &Path) -> Result<String, String> {
    let ext = path.extension().unwrap().to_str().unwrap();
    return match ext {
        "html" => match read_file(path).await {
            Ok(data) => Ok(data),
            Err(err) => Err(err),
        },
        "rhc" => concatenate_rhc(path, &HashMap::new()).await,
        _ => Err(format!("Unknown extension '{}'!", ext)),
    };
}

