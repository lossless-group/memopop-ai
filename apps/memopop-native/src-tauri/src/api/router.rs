use serde_json::Value;

use super::{actions, queries, sidecar, ApiError};

#[tauri::command]
pub async fn api_dispatch(
    app: tauri::AppHandle,
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

        // Stop the Python sidecar process. Goes directly to the SidecarManager
        // — it does NOT round-trip through the FastAPI server, because the whole
        // point of "stop" is to handle the case where Python is unresponsive
        // (rate-limited, deadlocked, in a runaway retry loop). SIGKILL is the
        // promise here, not a polite request.
        ("POST", "/actions/stop-sidecar") => {
            let manager = sidecar::manager(&app);
            manager.shutdown();
            Ok(serde_json::json!({"stopped": true}))
        }

        // Brand fetch + save — forwarded to the Python sidecar. fetch-brand drives
        // a Claude tool-use loop (~10–20s) reading the firm's website; save-brand
        // writes the user-confirmed config to disk.
        ("POST", "/actions/fetch-brand") | ("POST", "/actions/save-brand") => {
            forward_to_sidecar(&app, &body, &method, &path).await
        }

        // --- Sidecar-forwarded routes (FastAPI orchestrator API) ---
        // The sidecar is lazy-spawned on the first /memos call. SSE streaming
        // (`/memos/{id}/events`) intentionally goes direct from the webview to
        // localhost:8765 — only JSON routes pass through here.
        ("POST", "/memos") | ("GET", "/memos") => {
            forward_to_sidecar(&app, &body, &method, &path).await
        }

        // POST /memos/resume — pick up an interrupted run from the last
        // on-disk checkpoint. Same job machinery as a fresh run; different
        // worker entry point on the Python side.
        ("POST", "/memos/resume") => {
            forward_to_sidecar(&app, &body, &method, &path).await
        }

        ("GET", p)
            if p.starts_with("/memos/")
                && !p.ends_with("/events") =>
        {
            forward_to_sidecar(&app, &body, &method, p).await
        }

        _ => Err(ApiError::not_found(&format!("{} {}", method, path))),
    }
}

async fn forward_to_sidecar(
    app: &tauri::AppHandle,
    body: &Option<Value>,
    method: &str,
    path: &str,
) -> Result<Value, ApiError> {
    let repo_path = require_string(body, "repoPath")?.to_string();
    let manager = sidecar::manager(app);
    manager.ensure_running(app, &repo_path).await?;

    // Strip the dispatcher-only `repoPath` field before forwarding — the FastAPI
    // sidecar doesn't expect it and Pydantic won't gracefully ignore unknown keys
    // unless the model declares `extra=allow` (which we deliberately don't).
    let forward_body = body.as_ref().map(|b| {
        if let Value::Object(map) = b {
            let mut filtered = map.clone();
            filtered.remove("repoPath");
            Value::Object(filtered)
        } else {
            b.clone()
        }
    });

    manager.forward(method, path, forward_body).await
}

fn require_string<'a>(body: &'a Option<Value>, key: &str) -> Result<&'a str, ApiError> {
    body.as_ref()
        .and_then(|b| b.get(key))
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::validation(format!("{} required", key)))
}
