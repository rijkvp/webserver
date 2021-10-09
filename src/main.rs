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
    let config = match ServerConfig::load() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Failed to load configuration file!");
            eprintln!("Error: {}", err);
            std::process::exit(1);
        },
    };

    let mut template_engine = match TemplateEngine::load(&config) {
        Ok(template_engine) => template_engine,
        Err(err) => {
            eprintln!("Failed to load template engine!");
            eprintln!("Error: {}", err);
            std::process::exit(1);
        },
    };

    let generator = match  Generator::generate(&config, &mut template_engine){
        Ok(generator) => generator,
        Err(err) => {
            eprintln!("Failed to generate feeds!");
            eprintln!("Error: {}", err);
            std::process::exit(1);
        },
    };  

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
