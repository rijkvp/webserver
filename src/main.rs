#[macro_use]
extern crate rocket;

mod config;
mod error_handler;
mod file_server;
mod generator;
mod template_engine;

use file_server::files;

use crate::{
    config::ServerConfig, error_handler::ErrorHandler, generator::Generator,
    template_engine::TemplateEngine,
};

#[launch]
fn rocket() -> _ {
    let config = ServerConfig::load().expect("Error while reading config file!");

    let mut template_engine =
        TemplateEngine::load(&config).expect("Failed to initialize template engine!");

    let generator =
        Generator::generate(&config, &mut template_engine).expect("Failed to generate templates!");

    rocket::build()
        .register(
            "/",
            ErrorHandler::new(config.clone(), template_engine.clone()),
        )
        .manage(config)
        .manage(template_engine)
        .manage(generator)
        .mount("/", routes![files])
}
