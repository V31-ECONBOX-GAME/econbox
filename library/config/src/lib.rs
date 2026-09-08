use bevy::prelude::*;
use serde::Deserialize;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Resource, Deserialize, Debug, Clone, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct AppConfig {
    pub profile: String,
    pub window: WindowConfig,
    pub log: LogConfig,
    pub debug: DebugConfig,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct LogConfig {
    pub level: String,
    pub filter: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default, deny_unknown_fields)]
pub struct DebugConfig {
    pub overlay: bool,
    pub inspector: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            profile: active_profile(),
            window: WindowConfig::default(),
            log: LogConfig::default(),
            debug: DebugConfig::default(),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "econbox".to_string(),
            width: 1280,
            height: 720,
            fullscreen: false,
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            filter: "wgpu=error,naga=warn".to_string(),
        }
    }
}

pub fn active_profile() -> String {
    env::var("ECONBOX_PROFILE").unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            "dev".to_string()
        } else {
            "release".to_string()
        }
    })
}

pub fn base_path() -> PathBuf {
    if let Ok(root) = env::var("BEVY_ASSET_ROOT") {
        return PathBuf::from(root);
    }
    if let Ok(root) = env::var("CARGO_MANIFEST_DIR") {
        return PathBuf::from(root);
    }
    env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
        .unwrap_or_default()
}

pub fn load() -> AppConfig {
    let profile = active_profile();
    let path = base_path().join("config").join(format!("{profile}.toml"));
    load_from(&path, &profile)
}

pub fn load_from(path: &Path, profile: &str) -> AppConfig {
    let mut config = match std::fs::read_to_string(path) {
        Ok(text) => match toml::from_str::<AppConfig>(&text) {
            Ok(config) => config,
            Err(error) => {
                eprintln!("config: {}: {error}", path.display());
                AppConfig::default()
            }
        },
        Err(_) => AppConfig::default(),
    };
    config.profile = profile.to_string();
    config
}
