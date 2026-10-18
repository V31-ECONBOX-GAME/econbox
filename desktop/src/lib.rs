pub mod config;
pub mod hud;
pub mod prelude;
pub mod world;

use config::properties::{self, ApplicationProperties, LoggingProperties};
use config::{logs, windows};
use econbox_brush::BrushPlugin;
use econbox_camera::CameraPlugin;
use econbox_clock::ClockPlugin;
use econbox_creature::CreaturePlugin;
use econbox_cursor::CursorPlugin;
use econbox_tileview::TileViewPlugin;
use econbox_toolbar::ToolbarPlugin;
use hud::HudPlugin;
use prelude::*;

pub fn ocean() -> Color {
    let [r, g, b] = Ground::DeepOcean.color();
    Color::srgb_u8(r, g, b)
}

pub fn app() -> App {
    app_with(&properties::load().expect("load properties"))
}

pub fn app_with(source: &Config) -> App {
    let application: ApplicationProperties =
        source.get(properties::APPLICATION).unwrap_or_default();
    let logging: LoggingProperties = source.get(properties::LOGGING).unwrap_or_default();

    let plugins = DefaultPlugins
        .set(windows::plugin(application))
        .set(logs::plugin(&logging));

    #[cfg(feature = "dev")]
    let plugins = plugins.disable::<bevy::dev_tools::render_debug::RenderDebugOverlayPlugin>();

    let mut app = App::new();
    app.add_plugins(plugins)
        .insert_resource(ClearColor(ocean()))
        .add_plugins((
            CameraPlugin,
            ClockPlugin,
            TileViewPlugin,
            CursorPlugin,
            CreaturePlugin,
            BrushPlugin,
        ))
        .add_plugins((HudPlugin, ToolbarPlugin));

    #[cfg(feature = "dev")]
    {
        app.add_plugins(bevy::dev_tools::fps_overlay::FpsOverlayPlugin::default());
        app.add_systems(Update, snapshot);
    }

    app
}

#[cfg(feature = "dev")]
fn snapshot(
    mut commands: Commands,
    frames: Res<bevy::diagnostic::FrameCount>,
    mut taken: Local<bool>,
) {
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};

    if *taken || frames.0 < 120 {
        return;
    }

    let Ok(path) = std::env::var("ECONBOX_SHOT") else {
        return;
    };

    *taken = true;
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
}

pub fn headless_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app
}
