//! The application properties and the file they are read from.

use econbox::config::{Config, ConfigError};
use serde::Deserialize;
use std::collections::BTreeMap;

pub const BASE_NAME: &str = "application";

pub const APPLICATION: &str = "application";

pub const LOGGING: &str = "logging";

pub const TOOLBAR: &str = "toolbar";

#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default, deny_unknown_fields)]
pub struct ApplicationProperties {
    pub name: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default, deny_unknown_fields)]
pub struct LoggingProperties {
    pub level: BTreeMap<String, String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct ToolbarProperties {
    pub size: u32,
}

impl Default for ToolbarProperties {
    fn default() -> Self {
        Self { size: 100 }
    }
}

pub fn load() -> Result<Config, ConfigError> {
    econbox::config::load(BASE_NAME)
}
