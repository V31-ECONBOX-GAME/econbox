use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::window::{MonitorSelection, WindowMode};
use econbox_config::AppConfig;
use econbox_simulation::SimulationStarter;

const PLAYER_SPEED: f32 = 300.0;

pub fn app() -> App {
    app_with(econbox_config::load())
}

pub fn app_with(config: AppConfig) -> App {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: config.window.title.clone(),
                    resolution: (config.window.width, config.window.height).into(),
                    mode: window_mode(config.window.fullscreen),
                    ..default()
                }),
                ..default()
            })
            .set(LogPlugin {
                level: log_level(&config.log.level),
                filter: config.log.filter.clone(),
                ..default()
            }),
    )
    .insert_resource(config)
    .add_plugins(SimulationStarter)
    .add_systems(Startup, setup)
    .add_systems(Update, move_player);

    #[cfg(feature = "dev")]
    app.add_plugins(econbox_debug::DebugPlugin);

    app
}

fn window_mode(fullscreen: bool) -> WindowMode {
    if fullscreen {
        WindowMode::BorderlessFullscreen(MonitorSelection::Current)
    } else {
        WindowMode::Windowed
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
