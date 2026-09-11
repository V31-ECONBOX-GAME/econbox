use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::prelude::*;
use entropybox_debug::{DebugConfig, DebugPlugin, overlay_enabled};

fn app_with(overlay: bool) -> App {
    let mut app = App::new();
    app.insert_resource(DebugConfig {
        overlay,
        inspector: false,
    });
    app
}

#[test]
fn overlay_follows_config() {
    assert!(overlay_enabled(&app_with(true)));
    assert!(!overlay_enabled(&app_with(false)));
}

#[test]
fn overlay_is_off_without_config() {
    assert!(!overlay_enabled(&App::new()));
}

#[test]
fn disabled_config_adds_nothing() {
    let mut app = app_with(false);
    app.add_plugins(DebugPlugin);

    assert!(!app.is_plugin_added::<FpsOverlayPlugin>());
}

#[test]
fn missing_config_adds_nothing() {
    let mut app = App::new();
    app.add_plugins(DebugPlugin);

    assert!(!app.is_plugin_added::<FpsOverlayPlugin>());
}
