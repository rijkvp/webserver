#[macro_use]
extern crate rocket;

use async_recursion::async_recursion;
use rocket::{
    fs::NamedFile,
    http::Status,
    response::{
        content::{self, Html},
        Redirect,
    },
    tokio::{fs::File, io::AsyncReadExt},
    Request,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Responder)]
pub enum FileResponse {
    Content(Html<String>),
    Template(NamedFile),
    Redirect(Redirect),
}

const CONTENT_FOLDER: &str = "public/";
const INDEX_ORDER: [&str; 2] = ["index", "home"];
const EXTENSION_ORDER: [&str; 2] = ["rhc", "html"];

#[get("/<url..>")]
async fn files(url: PathBuf) -> Option<FileResponse> {
    let path = Path::new(CONTENT_FOLDER).join(url.clone());
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

#[get("/blog/<file..>")]
async fn blog(file: PathBuf) -> Option<Html<String>> {
    let file = File::open(Path::new("docs/").join(file).with_extension("md"))
        .await
        .ok();
    if let Some(mut file) = file {
        let mut contents = vec![];
        if let Some(_) = file.read_to_end(&mut contents).await.ok() {
            if let Ok(md) = String::from_utf8(contents) {
                let html = markdown::to_html(&md);
                println!("Generated HTML: {}", html);
                return Some(content::Html(html));
            }
        }
    }
    None
}

async fn read_file(path: &Path) -> Result<String, String> {
    let file = File::open(path).await;
    match file {
        Ok(mut file) => {
            let mut contents = vec![];
            match file.read_to_end(&mut contents).await {
                Ok(_) => {
                    if let Ok(text) = String::from_utf8(contents) {
                        return Ok(text);
                    } else {
                        return Err("Failed to convert to UTF8".to_string());
                    }
                }
                Err(err) => {
                    return Err(format!("Failed to read file: {}", err));
                }
            }
        }
        Err(err) => {
            return Err(format!(
                "Failed to open file!\nMessage: {}\nPath: {}",
                err,
                path.display()
            ));
        }
    }
}

#[async_recursion]
async fn concatenate_rhc(path: &Path, values: &HashMap<String, String>) -> Result<String, String> {
    match read_file(path).await {
        Ok(text) => {
            let mut fmt_text = text.to_string();
            loop {
                let mut found_start = false;
                let mut found_end = false;
                let mut start_index = 0;
                let mut end_index = 0;
                for (i, c1) in fmt_text.chars().enumerate() {
                    if c1 == '{' {
                        found_start = true;
                        start_index = i;
                        for (j, c2) in fmt_text[i..fmt_text.len()].chars().enumerate() {
                            if c2 == '}' {
                                found_end = true;
                                end_index = i + j;
                                break;
                            }
                        }
                        if !found_end {
                            return Err("Syntax error: no end character found!".to_string());
                        } else {
                            break;
                        }
                    }
                }
                if !found_start {
                    break;
                } else if found_start && found_end {
                    let key = &fmt_text[start_index + 1..end_index];
                    let first_char = key.chars().nth(0).unwrap();
                    let start = &fmt_text[0..start_index];
                    let end = &fmt_text[end_index + 1..fmt_text.len()];

                    if first_char == '@' {
                        let file_ref = &key[1..key.len()];
                        let nested_path = path.parent().unwrap().join(file_ref);
                        match concatenate_rhc(nested_path.as_path(), &values).await {
                            Ok(result) => {
                                fmt_text = start.to_string() + result.as_str() + &end;
                            }
                            Err(err) => {
                                return Err(format!(
                                    "Error while concatenating '{}'!\n{}",
                                    nested_path.display(),
                                    err
                                ));
                            }
                        }
                    } else {
                        if let Some(value) = values.get(key) {
                            fmt_text = start.to_string() + value + &end;
                        } else {
                            return Err(format!("Key '{}' not found!", key).to_string());
                        }
                    }
                }
            }
            return Ok(fmt_text);
        }
        Err(err) => {
            return Err(
                format!("Something went wrong while reading the file: {}", err).to_string(),
            );
        }
    }
}

fn get_error_msg(code: u16) -> &'static str {
    match code {
        404 => "Page not found!",
        500 => "Server error!",
        _ => "Other error",
    }
}

#[catch(default)]
async fn default_catcher(status: Status, _request: &Request<'_>) -> Html<String> {
    let mut values = HashMap::new();
    let error_code = status.code;
    let error_message = get_error_msg(error_code);
    values.insert("error_code".to_string(), status.code.to_string());
    values.insert("error_message".to_string(), error_message.to_string());
    let html_output = concatenate_rhc(Path::new("public/error.rhc"), &values).await;
    match html_output {
        Ok(html) => {
            return content::Html(html);
        }
        Err(err) => {
            println!("Error while formatting error page!\n{}\n", err);
            return content::Html(format!(
                "Error code: {}, message: {}",
                error_code, error_message
            ));
        }
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .register("/", catchers![default_catcher])
        .mount("/", routes![files])
        .mount("/blog", routes![blog])
}
