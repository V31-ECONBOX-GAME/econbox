use bevy::input::ButtonState;
use bevy::input::InputPlugin;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::input::mouse::{MouseButtonInput, MouseScrollUnit, MouseWheel};
use bevy::input::touch::TouchPhase;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy::window::PrimaryWindow;
use econbox_camera::{
    CameraPlugin, MapView, Panning, TILE_SIZE, WindowLock, WorldBounds, WorldCamera, Zoom,
    pointer_offset,
};
use std::time::Duration;

const SCREEN: Vec2 = Vec2::new(1240.0, 719.0);

fn app(map: u32) -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )))
        .insert_resource(WorldBounds::from_tiles(map, map))
        .add_plugins(CameraPlugin);

    let window = app
        .world_mut()
        .spawn((
            Window {
                resolution: SCREEN.as_uvec2().into(),
                focused: true,
                ..default()
            },
            PrimaryWindow,
        ))
        .id();

    for _ in 0..3 {
        app.update();
    }
    (app, window)
}

fn point(app: &mut App, window: Entity, at: Vec2) {
    let mut entity = app.world_mut().entity_mut(window);
    let mut window = entity.get_mut::<Window>().expect("window");
    window.set_cursor_position(Some(at));
}

fn button(app: &mut App, window: Entity, button: MouseButton, state: ButtonState) {
    app.world_mut().write_message(MouseButtonInput {
        button,
        state,
        window,
    });
}

fn key(app: &mut App, window: Entity, key_code: KeyCode, state: ButtonState) {
    app.world_mut().write_message(KeyboardInput {
        key_code,
        logical_key: Key::Unidentified(NativeKey::Unidentified),
        state,
        text: None,
        repeat: false,
        window,
    });
}

fn wheel(app: &mut App, window: Entity, y: f32) {
    app.world_mut().write_message(MouseWheel {
        unit: MouseScrollUnit::Line,
        x: 0.0,
        y,
        window,
        phase: TouchPhase::Moved,
    });
}

fn centre(app: &mut App) -> Vec2 {
    let mut cameras = app
        .world_mut()
        .query_filtered::<&Transform, With<WorldCamera>>();
    cameras
        .single(app.world())
        .expect("camera")
        .translation
        .truncate()
}

fn zoom(app: &App) -> Zoom {
    *app.world().resource::<Zoom>()
}

fn under(app: &mut App, cursor: Vec2) -> Vec2 {
    let size = zoom(app).size;
    centre(app) + pointer_offset(cursor, SCREEN, size) * TILE_SIZE
}

#[test]
fn a_new_world_opens_centred_at_a_quarter_of_the_short_side() {
    let (mut app, _) = app(256);

    assert_eq!(centre(&mut app), Vec2::ZERO);
    assert_eq!(zoom(&app).size, 179.0);
    assert_eq!(zoom(&app).max, 281.0);

    let view = *app.world().resource::<MapView>();
    assert_eq!(view.zoom, 179.0);
    assert_eq!(view.visible.y, 358.0);
}

#[test]
fn a_right_drag_keeps_the_grabbed_spot_under_the_pointer_then_glides() {
    let (mut app, window) = app(256);
    let start = Vec2::new(620.0, 300.0);

    point(&mut app, window, start);
    app.update();
    let grabbed = under(&mut app, start);

    button(&mut app, window, MouseButton::Right, ButtonState::Pressed);
    app.update();

    for step in 1..=10 {
        let at = start + Vec2::new(-12.0 * step as f32, 4.0 * step as f32);
        point(&mut app, window, at);
        app.update();
        let spot = under(&mut app, at);
        assert!(spot.distance(grabbed) < 1e-2, "{step}: {spot} vs {grabbed}");
    }

    assert!(app.world().resource::<Panning>().activated);
    let held = centre(&mut app);

    button(&mut app, window, MouseButton::Right, ButtonState::Released);
    app.update();
    let first = centre(&mut app) - held;
    for _ in 0..60 {
        app.update();
    }
    let coasted = centre(&mut app) - held;

    assert!(first.x > 0.0 && first.y > 0.0, "{first}");
    assert!(coasted.length() > first.length() * 3.0, "{coasted}");
    assert_eq!(app.world().resource::<Panning>().velocity, Vec2::ZERO);
}

