#[macro_use]
extern crate rocket;

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
use std::path::{Path, PathBuf};
use std::collections::HashMap;

#[derive(Debug, Responder)]
pub enum FileResponse {
    Content(Html<String>),
    Template(NamedFile),
    Redirect(Redirect),
}


const EXTENSION_ORDER: [&str; 2] = ["rhc", "html"];

#[get("/<path..>")]
async fn files(path: PathBuf) -> Option<FileResponse> {
    if path.to_str().unwrap().eq("") {
        let index_path = Path::new("index");
        return match read_content_path(&index_path).await {
            Ok(content) => Some(FileResponse::Content(content)),
            Err(err) => None,
        }
    } else {
        if let Some(ext) = path.extension() {
            // Check if content ext & redirect to right path
            if EXTENSION_ORDER.contains(&ext.to_str().unwrap()) {
                let cleaned_path = path.with_extension("");
                let path_string = format!("/{}", cleaned_path.to_str().unwrap());
                return Some(FileResponse::Redirect(Redirect::to(path_string)));
            }
            // Regular file
            let file = NamedFile::open(Path::new("public/").join(path)).await.ok();
            if let Some(file) = file {
                return Some(FileResponse::Template(file));
            }
        } else {
            // No extension - search for extension
            return match read_content_path(&path).await {
                Ok(content) => {
                    Some(FileResponse::Content(content))
                },
                Err(err) => None,
            }
        }
        None
    }
}

async fn read_content_path(path: &Path) -> Result<Html<String>, String> {
    for &ext in &EXTENSION_ORDER {
        let file_path = Path::new("public/").join(path.clone()).with_extension(ext);
        if file_path.exists() {
            return read_content_file(&file_path).await;
        }
    }
    Err(format!("No file found with path/url '{}'!", path.to_str().unwrap()))
}

async fn read_content_file(path: &Path) -> Result<Html<String>, String> {
    let ext = path.extension().unwrap().to_str().unwrap(); 
    return match ext {
        "html" => {
            match read_file(path).await {
                Ok(data) => Ok(content::Html(data)),
                Err(err) => Err(err),
            }
        }
        "rhc" => concatenate_rhc(path, HashMap::new()).await,
        _ => Err(format!("Unknown extension '{}'!", ext)),
    }
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
                println!("Genrated HTML: {}", html);
                return Some(content::Html(html));
            }
        }
    }
    None
}

async fn read_file(path: &Path) -> Result<String, String> {
    let file = File::open(path).await.ok();
    if let Some(mut file) = file {
        let mut contents = vec![];
        if let Some(_) = file.read_to_end(&mut contents).await.ok() { 
            if let Ok(text) =  String::from_utf8(contents) {
                return Ok(text);
            }
        }
    }
    Err("Something went wrong: TODO: better errors".to_string())
}

async fn concatenate_rhc(path: &Path, values: HashMap<String, String>) -> Result<content::Html<String>, String> {
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
                        }
                        else {
                            break;
                        }
                    }
                } 
                if !found_start {
                    break;
                } else if found_start && found_end {
                    let key = &fmt_text[start_index+1..end_index];
                    if let Some(value) = values.get(key) {
                        let start = &fmt_text[0..start_index];
                        let end = &fmt_text[end_index+1..fmt_text.len()];
                        fmt_text = start.to_string() + value + &end;
                    } else {
                        return Err(format!("Key '{}' not found!", key).to_string());
                    }
                }
            }
            return Ok(content::Html(fmt_text));
        },
        Err(err) => {
            return Err(format!("Something went wrong while reading the file: {}", err).to_string()); 
        }
    }
}

fn get_error_msg(code: u16) -> &'static str {
    match code {
        404 => { "Page not found!" },
        500 => { "Server error!" },
        _ => {
            "Other error"
        }
    }
}

#[catch(default)] async fn default_catcher(status: Status, _request: &Request<'_>) -> Html<String> {
    let mut values = HashMap::new();
    let error_code = status.code;
    let error_message = get_error_msg(error_code);
    values.insert("error_code".to_string(), status.code.to_string());
    values.insert("error_message".to_string(), error_message.to_string());
    let html_output = concatenate_rhc(Path::new("public/error.rhc"), values).await;
    match html_output {
        Ok(html) => {
            return html;
        },
        Err(err) => {
            println!("Error while formatting error page: {}", err);
            return content::Html(format!("Error code: {}, message: {}", error_code, error_message));
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
