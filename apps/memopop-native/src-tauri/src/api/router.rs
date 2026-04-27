use serde_json::Value;

use super::{queries, ApiError};

#[tauri::command]
pub async fn api_dispatch(
    method: String,
    path: String,
    body: Option<Value>,
) -> Result<Value, ApiError> {
    match (method.as_str(), path.as_str()) {
        ("GET", "/firms") => {
            let repo_path = body
                .as_ref()
                .and_then(|b| b.get("repoPath"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| ApiError::validation("repoPath required"))?;
            queries::list_firms(repo_path).await
        }

        _ => Err(ApiError::not_found(&path)),
    }
}
