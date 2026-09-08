use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::prelude::*;
use debug::{DebugPlugin, overlay_enabled};
use econbox_config::{AppConfig, DebugConfig};

fn app_with_overlay(overlay: bool) -> App {
    let mut app = App::new();
    app.insert_resource(AppConfig {
        debug: DebugConfig {
            overlay,
            ..DebugConfig::default()
        },
        ..AppConfig::default()
    });
    app
}

#[test]
fn overlay_follows_config() {
    assert!(overlay_enabled(&app_with_overlay(true)));
    assert!(!overlay_enabled(&app_with_overlay(false)));
}

#[test]
fn overlay_is_off_without_config() {
    assert!(!overlay_enabled(&App::new()));
}

#[test]
fn disabled_config_adds_nothing() {
    let mut app = app_with_overlay(false);
    app.add_plugins(DebugPlugin);

    assert!(!app.is_plugin_added::<FpsOverlayPlugin>());
}

#[test]
fn missing_config_adds_nothing() {
    let mut app = App::new();
    app.add_plugins(DebugPlugin);

    assert!(!app.is_plugin_added::<FpsOverlayPlugin>());
}
