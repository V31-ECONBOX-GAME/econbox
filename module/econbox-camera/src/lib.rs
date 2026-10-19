//! The map camera, ported from WorldBox's MoveCamera: wheel and key zoom, right or middle button drag with glide, key panning, focus and follow.

mod focus;
mod pan;
mod zoom;

use bevy::camera::ScalingMode;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub use focus::{FOLLOW_EASE_TIME, Focus, FocusEnded, FocusOn, ease_out_cubic};
pub use pan::{
    DRAG_ACTIVATION, DRAG_CARRY, DRAG_HOLD_FRAMES, DRAG_STILL, DragInput, GLIDE_DECAY, GLIDE_STOP,
    Heading, KEY_FAST, KEY_RATE, Panning, REFERENCE_FRAME, clamp_to_map, key_distance,
    opening_position,
};
pub use zoom::{
    FOCUS_ZOOM, KEY_STEP, WHEEL_STEP, ZOOM_MAX_DEFAULT, ZOOM_MAX_RATIO, ZOOM_MIN, ZOOM_SLACK, Zoom,
    opening_zoom, unity_clamp, zoom_max_for,
};

pub const TILE_SIZE: f32 = 8.0;

pub const MOD_KEYS: [KeyCode; 7] = [
    KeyCode::AltLeft,
    KeyCode::AltRight,
    KeyCode::ControlLeft,
    KeyCode::ControlRight,
    KeyCode::SuperLeft,
    KeyCode::ShiftLeft,
    KeyCode::ShiftRight,
];

pub const FAST_KEYS: [KeyCode; 2] = [KeyCode::ShiftLeft, KeyCode::ShiftRight];
pub const ZOOM_IN_KEYS: [KeyCode; 3] = [KeyCode::KeyQ, KeyCode::Equal, KeyCode::NumpadAdd];
pub const ZOOM_OUT_KEYS: [KeyCode; 3] = [KeyCode::KeyE, KeyCode::Minus, KeyCode::NumpadSubtract];

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct WorldCamera;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CameraSystems;

#[derive(Resource, Debug, Clone, Copy)]
pub struct WorldBounds {
    pub width: f32,
    pub height: f32,
}

