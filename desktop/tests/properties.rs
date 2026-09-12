use desktop::properties::{BASE_NAME, LOG_KEY, LogProperties, WINDOW_KEY, WindowProperties};
use entropybox_starter_simulation::entropybox::config::{dir, load_from};
use std::path::Path;

fn window(profile: &str) -> WindowProperties {
    load_from(&dir(), BASE_NAME, profile)
        .expect("properties load")
        .get(WINDOW_KEY)
        .expect("window section")
}

fn log(profile: &str) -> LogProperties {
    load_from(&dir(), BASE_NAME, profile)
        .expect("properties load")
        .get(LOG_KEY)
        .expect("log section")
}

#[test]
fn dir_points_inside_desktop() {
    assert!(dir().join("application.toml").exists());
}

#[test]
fn base_layer_supplies_every_field() {
    assert_eq!(window("none").title, "entropybox");
    assert_eq!(window("none").width, 1280);
    assert_eq!(window("none").height, 720);
    assert!(!window("none").fullscreen);
    assert_eq!(log("none").level, "info");
}

#[test]
fn dev_overrides_only_what_it_declares() {
    assert_eq!(window("dev").title, "entropybox (dev)");
    assert_eq!(window("dev").width, 1280);
    assert_eq!(log("dev").level, "debug");
    assert_eq!(log("dev").filter, "");
}

#[test]
fn release_and_test_are_quiet() {
    assert_eq!(log("release").level, "warn");
    assert_eq!(log("test").level, "error");
    assert_eq!(window("release").title, "entropybox");
}

#[test]
fn every_profile_layer_parses() {
    assert!(dir().join(format!("{BASE_NAME}.toml")).exists());
    for profile in ["dev", "release", "test"] {
        assert!(dir().join(format!("{BASE_NAME}-{profile}.toml")).exists());
        assert!(load_from(&dir(), BASE_NAME, profile).is_ok());
    }
}

#[test]
fn missing_base_layer_is_an_error() {
    assert!(load_from(Path::new("/nonexistent"), BASE_NAME, "dev").is_err());
}

#[test]
fn debug_section_is_owned_by_the_debug_module() {
    let source = load_from(&dir(), BASE_NAME, "dev").expect("properties load");
    let overlay: bool = source.get("debug.overlay").expect("debug.overlay");

    assert!(overlay);
}
