use desktop::config::logs;
use desktop::config::properties::{
    self, ApplicationProperties, LoggingProperties, ToolbarProperties,
};
use econbox::config::{dir, load_from};
use std::path::Path;

const PROFILES: [&str; 3] = ["dev", "release", "test"];

fn application(profile: &str) -> ApplicationProperties {
    load_from(&dir(), properties::BASE_NAME, profile)
        .expect("properties load")
        .get(properties::APPLICATION)
        .expect("application section")
}

fn logging(profile: &str) -> LoggingProperties {
    load_from(&dir(), properties::BASE_NAME, profile)
        .expect("properties load")
        .get(properties::LOGGING)
        .expect("logging section")
}

fn toolbar(profile: &str) -> ToolbarProperties {
    load_from(&dir(), properties::BASE_NAME, profile)
        .expect("properties load")
        .get(properties::TOOLBAR)
        .expect("toolbar section")
}

#[test]
fn dir_points_inside_desktop() {
    assert!(dir().join("application.toml").exists());
}

#[test]
fn base_layer_supplies_every_field() {
    assert_eq!(application("none").name, "econbox");
    assert_eq!(logging("none").level[logs::ROOT], "info");
}

#[test]
fn the_application_name_does_not_move_between_profiles() {
    for profile in PROFILES {
        assert_eq!(application(profile).name, "econbox");
    }
}

#[test]
fn a_profile_layer_only_moves_the_log_levels() {
    assert_eq!(logging("dev").level[logs::ROOT], "debug");
    assert_eq!(logging("release").level[logs::ROOT], "warn");
    assert_eq!(logging("test").level[logs::ROOT], "error");
}

#[test]
fn every_profile_layer_parses() {
    assert!(
        dir()
            .join(format!("{}.toml", properties::BASE_NAME))
            .exists()
    );

    for profile in PROFILES {
        assert!(
            dir()
                .join(format!("{}-{profile}.toml", properties::BASE_NAME))
                .exists()
        );
        assert!(load_from(&dir(), properties::BASE_NAME, profile).is_ok());
    }
}

#[test]
fn the_power_bar_starts_at_full_size_in_every_profile() {
    assert_eq!(toolbar("none"), ToolbarProperties::default());

    for profile in PROFILES {
        assert_eq!(toolbar(profile).size, 100);
    }
}

#[test]
fn missing_base_layer_is_an_error() {
    assert!(load_from(Path::new("/nonexistent"), properties::BASE_NAME, "dev").is_err());
}
