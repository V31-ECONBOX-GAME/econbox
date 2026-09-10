use bevy::prelude::*;
use config_rs::{Config, ConfigError, File};
use serde::Deserialize;
use std::env;
use std::path::{Path, PathBuf};

pub const BASE_NAME: &str = "application";
pub const WINDOW_KEY: &str = "window";
pub const LOG_KEY: &str = "log";

#[derive(Resource, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default, deny_unknown_fields)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
}

#[derive(Resource, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default, deny_unknown_fields)]
pub struct LogConfig {
    pub level: String,
    pub filter: String,
}

pub fn active_profile() -> String {
    env::var("ENTROPYBOX_PROFILE").unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            "dev".to_string()
        } else {
            "release".to_string()
        }
    })
}

pub fn config_dir() -> PathBuf {
    env::var("ENTROPYBOX_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|_| env::var("CARGO_MANIFEST_DIR").map(|d| PathBuf::from(d).join("resources")))
        .unwrap_or_else(|_| {
            env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(Path::to_path_buf))
                .unwrap_or_default()
                .join("resources")
        })
}

pub fn load() -> Result<Config, ConfigError> {
    load_from(&config_dir(), &active_profile())
}

pub fn load_from(dir: &Path, profile: &str) -> Result<Config, ConfigError> {
    Config::builder()
        .add_source(File::from(dir.join(BASE_NAME)).required(true))
        .add_source(File::from(dir.join(format!("{BASE_NAME}-{profile}"))).required(false))
        .build()
}
