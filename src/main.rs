#[macro_use]
extern crate rocket;

mod config;
// mod error_catcher;
mod file_server;
mod template_engine;

// use error_catcher::catch_error;
use file_server::files;

use crate::{config::ServerConfig, template_engine::TemplateEngine};

#[launch]
async fn rocket() -> _ {
    let config = ServerConfig::load()
        .await
        .expect("Error while reading config file!");

    let template_engine =
        TemplateEngine::load(&config).expect("Failed to initialize template engine!");

    rocket::build()
        .manage(config)
        .manage(template_engine)
        .mount("/", routes![files])
    // .register("/", catchers![catch_error])
}
