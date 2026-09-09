pub mod config;

use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::window::{MonitorSelection, WindowMode};
use config::{LOG_KEY, LogConfig, WINDOW_KEY, WindowConfig};
use config_rs::Config;
use econbox_simulation::SimulationStarter;

const PLAYER_SPEED: f32 = 300.0;

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
    .add_systems(Update, move_player);

    #[cfg(feature = "dev")]
    {
        use econbox_debug::{CONFIG_KEY, DebugConfig, DebugPlugin};
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
struct Player;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Player,
        Sprite::from_color(Color::srgb(0.35, 0.7, 0.9), Vec2::splat(64.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

fn move_player(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut players: Query<&mut Transform, With<Player>>,
) {
    let mut direction = Vec2::ZERO;

    if keys.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
        direction.y += 1.0;
    }
    if keys.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
        direction.y -= 1.0;
    }
    if keys.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        direction.x -= 1.0;
    }
    if keys.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        direction.x += 1.0;
    }

    let Some(direction) = direction.try_normalize() else {
        return;
    };

    for mut transform in &mut players {
        transform.translation += direction.extend(0.0) * PLAYER_SPEED * time.delta_secs();
    }
}
