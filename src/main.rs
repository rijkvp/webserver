#[macro_use]
extern crate rocket;

mod error_catcher;
mod file_server;
mod template_engine;

use error_catcher::catch_error;
use file_server::files;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![files])
        .register("/", catchers![catch_error])
}
