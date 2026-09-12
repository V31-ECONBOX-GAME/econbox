use desktop::properties::{
    APPLICATION_KEY, ApplicationProperties, BASE_NAME, LOGGING_KEY, LoggingProperties, ROOT_LOGGER,
};
use entropybox_starter_simulation::entropybox::config::{dir, load_from};
use std::path::Path;

const PROFILES: [&str; 3] = ["dev", "release", "test"];

fn application(profile: &str) -> ApplicationProperties {
    load_from(&dir(), BASE_NAME, profile)
        .expect("properties load")
        .get(APPLICATION_KEY)
        .expect("application section")
}

fn logging(profile: &str) -> LoggingProperties {
    load_from(&dir(), BASE_NAME, profile)
        .expect("properties load")
        .get(LOGGING_KEY)
        .expect("logging section")
}

#[test]
fn dir_points_inside_desktop() {
    assert!(dir().join("application.toml").exists());
}

#[test]
fn base_layer_supplies_every_field() {
    assert_eq!(application("none").name, "entropybox");
    assert_eq!(logging("none").level[ROOT_LOGGER], "info");
}

#[test]
fn the_application_name_does_not_move_between_profiles() {
    for profile in PROFILES {
        assert_eq!(application(profile).name, "entropybox");
    }
}

#[test]
fn a_profile_layer_only_moves_the_log_levels() {
    assert_eq!(logging("dev").level[ROOT_LOGGER], "debug");
    assert_eq!(logging("dev").level["entropybox"], "trace");
    assert_eq!(logging("release").level[ROOT_LOGGER], "warn");
    assert_eq!(logging("test").level[ROOT_LOGGER], "error");
}

#[test]
fn a_profile_layer_merges_into_the_base_map() {
    assert_eq!(logging("dev").level.len(), 2);
    assert_eq!(logging("release").level.len(), 1);
    assert!(!logging("release").level.contains_key("entropybox"));
}

#[test]
fn every_profile_layer_parses() {
    assert!(dir().join(format!("{BASE_NAME}.toml")).exists());
    for profile in PROFILES {
        assert!(dir().join(format!("{BASE_NAME}-{profile}.toml")).exists());
        assert!(load_from(&dir(), BASE_NAME, profile).is_ok());
    }
}

#[test]
fn missing_base_layer_is_an_error() {
    assert!(load_from(Path::new("/nonexistent"), BASE_NAME, "dev").is_err());
}
