use crate::{config::ServerConfig, template_engine::TemplateEngine};
use rocket::{Catcher, Request, Response, catcher, http::{ContentType, Status}};
use std::io::Cursor;
use tera::Context;

#[derive(Clone)]
pub struct ErrorHandler {
    config: ServerConfig,
    template_engine: TemplateEngine,
}

#[rocket::async_trait]
impl catcher::Handler for ErrorHandler {
    async fn handle<'r>(&self, status: Status, _req: &'r Request<'_>) -> catcher::Result<'r> {
        let mut error_ctx = Context::new();
        error_ctx.insert("status_code", &status.code);
        error_ctx.insert("reason", status.reason_lossy());

        match self
            .template_engine
            .render_file(self.config.error_template.clone(), &error_ctx)
        {
            Ok(content) => Ok(Response::build()
                .header(ContentType::HTML)
                .sized_body(content.len(), Cursor::new(content))
                .status(status)
                .finalize()),
            Err(err) => {
                eprint!("Failed to render error page: {}", err);
                Ok(Response::build().status(status).finalize())
            }
        }
    }
}

impl ErrorHandler {
    pub fn new(config: ServerConfig, template_engine: TemplateEngine) -> Vec<Catcher> {
        vec![Catcher::new(
            None,
            ErrorHandler {
                config,
                template_engine,
            },
        )]
    }
}
