pub mod properties;

use bevy::log::{Level, LogPlugin};
use entropybox_starter_simulation::prelude::*;
use properties::{
    APPLICATION_KEY, ApplicationProperties, LOGGING_KEY, LoggingProperties, ROOT_LOGGER,
};

pub fn app() -> App {
    app_with(&properties::load().expect("load properties"))
}

pub fn app_with(source: &Config) -> App {
    let application: ApplicationProperties = source.get(APPLICATION_KEY).unwrap_or_default();
    let logging: LoggingProperties = source.get(LOGGING_KEY).unwrap_or_default();

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: application.name.clone(),
                    ..default()
                }),
                ..default()
            })
            .set(LogPlugin {
                level: root_level(&logging),
                filter: log_filter(&logging),
                ..default()
            }),
    )
    .insert_resource(application)
    .insert_resource(logging)
    .add_plugins(SimulationStarter)
    .add_systems(Startup, setup)
    .add_systems(Update, spin);

    #[cfg(feature = "dev")]
    app.add_plugins(bevy::dev_tools::fps_overlay::FpsOverlayPlugin::default());

    app
}

fn root_level(logging: &LoggingProperties) -> Level {
    logging
        .level
        .get(ROOT_LOGGER)
        .map_or(Level::INFO, |level| log_level(level))
}

fn log_filter(logging: &LoggingProperties) -> String {
    let directives = logging
        .level
        .iter()
        .filter(|(target, _)| target.as_str() != ROOT_LOGGER)
        .map(|(target, level)| format!("{target}={}", level.to_ascii_lowercase()))
        .collect::<Vec<_>>()
        .join(",");

    format!("{}{directives}", bevy::log::DEFAULT_FILTER)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn logging(pairs: &[(&str, &str)]) -> LoggingProperties {
        LoggingProperties {
            level: pairs
                .iter()
                .map(|(target, level)| ((*target).to_string(), (*level).to_string()))
                .collect(),
        }
    }

    #[test]
    fn root_becomes_the_plugin_level() {
        assert_eq!(root_level(&logging(&[("root", "DEBUG")])), Level::DEBUG);
        assert_eq!(root_level(&logging(&[])), Level::INFO);
    }

    #[test]
    fn targets_become_filter_directives() {
        let filter = log_filter(&logging(&[("root", "info"), ("entropybox", "TRACE")]));

        assert!(filter.starts_with(bevy::log::DEFAULT_FILTER));
        assert!(filter.ends_with("entropybox=trace"));
        assert!(!filter.contains("root="));
    }
}
