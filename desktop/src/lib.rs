pub mod config;
pub mod hud;
pub mod prelude;
pub mod world;

use config::properties::{self, ApplicationProperties, LoggingProperties, ToolbarProperties};
use config::{logs, settings, windows};
use econbox_brush::BrushPlugin;
use econbox_camera::CameraPlugin;
use econbox_clock::ClockPlugin;
use econbox_clouds::CloudsPlugin;
use econbox_creature::CreaturePlugin;
use econbox_cursor::CursorPlugin;
use econbox_flora::FloraPlugin;
use econbox_tileview::TileViewPlugin;
use econbox_toolbar::{BarSize, ToolbarPlugin};
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
    let toolbar: ToolbarProperties = source.get(properties::TOOLBAR).unwrap_or_default();
    let file = settings::file();
    let options = settings::load(&file, toolbar.size);
    let bar = BarSize::new(options.int("ui_size_main").max(0) as u32);

    let plugins = DefaultPlugins
        .set(windows::plugin(application))
        .set(logs::plugin(&logging));

    #[cfg(feature = "dev")]
    let plugins = plugins.disable::<bevy::dev_tools::render_debug::RenderDebugOverlayPlugin>();

    let mut app = App::new();
    app.add_plugins(plugins)
        .insert_resource(ClearColor(ocean()))
        .insert_resource(bar)
        .insert_resource(options)
        .insert_resource(file)
        .add_plugins((
            CameraPlugin,
            ClockPlugin,
            TileViewPlugin,
            CursorPlugin,
            CreaturePlugin,
            BrushPlugin,
            FloraPlugin,
            CloudsPlugin,
        ))
        .add_plugins((HudPlugin, ToolbarPlugin));

    #[cfg(feature = "dev")]
    {
        app.add_plugins(bevy::dev_tools::fps_overlay::FpsOverlayPlugin::default());
        app.add_systems(Update, (keep_fps_size, snapshot));
    }

    app
}

#[cfg(feature = "dev")]
fn keep_fps_size(
    mut commands: Commands,
    roots: Query<(Entity, &GlobalZIndex), Added<GlobalZIndex>>,
) {
    for (entity, layer) in &roots {
        if layer.0 == bevy::dev_tools::fps_overlay::FPS_OVERLAY_ZINDEX {
            commands
                .entity(entity)
                .insert(econbox_toolbar::UiLayer::Fixed);
        }
    }
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
