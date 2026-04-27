use std::path::Path;

use serde::Serialize;
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

#[derive(Debug, Serialize)]
struct OutlineSummary {
    id: String,
    title: String,
    outline_type: String,
    type_label: String,
    description: String,
    section_count: usize,
    compatible_modes: Vec<String>,
    firm: Option<String>,
    version: Option<String>,
}

pub async fn list_outlines(repo_path: &str) -> Result<Value, ApiError> {
    let outlines_dir = Path::new(repo_path).join("templates").join("outlines");
    if !outlines_dir.is_dir() {
        return Err(ApiError {
            status: 404,
            code: "outlines_dir_missing".into(),
            message: format!(
                "No templates/outlines directory at {}. Did you pick the orchestrator repo root?",
                repo_path
            ),
        });
    }

    let mut outlines: Vec<OutlineSummary> = Vec::new();

    let entries = std::fs::read_dir(&outlines_dir)
        .map_err(|e| ApiError::internal(format!("read_dir({}): {}", outlines_dir.display(), e)))?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if extension != "yaml" && extension != "yml" {
            continue;
        }

        let stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };

        match parse_outline(&path, &stem) {
            Ok(summary) => outlines.push(summary),
            Err(_) => continue, // skip unparseable outlines silently
        }
    }

    outlines.sort_by(|a, b| a.title.cmp(&b.title));
    Ok(json!({ "outlines": outlines }))
}

pub async fn get_outline_detail(repo_path: &str, id: &str) -> Result<Value, ApiError> {
    let outlines_dir = Path::new(repo_path).join("templates").join("outlines");
    let path = outlines_dir.join(format!("{}.yaml", id));

    let path = if path.exists() {
        path
    } else {
        let yml_path = outlines_dir.join(format!("{}.yml", id));
        if yml_path.exists() {
            yml_path
        } else {
            return Err(ApiError {
                status: 404,
                code: "outline_not_found".into(),
                message: format!("Outline '{}' not found in {}", id, outlines_dir.display()),
            });
        }
    };

    let content = std::fs::read_to_string(&path)
        .map_err(|e| ApiError::internal(format!("read({}): {}", path.display(), e)))?;

    let value: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| ApiError::internal(format!("yaml parse {}: {}", path.display(), e)))?;

    let json_value = serde_yaml_to_json(value);
    let summary = parse_outline(&path, path.file_stem().unwrap().to_str().unwrap())
        .map_err(|e| ApiError::internal(e))?;

    Ok(json!({
        "summary": summary,
        "raw": json_value,
    }))
}

fn parse_outline(path: &Path, stem: &str) -> Result<OutlineSummary, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("read: {}", e))?;

    let value: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| format!("yaml: {}", e))?;

    let metadata = value.get("metadata");

    let outline_type = metadata
        .and_then(|m| m.get("outline_type"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let type_label = match outline_type.as_str() {
        "direct_investment" => "Direct Investment".to_string(),
        "fund_commitment" => "Fund Commitment".to_string(),
        other => humanize(other),
    };

    let description = metadata
        .and_then(|m| m.get("description"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let firm = metadata
        .and_then(|m| m.get("firm"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let version = metadata
        .and_then(|m| m.get("version"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let compatible_modes = metadata
        .and_then(|m| m.get("compatible_modes"))
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_else(Vec::new);

    let section_count = value
        .get("sections")
        .and_then(|s| s.as_sequence())
        .map(|seq| seq.len())
        .unwrap_or(0);

    let title = humanize(stem);

    Ok(OutlineSummary {
        id: stem.to_string(),
        title,
        outline_type,
        type_label,
        description,
        section_count,
        compatible_modes,
        firm,
        version,
    })
}

fn humanize(slug: &str) -> String {
    slug.replace('-', " ")
        .replace('_', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn serde_yaml_to_json(value: serde_yaml::Value) -> Value {
    match value {
        serde_yaml::Value::Null => Value::Null,
        serde_yaml::Value::Bool(b) => Value::Bool(b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                json!(i)
            } else if let Some(f) = n.as_f64() {
                json!(f)
            } else {
                Value::Null
            }
        }
        serde_yaml::Value::String(s) => Value::String(s),
        serde_yaml::Value::Sequence(seq) => Value::Array(seq.into_iter().map(serde_yaml_to_json).collect()),
        serde_yaml::Value::Mapping(map) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                let key = match k {
                    serde_yaml::Value::String(s) => s,
                    other => serde_yaml::to_string(&other).unwrap_or_default(),
                };
                obj.insert(key, serde_yaml_to_json(v));
            }
            Value::Object(obj)
        }
        serde_yaml::Value::Tagged(tagged) => serde_yaml_to_json(tagged.value),
    }
}
