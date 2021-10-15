#[macro_use]
extern crate rocket;

mod config;
mod error_handler;
mod file_server;
mod generator;
mod rss;
mod template_engine;

use crate::{
    config::ServerConfig, error_handler::ErrorHandler, generator::Generator,
    template_engine::TemplateEngine,
};
use file_server::files;
use rocket::figment::{
    providers::{Env, Format, Toml},
    Figment,
};

const CONFIG_SUBDIR: &str = "webserver";
const ROCKET_CONFIG_FILE: &str = "rocket_config.toml";
const SERVER_CONFIG_FILE: &str = "server_config.ron";

#[launch]
fn rocket() -> _ {
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

    let figment = Figment::from(rocket::Config::default())
        .merge(Toml::file(config_dir.join(ROCKET_CONFIG_FILE)).nested())
        .merge(Env::prefixed("ROCKET_").global());

    rocket::custom(figment)
        .register(
            "/",
            ErrorHandler::new(config.clone(), template_engine.clone()),
        )
        .manage(config)
        .manage(template_engine)
        .manage(generator)
        .mount("/", routes![files])
}
