use crate::template_engine::TEMPLATES;
use rocket::{
    http::Status,
    response::content::{self, Html},
    Request,
};
use std::path::Path;
use tera::Context;

#[catch(default)]
pub async fn catch_error(status: Status, _request: &Request<'_>) -> (Status, Html<String>) {
    let mut context = Context::new();
    context.insert("status_code", &status.code);
    context.insert("reason", status.reason_lossy());
    let path = Path::new("error.html").to_str().unwrap();
    let rendered_html = TEMPLATES.render(&path, &context);
    match rendered_html {
        Ok(html) => {
            return (status, content::Html(html));
        }
        Err(err) => {
            eprintln!("Error while formatting error page!\n{}\n", err);
            return (
                Status::InternalServerError,
                content::Html(format!(
                    "There was an internal error while processing your error :(<br>Original error: {}, {}",
                    status.code,
                    status.reason_lossy()
                )),
            );
        }
    }
}
