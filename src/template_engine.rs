use ::lazy_static::lazy_static;
use tera::Tera;

use crate::file_server::CONTENT_FOLDER;

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new(&(CONTENT_FOLDER.to_owned() + "**/*.html")) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed to compile templates!\nParsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![".html"]);
        tera
    };
}
