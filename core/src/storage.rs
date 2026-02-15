//! Persistent app config (JSON in user data dir).
//!
//! Loads and saves config.json; used for window state, theme preference, and
//! generic key-value storage (ReadConfig/WriteConfig).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

use crate::paths::user_data_dir;

const CONFIG_FILENAME: &str = "config.json";

/// Window bounds for persistence (physical position and size).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Root config structure. Extensible via generic key-value map.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<WindowBounds>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(flatten)]
    pub data: HashMap<String, serde_json::Value>,
}

fn config_path() -> std::path::PathBuf {
    user_data_dir().join(CONFIG_FILENAME)
}

/// Loads config from user data dir. Returns default on missing or parse error.
#[must_use]
pub fn load_config() -> AppConfig {
    let path = config_path();
    let Ok(content) = fs::read_to_string(&path) else {
        return AppConfig::default();
    };
    serde_json::from_str(&content).unwrap_or_default()
}

/// Saves config to user data dir. Logs and ignores errors.
pub fn save_config(config: &AppConfig) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(config) {
        let _ = fs::write(&path, json);
    } else {
        log::warn!("Failed to serialize config");
    }
}

/// Reads a single key from config.
#[must_use]
#[allow(dead_code)]
pub fn get_value(key: &str) -> Option<serde_json::Value> {
    let mut config = load_config();
    config.data.remove(key)
}

/// Writes a single key-value pair into config and persists.
pub fn set_value(key: String, value: serde_json::Value) {
    let mut config = load_config();
    config.data.insert(key, value);
    save_config(&config);
}

/// Saves window bounds and persists.
pub fn save_window_bounds(x: i32, y: i32, width: u32, height: u32) {
    let mut config = load_config();
    config.window = Some(WindowBounds {
        x,
        y,
        width,
        height,
    });
    save_config(&config);
}

/// Returns saved window bounds if any.
#[must_use]
pub fn load_window_bounds() -> Option<WindowBounds> {
    load_config().window
}

/// Returns the full config as a JSON-serializable object for ReadConfig.
#[must_use]
pub fn get_full_config() -> serde_json::Value {
    let config = load_config();
    let mut obj = serde_json::Map::new();
    if let Some(ref w) = config.window {
        obj.insert(
            "window".to_string(),
            serde_json::json!({
                "x": w.x,
                "y": w.y,
                "width": w.width,
                "height": w.height
            }),
        );
    }
    if let Some(ref t) = config.theme {
        obj.insert("theme".to_string(), serde_json::json!(t));
    }
    for (k, v) in config.data {
        obj.insert(k, v);
    }
    serde_json::Value::Object(obj)
}
