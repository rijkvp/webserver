use rocket::{
    tokio::{fs::File, io::AsyncReadExt},
};
use std::path::Path;

pub async fn read_file(path: &Path) -> Result<String, String> {
    let file = File::open(path).await;
    match file {
        Ok(mut file) => {
            let mut contents = vec![];
            match file.read_to_end(&mut contents).await {
                Ok(_) => {
                    if let Ok(text) = String::from_utf8(contents) {
                        return Ok(text);
                    } else {
                        return Err("Failed to convert to UTF8".to_string());
                    }
                }
                Err(err) => {
                    return Err(format!("Failed to read file: {}", err));
                }
            }
        }
        Err(err) => {
            return Err(format!(
                "Failed to open file!\nMessage: {}\nPath: {}",
                err,
                path.display()
            ));
        }
    }
}