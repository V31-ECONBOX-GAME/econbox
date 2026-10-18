//! Top down map camera: drag to pan, wheel to zoom, clamped to the world.

use bevy::camera::ScalingMode;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;

pub const TILE_SIZE: f32 = 8.0;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct WorldCamera;

#[derive(Resource, Debug, Clone, Copy)]
pub struct WorldBounds {
    pub width: f32,
    pub height: f32,
}

impl Default for WorldBounds {
    fn default() -> Self {
        Self {
            width: 448.0 * TILE_SIZE,
            height: 448.0 * TILE_SIZE,
        }
    }
}

impl WorldBounds {
    pub fn from_tiles(width: u32, height: u32) -> Self {
        Self {
            width: width as f32 * TILE_SIZE,
            height: height as f32 * TILE_SIZE,
        }
    }

    pub fn half(self) -> Vec2 {
        Vec2::new(self.width, self.height) * 0.5
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct CameraConfig {
    pub zoom_speed: f32,
    pub keyboard_speed: f32,
    pub min_span: f32,
    pub max_span: f32,
    pub ease: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            zoom_speed: 0.16,
            keyboard_speed: 1.1,
            min_span: 24.0 * TILE_SIZE,
            max_span: 460.0 * TILE_SIZE,
            ease: 11.0,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct PointerOverUi(pub bool);

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Panning {
    pub with_left: bool,
}

impl Default for Panning {
    fn default() -> Self {
        Self { with_left: true }
    }
}

impl Panning {
    pub fn dragging(self, buttons: &ButtonInput<MouseButton>) -> bool {
        buttons.pressed(MouseButton::Right)
            || (self.with_left && buttons.pressed(MouseButton::Left))
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct Zoom {
    pub span: f32,
    pub target: f32,
}

impl Default for Zoom {
    fn default() -> Self {
        let span = 260.0 * TILE_SIZE;
        Self { span, target: span }
    }
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct MapView {
    pub span: f32,
    pub visible_tiles: f32,
    pub centre: Vec2,
}

pub fn ease(current: f32, target: f32, rate: f32, seconds: f32) -> f32 {
    current + (target - current) * (1.0 - (-rate * seconds).exp())
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraConfig>()
            .init_resource::<Panning>()
            .init_resource::<PointerOverUi>()
            .init_resource::<WorldBounds>()
            .init_resource::<Zoom>()
            .init_resource::<MapView>()
            .add_systems(Startup, spawn)
            .add_systems(Update, (aim, glide, pan, clamp, track).chain());
    }
}

fn spawn(mut commands: Commands, zoom: Res<Zoom>) {
    commands.spawn((
        WorldCamera,
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: zoom.span,
                min_height: zoom.span,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn aim(
    scroll: Res<AccumulatedMouseScroll>,
    config: Res<CameraConfig>,
    keys: Res<ButtonInput<KeyCode>>,
    over_ui: Res<PointerOverUi>,
    mut zoom: ResMut<Zoom>,
) {
    if scroll.delta.y == 0.0 || over_ui.0 || modified(&keys) {
        return;
    }

    let factor = 1.0 - scroll.delta.y.clamp(-3.0, 3.0) * config.zoom_speed;
    zoom.target = (zoom.target * factor).clamp(config.min_span, config.max_span);
}

fn glide(
    camera: Option<Single<&mut Projection, With<WorldCamera>>>,
    config: Res<CameraConfig>,
    time: Res<Time>,
    mut zoom: ResMut<Zoom>,
) {
    let Some(camera) = camera else { return };

    zoom.span = ease(zoom.span, zoom.target, config.ease, time.delta_secs())
        .clamp(config.min_span, config.max_span);

    if let Projection::Orthographic(projection) = &mut *camera.into_inner() {
        projection.scaling_mode = ScalingMode::AutoMin {
            min_width: zoom.span,
            min_height: zoom.span,
        };
    }
}

fn pan(
    camera: Option<Single<(&mut Transform, &Projection), With<WorldCamera>>>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    motion: Res<AccumulatedMouseMotion>,
    panning: Res<Panning>,
    config: Res<CameraConfig>,
    over_ui: Res<PointerOverUi>,
    windows: Query<&Window>,
    time: Res<Time>,
    mut grabbed: Local<bool>,
) {
    let Some(camera) = camera else { return };
    let (mut transform, projection) = camera.into_inner();
    let Projection::Orthographic(projection) = projection else {
        return;
    };

    let visible = projection.area.size();

    if buttons.any_just_pressed([MouseButton::Left, MouseButton::Right]) {
        *grabbed = !over_ui.0;
    }

    if !panning.dragging(&buttons) {
        *grabbed = false;
    }

    if *grabbed
        && motion.delta != Vec2::ZERO
        && let Ok(window) = windows.single()
    {
        let viewport = Vec2::new(window.width(), window.height()).max(Vec2::splat(1.0));
        let per_pixel = visible / viewport;
        transform.translation.x -= motion.delta.x * per_pixel.x;
        transform.translation.y += motion.delta.y * per_pixel.y;
    }

    let axis = if modified(&keys) {
        Vec2::ZERO
    } else {
        keyboard_axis(&keys)
    };
    if axis != Vec2::ZERO {
        let step = axis * visible * config.keyboard_speed * time.delta_secs() * 0.5;
        transform.translation.x += step.x;
        transform.translation.y += step.y;
    }
}

pub fn modified(keys: &ButtonInput<KeyCode>) -> bool {
    keys.any_pressed([
        KeyCode::AltLeft,
        KeyCode::AltRight,
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ])
}

pub fn keyboard_axis(keys: &ButtonInput<KeyCode>) -> Vec2 {
    let mut axis = Vec2::ZERO;

    if keys.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        axis.x -= 1.0;
    }
    if keys.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        axis.x += 1.0;
    }
    if keys.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
        axis.y -= 1.0;
    }
    if keys.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) {
        axis.y += 1.0;
    }

    axis.normalize_or_zero()
}

fn clamp(
    camera: Option<Single<(&mut Transform, &Projection), With<WorldCamera>>>,
    bounds: Res<WorldBounds>,
) {
    let Some(camera) = camera else { return };
    let (mut transform, projection) = camera.into_inner();
    let Projection::Orthographic(projection) = projection else {
        return;
    };

    let limit = (bounds.half() - projection.area.size() * 0.5).max(Vec2::ZERO);
    transform.translation.x = transform.translation.x.clamp(-limit.x, limit.x);
    transform.translation.y = transform.translation.y.clamp(-limit.y, limit.y);
}

fn track(
    camera: Option<Single<&Transform, With<WorldCamera>>>,
    zoom: Res<Zoom>,
    mut view: ResMut<MapView>,
) {
    let Some(camera) = camera else { return };

    view.span = zoom.span;
    view.visible_tiles = zoom.span / TILE_SIZE;
    view.centre = camera.into_inner().translation.truncate();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_worldbox_sized_map_is_measured_in_tiles() {
        let bounds = WorldBounds::from_tiles(448, 448);

        assert_eq!(bounds.width, 448.0 * TILE_SIZE);
        assert_eq!(bounds.half(), Vec2::splat(448.0 * TILE_SIZE * 0.5));
    }

    #[test]
    fn the_keyboard_axis_is_normalised() {
        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyW);
        keys.press(KeyCode::KeyD);

        assert!((keyboard_axis(&keys).length() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn easing_settles_on_the_target() {
        let mut span = 4000.0;

        for _ in 0..500 {
            span = ease(span, 500.0, 11.0, 1.0 / 120.0);
        }

        assert!((span - 500.0).abs() < 1e-2, "{span}");
    }

    #[test]
    fn easing_never_runs_away_however_fast_the_frames_come() {
        for step in [1.0 / 240.0_f32, 1.0 / 60.0, 0.5] {
            let mut span = 4000.0_f32;

            for _ in 0..2000 {
                span = ease(span, 500.0, 11.0, step);
            }

            assert!(span.is_finite() && span <= 4000.0, "step {step}: {span}");
        }
    }

    #[test]
    fn the_right_button_always_pans_and_the_left_one_only_when_it_is_free() {
        let mut left = ButtonInput::<MouseButton>::default();
        left.press(MouseButton::Left);
        let mut right = ButtonInput::<MouseButton>::default();
        right.press(MouseButton::Right);

        assert!(Panning::default().dragging(&left));
        assert!(Panning::default().dragging(&right));
        assert!(!Panning { with_left: false }.dragging(&left));
        assert!(Panning { with_left: false }.dragging(&right));
    }

    #[test]
    fn the_zoom_range_runs_from_a_whole_map_to_a_handful_of_tiles() {
        let config = CameraConfig::default();

        assert!(config.max_span >= 440.0 * TILE_SIZE);
        assert!(config.min_span <= 32.0 * TILE_SIZE);
        assert!(Zoom::default().span < config.max_span);
        assert!(Zoom::default().span > config.min_span);
    }
}
