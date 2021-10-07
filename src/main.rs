#[macro_use]
extern crate rocket;

mod config;
mod error_handler;
mod file_server;
mod template_engine;

use file_server::files;

use crate::{config::ServerConfig, error_handler::ErrorHandler, template_engine::TemplateEngine};

#[launch]
async fn rocket() -> _ {
    let config = ServerConfig::load()
        .await
        .expect("Error while reading config file!");

    let template_engine =
        TemplateEngine::load(&config).expect("Failed to initialize template engine!");

    rocket::build()
        .register(
            "/",
            ErrorHandler::new(config.clone(), template_engine.clone()),
        )
        .manage(config)
        .manage(template_engine)
        .mount("/", routes![files])
}
