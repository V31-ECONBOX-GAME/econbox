use econbox_config::{AppConfig, load_from};
use std::path::Path;

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("library/config lives two levels below the repo root")
}

#[test]
fn dev_profile_enables_debug_overlay() {
    let config = load_from(&repo_root().join("config/dev.toml"), "dev");

    assert_eq!(config.profile, "dev");
    assert!(config.debug.overlay);
    assert_eq!(config.log.level, "debug");
}

#[test]
fn release_profile_disables_debug_overlay() {
    let config = load_from(&repo_root().join("config/release.toml"), "release");

    assert_eq!(config.profile, "release");
    assert!(!config.debug.overlay);
    assert_eq!(config.log.level, "warn");
}

#[test]
fn test_profile_is_quiet() {
    let config = load_from(&repo_root().join("config/test.toml"), "test");

    assert_eq!(config.profile, "test");
    assert!(!config.debug.overlay);
    assert_eq!(config.log.level, "error");
}

#[test]
fn every_profile_file_parses() {
    for profile in ["dev", "release", "test"] {
        let path = repo_root().join(format!("config/{profile}.toml"));
        assert!(path.exists(), "{} is missing", path.display());
        assert_eq!(load_from(&path, profile).profile, profile);
    }
}

#[test]
fn missing_file_falls_back_to_defaults() {
    let config = load_from(Path::new("/nonexistent/nope.toml"), "release");

    assert_eq!(
        config,
        AppConfig {
            profile: "release".to_string(),
            ..AppConfig::default()
        }
    );
}
