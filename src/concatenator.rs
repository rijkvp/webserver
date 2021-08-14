use async_recursion::async_recursion;

use std::collections::HashMap;
use std::path::Path;

use crate::file_util::read_file;

#[async_recursion]
pub async fn concatenate_rhc(path: &Path, values: &HashMap<String, String>) -> Result<String, String> {
    match read_file(path).await {
        Ok(text) => {
            let mut fmt_text = text.to_string();
            loop {
                let mut found_start = false;
                let mut found_end = false;
                let mut start_index = 0;
                let mut end_index = 0;
                for (i, c1) in fmt_text.chars().enumerate() {
                    if c1 == '{' {
                        found_start = true;
                        start_index = i;
                        for (j, c2) in fmt_text[i..fmt_text.len()].chars().enumerate() {
                            if c2 == '}' {
                                found_end = true;
                                end_index = i + j;
                                break;
                            }
                        }
                        if !found_end {
                            return Err("Syntax error: no end character found!".to_string());
                        } else {
                            break;
                        }
                    }
                }
                if !found_start {
                    break;
                } else if found_start && found_end {
                    let key = &fmt_text[start_index + 1..end_index];
                    let first_char = key.chars().nth(0).unwrap();
                    let start = &fmt_text[0..start_index];
                    let end = &fmt_text[end_index + 1..fmt_text.len()];

                    if first_char == '@' {
                        let file_ref = &key[1..key.len()];
                        let nested_path = path.parent().unwrap().join(file_ref);
                        match concatenate_rhc(nested_path.as_path(), &values).await {
                            Ok(result) => {
                                fmt_text = start.to_string() + result.as_str() + &end;
                            }
                            Err(err) => {
                                return Err(format!(
                                    "Error while concatenating '{}'!\n{}",
                                    nested_path.display(),
                                    err
                                ));
                            }
                        }
                    } else {
                        if let Some(value) = values.get(key) {
                            fmt_text = start.to_string() + value + &end;
                        } else {
                            return Err(format!("Key '{}' not found!", key).to_string());
                        }
                    }
                }
            }
            return Ok(fmt_text);
        }
        Err(err) => {
            return Err(
                format!("Something went wrong while reading the file: {}", err).to_string(),
            );
        }
    }
}