impl Default for WorldBounds {
    fn default() -> Self {
        Self::from_tiles(256, 256)
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

    pub fn tiles(self) -> Vec2 {
        Vec2::new(self.width, self.height) / TILE_SIZE
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct CameraConfig {
    pub zoom_speed: f32,
    pub move_speed: f32,
    pub move_max: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            zoom_speed: 5.0,
            move_speed: 0.01,
            move_max: 0.06,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct PointerOverUi(pub bool);

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WindowLock {
    pub active: bool,
    pub animating: bool,
}

impl WindowLock {
    pub fn controls_locked(self) -> bool {
        self.active || self.animating
    }
}

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq)]
pub struct MapView {
    pub zoom: f32,
    pub zoom_target: f32,
    pub zoom_max: f32,
    pub visible: Vec2,
    pub visible_tiles: f32,
    pub centre: Vec2,
    pub pixels_per_tile: f32,
}

pub fn holding_mod(keys: &ButtonInput<KeyCode>) -> bool {
    keys.any_pressed(MOD_KEYS)
}

pub fn pointer_offset(cursor: Vec2, viewport: Vec2, size: f32) -> Vec2 {
    let per_pixel = size * 2.0 / viewport.y.max(1.0);
    Vec2::new(cursor.x - viewport.x * 0.5, viewport.y * 0.5 - cursor.y) * per_pixel
}

pub fn visible_tiles(viewport: Vec2, size: f32) -> Vec2 {
    let aspect = viewport.x / viewport.y.max(1.0);
    Vec2::new(size * 2.0 * aspect, size * 2.0)
}

pub fn projection_for(size: f32) -> OrthographicProjection {
    OrthographicProjection {
        scaling_mode: ScalingMode::FixedVertical {
            viewport_height: size * 2.0 * TILE_SIZE,
        },
        ..OrthographicProjection::default_2d()
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraConfig>()
            .init_resource::<Panning>()
            .init_resource::<PointerOverUi>()
            .init_resource::<WindowLock>()
            .init_resource::<WorldBounds>()
            .init_resource::<Zoom>()
            .init_resource::<Focus>()
            .init_resource::<MapView>()
            .add_message::<FocusOn>()
            .add_message::<FocusEnded>()
            .add_systems(Startup, spawn)
            .add_systems(
                Update,
                (steer, hotkeys, apply).chain().in_set(CameraSystems),
            );
    }
}

fn spawn(mut commands: Commands, zoom: Res<Zoom>) {
    commands.spawn((
        WorldCamera,
        Camera2d,
        Projection::Orthographic(projection_for(zoom.size)),
    ));
}

#[derive(Default)]
struct Steering {
    screen: UVec2,
    frame: u32,
}

fn steer(
    camera: Option<Single<&mut Transform, With<WorldCamera>>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    bounds: Res<WorldBounds>,
    config: Res<CameraConfig>,
    over_ui: Res<PointerOverUi>,
    lock: Res<WindowLock>,
    time: Res<Time>,
    units: Query<&GlobalTransform>,
    mut asked: MessageReader<FocusOn>,
    mut ended: MessageWriter<FocusEnded>,
    mut zoom: ResMut<Zoom>,
    mut panning: ResMut<Panning>,
    mut focus: ResMut<Focus>,
    mut state: Local<Steering>,
) {
    let Some(camera) = camera else { return };
    let mut transform = camera.into_inner();
    let Ok(window) = windows.single() else {
        return;
    };

    state.frame = state.frame.wrapping_add(1);
    let frame = state.frame;
    let dt = time.delta_secs();
    let map = bounds.tiles();
    let screen = window.physical_size();
    let mut position = transform.translation.truncate() / TILE_SIZE;

    if bounds.is_changed() {
        position = opening_position(map);
        zoom.reset(screen, map);
        panning.mouse_last = false;
    }

    for request in asked.read() {
        match *request {
            FocusOn::Spot(at) => focus.spot(&mut zoom, &mut position, at / TILE_SIZE),
            FocusOn::Follow(unit) => focus.follow(&mut zoom, unit),
        }
    }

    'update: {
        if state.screen != screen {
            state.screen = screen;
            zoom.reset(screen, map);
            panning.mouse_last = false;
            break 'update;
        }

        if !lock.active && (!over_ui.0 || focus.following()) {
            let unit_at = focus
                .unit
                .and_then(|unit| units.get(unit).ok())
                .map(|place| place.translation().truncate() / TILE_SIZE);
            focus.track(&mut position, unit_at, panning.running, dt);
        }

        if zoom.target != zoom.size {
            zoom.approach(dt, config.zoom_speed);
        }

        if let Some(end) = focus.check(&zoom) {
            ended.write(end);
        }

        if lock.controls_locked() {
            panning.clear(frame);
            break 'update;
        }

        let drag_buttons = [MouseButton::Right, MouseButton::Middle];
        let viewport = Vec2::new(window.width(), window.height());
        let input = DragInput {
            pressed: buttons.any_just_pressed(drag_buttons),
            held: buttons.any_pressed(drag_buttons),
            over_ui: over_ui.0,
            pointer: window
                .cursor_position()
                .map(|cursor| pointer_offset(cursor, viewport, zoom.size)),
        };
        panning.drag(&mut position, input, map, frame);

        if !lock.active {
            panning.glide(&mut position, buttons.pressed(MouseButton::Right), map, dt);
        }
    }

    let next = position * TILE_SIZE;
    if transform.translation.truncate() != next {
        transform.translation.x = next.x;
        transform.translation.y = next.y;
    }
}

fn hotkeys(
    windows: Query<&Window, With<PrimaryWindow>>,
    keys: Res<ButtonInput<KeyCode>>,
    wheel: Res<AccumulatedMouseScroll>,
    over_ui: Res<PointerOverUi>,
    lock: Res<WindowLock>,
    config: Res<CameraConfig>,
    focus: Res<Focus>,
    time: Res<Time>,
    mut zoom: ResMut<Zoom>,
    mut panning: ResMut<Panning>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    if !window.focused || lock.active {
        return;
    }

    let modded = holding_mod(&keys);

    if wheel.delta.y != 0.0
        && !modded
        && window.cursor_position().is_some()
        && (!over_ui.0 || focus.following())
    {
        if wheel.delta.y < 0.0 {
            zoom.wheel_out();
        } else {
            zoom.wheel_in();
        }
    }

    let fast = keys.any_pressed(FAST_KEYS);
    let dt = time.delta_secs();

    for heading in Heading::ALL {
        if !keys.any_pressed(heading.keys()) {
            continue;
        }

        if !modded {
            let distance = key_distance(dt, false, zoom.target, config.move_speed);
            panning.push(heading, distance, zoom.target, config.move_max);
        }

        if fast {
            let distance = key_distance(dt, true, zoom.target, config.move_speed);
            panning.push(heading, distance, zoom.target, config.move_max);
        }
    }

    if !modded {
        if keys.any_pressed(ZOOM_IN_KEYS) {
            zoom.key_in();
        }

        if keys.any_pressed(ZOOM_OUT_KEYS) {
            zoom.key_out();
        }
    }
}

fn apply(
    camera: Option<Single<(&Transform, &mut Projection), With<WorldCamera>>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    zoom: Res<Zoom>,
    mut view: ResMut<MapView>,
) {
    let Some(camera) = camera else { return };
    let (transform, mut projection) = camera.into_inner();

    let height = zoom.size * 2.0 * TILE_SIZE;
    if let Projection::Orthographic(projection) = &mut *projection
        && !matches!(
            projection.scaling_mode,
            ScalingMode::FixedVertical { viewport_height } if viewport_height == height
        )
    {
        projection.scaling_mode = ScalingMode::FixedVertical {
            viewport_height: height,
        };
    }

    let viewport = windows
        .single()
        .map(|window| Vec2::new(window.width(), window.height()))
        .unwrap_or(Vec2::ONE);
    let visible = visible_tiles(viewport, zoom.size);
    let next = MapView {
        zoom: zoom.size,
        zoom_target: zoom.target,
        zoom_max: zoom.max,
        visible,
        visible_tiles: visible.x,
        centre: transform.translation.truncate(),
        pixels_per_tile: viewport.y / (zoom.size * 2.0).max(f32::EPSILON),
    };

    if *view != next {
        *view = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_map_is_measured_in_tiles() {
        let bounds = WorldBounds::from_tiles(448, 300);

        assert_eq!(bounds.width, 448.0 * TILE_SIZE);
        assert_eq!(bounds.half(), Vec2::new(448.0, 300.0) * TILE_SIZE * 0.5);
        assert_eq!(bounds.tiles(), Vec2::new(448.0, 300.0));
        assert_eq!(WorldBounds::default().tiles(), Vec2::splat(256.0));
    }

    #[test]
    fn the_pointer_offset_is_measured_from_the_screen_centre_in_tiles() {
        let viewport = Vec2::new(1000.0, 500.0);

        assert_eq!(
            pointer_offset(Vec2::new(500.0, 250.0), viewport, 50.0),
            Vec2::ZERO
        );
        assert_eq!(
            pointer_offset(Vec2::new(1000.0, 0.0), viewport, 50.0),
            Vec2::new(100.0, 50.0)
        );
        assert_eq!(
            pointer_offset(Vec2::new(0.0, 500.0), viewport, 50.0),
            Vec2::new(-100.0, -50.0)
        );
    }

    #[test]
    fn the_orthographic_size_is_half_the_visible_height() {
        let visible = visible_tiles(Vec2::new(1600.0, 900.0), 90.0);

        assert_eq!(visible.y, 180.0);
        assert!((visible.x - 320.0).abs() < 1e-3);

        let ScalingMode::FixedVertical { viewport_height } = projection_for(90.0).scaling_mode
        else {
            panic!("fixed vertical");
        };
        assert_eq!(viewport_height, 180.0 * TILE_SIZE);
    }

    #[test]
    fn any_worldbox_modifier_blocks_plain_camera_keys() {
        for key in MOD_KEYS {
            let mut keys = ButtonInput::<KeyCode>::default();
            keys.press(key);
            assert!(holding_mod(&keys), "{key:?}");
        }

        let mut keys = ButtonInput::<KeyCode>::default();
        keys.press(KeyCode::KeyW);
        assert!(!holding_mod(&keys));
    }

    #[test]
    fn the_defaults_are_worldbox_numbers() {
        let config = CameraConfig::default();

        assert_eq!(
            (config.zoom_speed, config.move_speed, config.move_max),
            (5.0, 0.01, 0.06)
        );
        assert!(Panning::default().can_drag_map);
        assert!(
            WindowLock {
                active: false,
                animating: true
            }
            .controls_locked()
        );
    }
}
