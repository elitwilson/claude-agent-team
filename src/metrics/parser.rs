use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::Value;

/// Token usage for a single agent.
#[derive(Debug, Clone, Default)]
pub struct AgentUsage {
    pub role: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
}

/// Derive the project directory name from a cwd path.
/// Replaces all `/` with `-` to match Claude's project directory naming convention.
/// e.g. `/Users/charlo/dev/myproject` → `-Users-charlo-dev-myproject`
pub fn derive_project_dir(cwd: &str) -> String {
    cwd.replace('/', "-")
}

/// Determine if a JSONL filename is a sub-agent file (starts with `agent-`).
pub fn is_agent_file(filename: &str) -> bool {
    filename.starts_with("agent-")
}

/// Attribute a role to an agent based on the first user message content.
/// Returns "coder" if content contains "Coder" (case-insensitive),
/// "reviewer" if it contains "Reviewer", otherwise None.
pub fn attribute_role(first_user_message: &str) -> Option<String> {
    let lower = first_user_message.to_lowercase();
    if lower.contains("coder") {
        Some("coder".to_string())
    } else if lower.contains("reviewer") {
        Some("reviewer".to_string())
    } else {
        None
    }
}

/// Extract token fields from a parsed assistant message's usage object.
/// Returns (input_tokens, output_tokens, cache_creation_tokens, cache_read_tokens).
pub fn extract_tokens(usage: &Value) -> (u64, u64, u64, u64) {
    let input = usage["input_tokens"].as_u64().unwrap_or(0);
    let output = usage["output_tokens"].as_u64().unwrap_or(0);
    let cache_creation = usage["cache_creation_input_tokens"].as_u64().unwrap_or(0);
    let cache_read = usage["cache_read_input_tokens"].as_u64().unwrap_or(0);
    (input, output, cache_creation, cache_read)
}

/// Discover JSONL files in the Claude projects directory for the given project.
pub fn discover_jsonl_files(project_dir: &str) -> Result<Vec<PathBuf>> {
    let base = dirs_home()?.join(".claude/projects").join(project_dir);
    if !base.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    for entry in std::fs::read_dir(&base).context("Failed to read Claude projects directory")? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".jsonl") {
                    files.push(entry.path());
                }
            }
        }
    }
    files.sort();
    Ok(files)
}

/// Parse JSONL files, filter by started_at timestamp, extract token usage per agent.
pub fn collect_agent_usage(jsonl_files: &[PathBuf], started_at: &str) -> Result<Vec<AgentUsage>> {
    let mut usages = Vec::new();
    let mut agent_counter = 0u32;

    for path in jsonl_files {
        let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("");

        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        let mut input_tokens = 0u64;
        let mut output_tokens = 0u64;
        let mut cache_creation = 0u64;
        let mut cache_read = 0u64;
        let mut first_user_message: Option<String> = None;

        for line in contents.lines() {
            let msg: Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            // Filter by timestamp
            let timestamp = msg["timestamp"].as_str().unwrap_or("");
            if timestamp < started_at {
                continue;
            }

            let msg_type = msg["type"].as_str().unwrap_or("");

            // Capture first user message for role attribution
            if msg_type == "user" && first_user_message.is_none() {
                if let Some(content) = msg["message"]["content"].as_str() {
                    first_user_message = Some(content.to_string());
                }
            }

            // Extract tokens from assistant messages
            if msg_type == "assistant" {
                if let Some(usage) = msg["message"]["usage"].as_object() {
                    let usage_val = Value::Object(usage.clone());
                    let (i, o, cc, cr) = extract_tokens(&usage_val);
                    input_tokens += i;
                    output_tokens += o;
                    cache_creation += cc;
                    cache_read += cr;
                }
            }
        }

        // Determine role
        let role = if is_agent_file(filename) {
            if let Some(ref msg) = first_user_message {
                attribute_role(msg).unwrap_or_else(|| {
                    agent_counter += 1;
                    format!("agent_{agent_counter}")
                })
            } else {
                agent_counter += 1;
                format!("agent_{agent_counter}")
            }
        } else {
            "orchestrator".to_string()
        };

        usages.push(AgentUsage {
            role,
            input_tokens,
            output_tokens,
            cache_creation_tokens: cache_creation,
            cache_read_tokens: cache_read,
        });
    }

    Ok(usages)
}

fn dirs_home() -> Result<PathBuf> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .context("HOME environment variable not set")
}

#[cfg(test)]
mod tests;
