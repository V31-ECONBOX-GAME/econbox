use bevy::prelude::*;
use entropybox_starter_simulation::entropybox::config::{Config, ConfigError};
use serde::Deserialize;

pub const BASE_NAME: &str = "application";
pub const WINDOW_KEY: &str = "window";
pub const LOG_KEY: &str = "log";

#[derive(Resource, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default, deny_unknown_fields)]
pub struct WindowProperties {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub fullscreen: bool,
}

#[derive(Resource, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default, deny_unknown_fields)]
pub struct LogProperties {
    pub level: String,
    pub filter: String,
}

pub fn load() -> Result<Config, ConfigError> {
    entropybox_starter_simulation::entropybox::config::load(BASE_NAME)
}
