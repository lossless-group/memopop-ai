use std::path::Path;

use serde_json::{json, Value};

use super::ApiError;

pub async fn list_firms(repo_path: &str) -> Result<Value, ApiError> {
    let io_dir = Path::new(repo_path).join("io");
    if !io_dir.is_dir() {
        return Ok(json!({ "firms": [] }));
    }

    let mut firms: Vec<String> = Vec::new();

    let entries = std::fs::read_dir(&io_dir)
        .map_err(|e| ApiError::internal(format!("read_dir({}): {}", io_dir.display(), e)))?;

    for entry in entries {
        let entry = entry
            .map_err(|e| ApiError::internal(format!("entry: {}", e)))?;

        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }

        let name = match entry.file_name().into_string() {
            Ok(s) => s,
            Err(_) => continue,
        };

        if name.starts_with('.') {
            continue;
        }

        firms.push(name);
    }

    firms.sort();
    Ok(json!({ "firms": firms }))
}