#[test]
fn the_left_button_never_drags_the_map() {
    let (mut app, window) = app(256);
    let start = Vec2::new(620.0, 300.0);

    point(&mut app, window, start);
    app.update();
    button(&mut app, window, MouseButton::Left, ButtonState::Pressed);
    app.update();
    point(&mut app, window, start + Vec2::new(-200.0, 0.0));
    app.update();

    assert_eq!(centre(&mut app), Vec2::ZERO);
}

#[test]
fn a_wheel_notch_aims_a_fifth_closer_and_eases_there() {
    let (mut app, window) = app(256);
    point(&mut app, window, Vec2::new(400.0, 300.0));
    app.update();

    wheel(&mut app, window, 1.0);
    app.update();
    let aimed = zoom(&app).target;
    assert!((aimed - 179.0 * 0.8).abs() < 1e-3, "{aimed}");

    for _ in 0..60 {
        app.update();
    }
    assert_eq!(zoom(&app).size, aimed);

    wheel(&mut app, window, -1.0);
    app.update();
    assert!((zoom(&app).target - aimed * 1.2).abs() < 1e-3);
}

#[test]
fn holding_a_key_pans_and_shift_pans_faster() {
    let (mut app, window) = app(1024);
    let size = zoom(&app).target;

    key(&mut app, window, KeyCode::KeyD, ButtonState::Pressed);
    for _ in 0..30 {
        app.update();
    }
    let slow = centre(&mut app).x / TILE_SIZE;
    key(&mut app, window, KeyCode::KeyD, ButtonState::Released);
    for _ in 0..60 {
        app.update();
    }

    let before = centre(&mut app).x / TILE_SIZE;
    key(&mut app, window, KeyCode::ShiftLeft, ButtonState::Pressed);
    key(&mut app, window, KeyCode::KeyD, ButtonState::Pressed);
    for _ in 0..30 {
        app.update();
    }
    let fast = centre(&mut app).x / TILE_SIZE - before;

    let cruise = 4.0 * (55.0 / 60.0) * size * 0.01;
    assert!(
        slow > cruise * 20.0 && slow < cruise * 30.0,
        "{slow} vs {cruise}"
    );
    assert!(fast > slow * 1.2, "{fast} vs {slow}");
}

#[test]
fn an_open_window_freezes_keys_wheel_and_drag() {
    let (mut app, window) = app(256);
    app.world_mut().resource_mut::<WindowLock>().active = true;
    point(&mut app, window, Vec2::new(400.0, 300.0));

    wheel(&mut app, window, 1.0);
    key(&mut app, window, KeyCode::KeyW, ButtonState::Pressed);
    button(&mut app, window, MouseButton::Right, ButtonState::Pressed);
    for _ in 0..10 {
        app.update();
    }
    point(&mut app, window, Vec2::new(100.0, 100.0));
    app.update();

    assert_eq!(centre(&mut app), Vec2::ZERO);
    assert_eq!(zoom(&app).target, 179.0);
}

#[test]
fn the_camera_centre_stops_at_the_map_edge() {
    let (mut app, window) = app(64);

    key(&mut app, window, KeyCode::KeyA, ButtonState::Pressed);
    key(&mut app, window, KeyCode::KeyS, ButtonState::Pressed);
    for _ in 0..300 {
        app.update();
    }

    assert_eq!(centre(&mut app), Vec2::new(-32.0, -32.0) * TILE_SIZE);
}

#[test]
fn writing_new_bounds_recentres_and_resets_the_zoom() {
    let (mut app, window) = app(256);

    key(&mut app, window, KeyCode::KeyD, ButtonState::Pressed);
    wheel(&mut app, window, 1.0);
    point(&mut app, window, Vec2::new(400.0, 300.0));
    for _ in 0..20 {
        app.update();
    }
    key(&mut app, window, KeyCode::KeyD, ButtonState::Released);
    for _ in 0..40 {
        app.update();
    }
    assert_ne!(centre(&mut app), Vec2::ZERO);

    app.insert_resource(WorldBounds::from_tiles(1024, 512));
    app.update();

    assert_eq!(centre(&mut app), Vec2::ZERO);
    assert_eq!(zoom(&app).max, 1126.0);
    assert_eq!(zoom(&app).size, 179.0);
}
