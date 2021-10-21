mod config;
mod error_handler;
mod file_server;
mod generator;
mod rss;
mod template_engine;

use crate::{
    config::ServerConfig, error_handler::handle_errors, generator::Generator,
    template_engine::TemplateEngine,
};
use actix_web::{web, App, HttpServer};
use file_server::files;

const CONFIG_SUBDIR: &str = "webserver";
const SERVER_CONFIG_FILE: &str = "config.yaml";

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let config_dir = match dirs::config_dir() {
        Some(dir) => dir.join(CONFIG_SUBDIR),
        None => {
            eprintln!("Failed to get config dir!");
            std::process::exit(1);
        }
    };

    let config = match ServerConfig::load(config_dir.join(SERVER_CONFIG_FILE)) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Failed to load configuration file!");
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    };

    let mut template_engine = match TemplateEngine::load(&config) {
        Ok(template_engine) => template_engine,
        Err(err) => {
            eprintln!("Failed to load template engine!");
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    };

    let generator = match Generator::generate(&config, &mut template_engine) {
        Ok(generator) => generator,
        Err(err) => {
            eprintln!("Failed to generate feeds!");
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    };

    let socket = (config.address, config.port);

    HttpServer::new(move || {
        App::new()
            .data(config.clone())
            .data(template_engine.clone())
            .data(generator.clone())
            .default_service(web::get().to(files))
            .service(web::scope("").wrap(handle_errors()))
    })
    .bind(socket)?
    .run()
    .await
}
