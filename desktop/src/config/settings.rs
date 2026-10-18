//! Where the options chosen in the settings window are kept between runs.

use econbox_toolbar::{Options, SettingsFile};
use std::env;
use std::path::PathBuf;

pub const PATH_VAR: &str = "ECONBOX_SETTINGS";
pub const FILE_NAME: &str = "settings.toml";

pub fn user_dir() -> Option<PathBuf> {
    let home = env::var_os("HOME").map(PathBuf::from);

    if cfg!(target_os = "macos") {
        home.map(|home| home.join("Library/Application Support/econbox"))
    } else if cfg!(target_os = "windows") {
        env::var_os("APPDATA").map(|data| PathBuf::from(data).join("econbox"))
    } else {
        env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| home.map(|home| home.join(".config")))
            .map(|config| config.join("econbox"))
    }
}

pub fn file() -> SettingsFile {
    let path = env::var_os(PATH_VAR)
        .map(PathBuf::from)
        .or_else(|| user_dir().map(|dir| dir.join(FILE_NAME)));
    SettingsFile(path)
}

pub fn load(file: &SettingsFile, bar_size: u32) -> Options {
    file.load().unwrap_or_else(|| {
        let mut options = Options::default();
        options.set(
            "ui_size_main",
            econbox_toolbar::options::Value::Int(bar_size as i32),
        );
        options
    })
}
