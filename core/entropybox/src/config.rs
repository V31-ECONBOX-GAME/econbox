pub use config_rs::{Config, ConfigError};

use config_rs::File;
use std::env;
use std::path::{Path, PathBuf};

pub const PROFILE_VAR: &str = "ENTROPYBOX_PROFILE";
pub const DIR_VAR: &str = "ENTROPYBOX_CONFIG_DIR";
pub const RESOURCES: &str = "resources";

pub fn profile() -> String {
    env::var(PROFILE_VAR).unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            "dev".to_string()
        } else {
            "release".to_string()
        }
    })
}

pub fn dir() -> PathBuf {
    env::var(DIR_VAR)
        .map(PathBuf::from)
        .unwrap_or_else(|_| root().join(RESOURCES))
}

pub fn load(name: &str) -> Result<Config, ConfigError> {
    load_from(&dir(), name, &profile())
}

pub fn load_from(dir: &Path, name: &str, profile: &str) -> Result<Config, ConfigError> {
    Config::builder()
        .add_source(File::from(dir.join(name)).required(true))
        .add_source(File::from(dir.join(format!("{name}-{profile}"))).required(false))
        .build()
}

fn root() -> PathBuf {
    env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(Path::to_path_buf))
                .unwrap_or_default()
        })
}
