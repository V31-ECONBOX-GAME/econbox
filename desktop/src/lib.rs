pub mod config;

use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::window::{MonitorSelection, WindowMode};
use config::{LOG_KEY, LogConfig, WINDOW_KEY, WindowConfig};
use config_rs::Config;
use entropybox_starter_simulation::SimulationStarter;

pub fn app() -> App {
    app_with(&config::load().expect("load config"))
}

pub fn app_with(source: &Config) -> App {
    let window: WindowConfig = source.get(WINDOW_KEY).unwrap_or_default();
    let log: LogConfig = source.get(LOG_KEY).unwrap_or_default();

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: window.title.clone(),
                    resolution: (window.width, window.height).into(),
                    mode: window_mode(window.fullscreen),
                    ..default()
                }),
                ..default()
            })
            .set(LogPlugin {
                level: log_level(&log.level),
                filter: log_filter(&log.filter),
                ..default()
            }),
    )
    .insert_resource(window)
    .insert_resource(log)
    .add_plugins(SimulationStarter)
    .add_systems(Startup, setup)
    .add_systems(Update, spin);

    #[cfg(feature = "dev")]
    {
        use entropybox_debug::{CONFIG_KEY, DebugConfig, DebugPlugin};
        app.insert_resource(source.get::<DebugConfig>(CONFIG_KEY).unwrap_or_default())
            .add_plugins(DebugPlugin);
    }

    app
}

fn window_mode(fullscreen: bool) -> WindowMode {
    if fullscreen {
        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
    } else {
        WindowMode::Windowed
    }
}

fn log_filter(filter: &str) -> String {
    if filter.is_empty() {
        bevy::log::DEFAULT_FILTER.to_string()
    } else {
        filter.to_string()
    }
}

fn log_level(level: &str) -> Level {
    match level.to_ascii_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    }
}

pub fn headless_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(SimulationStarter);
    app
}

#[derive(Component)]
struct Example;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Example,
        Sprite::from_color(Color::srgb(0.35, 0.7, 0.9), Vec2::splat(64.0)),
    ));
}

fn spin(time: Res<Time>, mut examples: Query<&mut Transform, With<Example>>) {
    for mut transform in &mut examples {
        transform.rotate_z(time.delta_secs());
    }
}
