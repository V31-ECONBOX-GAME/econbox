use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::prelude::*;
use serde::Deserialize;

pub const CONFIG_KEY: &str = "debug";

#[derive(Resource, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default, deny_unknown_fields)]
pub struct DebugConfig {
    pub overlay: bool,
    pub inspector: bool,
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        if overlay_enabled(app) {
            app.add_plugins(FpsOverlayPlugin::default());
        }
    }
}

pub fn overlay_enabled(app: &App) -> bool {
    app.world()
        .get_resource::<DebugConfig>()
        .is_some_and(|config| config.overlay)
}
