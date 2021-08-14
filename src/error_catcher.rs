extern crate rocket;

use rocket::{
    http::Status,
    response::{
        content::{self, Html},
    },
    Request,
};
use std::collections::HashMap;
use std::path::Path;

use crate::concatenator::concatenate_rhc;

#[catch(default)]
pub async fn catch_error(status: Status, _request: &Request<'_>) -> Html<String> {
    let mut values = HashMap::new();
    let error_code = status.code;
    let error_message = status.reason().unwrap();
    values.insert("error_code".to_string(), status.code.to_string());
    values.insert("error_message".to_string(), error_message.to_string());
    let html_output = concatenate_rhc(Path::new("public/error.rhc"), &values).await;
    match html_output {
        Ok(html) => {
            return content::Html(html);
        }
        Err(err) => {
            println!("Error while formatting error page!\n{}\n", err);
            return content::Html(format!(
                "Error code: {}, message: {}",
                error_code, error_message
            ));
        }
    }
}