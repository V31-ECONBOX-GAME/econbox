use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::prelude::*;
use econbox_config::AppConfig;

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
        .get_resource::<AppConfig>()
        .is_some_and(|config| config.debug.overlay)
}
