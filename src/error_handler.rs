use crate::{config::ServerConfig, template_engine::TemplateEngine};
use actix_http::Response;
use actix_web::{
    dev::{Body, ServiceResponse},
    http::StatusCode,
    middleware::errhandlers::{ErrorHandlerResponse, ErrorHandlers},
    web::Data,
    Result,
};
use tera::Context;

pub fn handle_errors() -> ErrorHandlers<Body> {
    ErrorHandlers::new()
        .handler(StatusCode::NOT_FOUND, |res| {
            hanle_generic_error(res, StatusCode::NOT_FOUND)
        })
        .handler(StatusCode::INTERNAL_SERVER_ERROR, |res| {
            hanle_generic_error(res, StatusCode::INTERNAL_SERVER_ERROR)
        })
}

fn hanle_generic_error<B>(
    res: ServiceResponse<B>,
    code: StatusCode,
) -> Result<ErrorHandlerResponse<B>> {
    let response = get_error_response(&res, &code);
    Ok(ErrorHandlerResponse::Response(
        res.into_response(response.into_body()),
    ))
}

fn get_error_response<B>(res: &ServiceResponse<B>, status: &StatusCode) -> Response<Body> {
    let request = res.request();

    let mut error_ctx = Context::new();
    error_ctx.insert("status_code", &status.as_str());
    error_ctx.insert(
        "reason",
        status.canonical_reason().unwrap_or_else(|| "Unknown"),
    );

    let template_engine = request
        .app_data::<Data<TemplateEngine>>()
        .map(|t| t.get_ref());
    let config = request
        .app_data::<Data<ServerConfig>>()
        .map(|t| t.get_ref());

    if let (Some(template_engine), Some(config)) = (template_engine, config) {
        if let Some(error_template) = config.error_template.clone() {
            if let Ok(content) =
                template_engine.render_file(error_template, &error_ctx)
            {
                return Response::build(res.status())
                    .content_type("text/html")
                    .body(content);
            }
        }
    }
    Response::build(res.status())
        .content_type("text/plain")
        .body(status.to_string())
}
