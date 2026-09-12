use bevy::prelude::*;
use entropybox_starter_simulation::entropybox::config::{Config, ConfigError};
use serde::Deserialize;
use std::collections::BTreeMap;

pub const BASE_NAME: &str = "application";
pub const APPLICATION_KEY: &str = "application";
pub const LOGGING_KEY: &str = "logging";
pub const ROOT_LOGGER: &str = "root";

#[derive(Resource, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default, deny_unknown_fields)]
pub struct ApplicationProperties {
    pub name: String,
}

#[derive(Resource, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default, deny_unknown_fields)]
pub struct LoggingProperties {
    pub level: BTreeMap<String, String>,
}

pub fn load() -> Result<Config, ConfigError> {
    entropybox_starter_simulation::entropybox::config::load(BASE_NAME)
}
