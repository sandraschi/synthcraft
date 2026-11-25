//! SEP-1865 MCP App support for Antigravity (`ui://synthcraft/dashboard.html`).

use serde_json::{json, Value};
use synthcraft_core::{Command, SynthEngine};

pub const APP_URI: &str = "ui://synthcraft/dashboard.html";
pub const APP_MIME: &str = "text/html;profile=mcp-app";
pub const TOOL_NAME: &str = "synthcraft_dashboard";

const EMBEDDED_HTML: &str = include_str!("dashboard.html");

pub fn resources_list() -> Value {
    json!({
        "resources": [
            {
                "uri": APP_URI,
                "name": "SynthCraft Dashboard",
                "description": "Interactive SynthCraft modular DSP synthesizer dashboard.",
                "mimeType": APP_MIME,
                "_meta": { "ui": { "prefersBorder": false } }
            }
        ]
    })
}

pub fn load_html() -> String {
    if let Ok(path) = std::env::var("SYNTHCRAFT_APP_HTML") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            if let Ok(html) = std::fs::read_to_string(trimmed) {
                return html;
            }
        }
    }
    EMBEDDED_HTML.to_string()
}

pub fn resource_read(uri: &str) -> Result<Value, (i32, String)> {
    if uri == APP_URI {
        Ok(json!({
            "contents": [
                {
                    "uri": APP_URI,
                    "mimeType": APP_MIME,
                    "text": load_html(),
                    "_meta": { "ui": { "prefersBorder": false } }
                }
            ]
        }))
    } else {
        Err((-32002, format!("Resource not found: {uri}")))
    }
}

pub fn tool_spec() -> Value {
    json!({
        "name": TOOL_NAME,
        "title": "SynthCraft",
        "description": "Open the interactive SynthCraft DSP synthesizer dashboard (status, patch browser, and engine state).",
        "inputSchema": { "type": "object", "properties": {} },
        "_meta": {
            "ui": { "resourceUri": APP_URI },
            "com.google.antigravity/ui": {
                "entrypoints": [
                    { "type": "aux_pane", "title": "SynthCraft" },
                    { "type": "global", "title": "SynthCraft" }
                ]
            }
        }
    })
}

pub fn call_dashboard(engine: &mut SynthEngine) -> Value {
    let status = Command::Status
        .dispatch(engine)
        .unwrap_or_else(|e| json!({ "error": e.to_string() }));
    let summary = Command::Summary
        .dispatch(engine)
        .unwrap_or_else(|e| json!({ "error": e.to_string() }));
    let patches_val = Command::ListPatches
        .dispatch(engine)
        .unwrap_or_else(|_| json!([]));
    let patches = if patches_val.is_array() {
        patches_val
    } else if let Some(arr) = patches_val.get("patches") {
        arr.clone()
    } else {
        json!([])
    };

    let patch_name = status["active_patch"]
        .as_str()
        .or_else(|| summary["active_patch"].as_str())
        .unwrap_or("default");
    let count = patches.as_array().map(Vec::len).unwrap_or(0);
    let text = format!("SynthCraft ready: active patch '{patch_name}', {count} patches available.");

    json!({
        "content": [{ "type": "text", "text": text }],
        "structuredContent": {
            "status": status,
            "summary": summary,
            "patches": patches
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthcraft_mcp_app_metadata_valid() {
        let spec = tool_spec();
        assert_eq!(spec["name"], TOOL_NAME);
        assert_eq!(spec["_meta"]["ui"]["resourceUri"], APP_URI);
        assert!(resource_read(APP_URI).is_ok());
    }
}
