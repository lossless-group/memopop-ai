use serde_json::Value;

use super::{actions, queries, ApiError};

#[tauri::command]
pub async fn api_dispatch(
    method: String,
    path: String,
    body: Option<Value>,
) -> Result<Value, ApiError> {
    match (method.as_str(), path.as_str()) {
        // --- Queries ---
        ("GET", "/firms") => {
            let repo_path = require_string(&body, "repoPath")?;
            queries::list_firms(repo_path).await
        }

        ("GET", "/outlines") => {
            let repo_path = require_string(&body, "repoPath")?;
            queries::list_outlines(repo_path).await
        }

        ("GET", p) if p.starts_with("/outlines/") => {
            let repo_path = require_string(&body, "repoPath")?;
            let id = p.trim_start_matches("/outlines/");
            if id.is_empty() {
                return Err(ApiError::validation("outline id required"));
            }
            queries::get_outline_detail(repo_path, id).await
        }

        // --- Actions ---
        ("POST", "/actions/create-firm") => {
            let repo_path = require_string(&body, "repoPath")?;
            let conventional_name = require_string(&body, "conventionalName")?;
            actions::create_firm(repo_path, conventional_name).await
        }

        _ => Err(ApiError::not_found(&format!("{} {}", method, path))),
    }
}

fn require_string<'a>(body: &'a Option<Value>, key: &str) -> Result<&'a str, ApiError> {
    body.as_ref()
        .and_then(|b| b.get(key))
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::validation(format!("{} required", key)))
}
