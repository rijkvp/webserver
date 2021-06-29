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

#[derive(Debug, Responder)]
pub enum FileResponse {
    Template(NamedFile),
    Redirect(Redirect),
}

#[get("/<path..>")]
async fn files(path: PathBuf) -> Option<FileResponse> {
    if path.to_str().unwrap().eq("") {
        // Index
        let file = NamedFile::open(Path::new("public/").join("index").with_extension("html"))
            .await
            .ok();
        if let Some(file) = file {
            return Some(FileResponse::Template(file));
        }
        None
    } else {
        if let Some(extension) = path.extension() {
            if extension == "html" {
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
            // No extension - add html
            let file = NamedFile::open(Path::new("public/").join(path).with_extension("html"))
                .await
                .ok();
            if let Some(file) = file {
                return Some(FileResponse::Template(file));
            }
        }
        None
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

#[catch(default)]
fn default_catcher(status: Status, request: &Request) -> String { /* .. */
    format!("{:?}", status)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .register("/", catchers![default_catcher])
        .mount("/", routes![files])
        .mount("/blog", routes![blog])
}
