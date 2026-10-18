use bevy::ecs::system::RunSystemOnce;
use bevy::input::ButtonState;
use bevy::input::InputPlugin;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::prelude::*;
use econbox_brush::{Brush, BrushPlugin, Tool};
use econbox_camera::Panning;
use econbox_clock::{ClockPlugin, WorldClock};
use econbox_creature::CreaturePlugin;
use econbox_cursor::CursorPlugin;
use econbox_tile::{Ground, Overlay, Tile};
use econbox_tilemap::TileMap;
use econbox_toolbar::catalog::TabId;
use econbox_toolbar::{Chosen, Controls, Shown, ToolbarPlugin, UiHidden, find};

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin))
        .add_plugins((ClockPlugin, CreaturePlugin, BrushPlugin, CursorPlugin))
        .init_resource::<Panning>()
        .init_resource::<UiScale>()
        .insert_resource(TileMap::filled(
            32,
            32,
            Tile::new(Ground::SoilLow, Overlay::Grass),
        ))
        .add_plugins(ToolbarPlugin);
    app
}

fn press(app: &mut App, id: &'static str) {
    let def = find(id).expect("a button");
    app.world_mut()
        .run_system_once(move |mut controls: Controls| controls.press(def))
        .expect("press runs");
}

fn send(app: &mut App, code: KeyCode, state: ButtonState) {
    app.world_mut().write_message(KeyboardInput {
        key_code: code,
        logical_key: Key::Unidentified(NativeKey::Unidentified),
        state,
        text: None,
        repeat: false,
        window: Entity::PLACEHOLDER,
    });
    app.update();
}

fn key(app: &mut App, code: KeyCode) {
    send(app, code, ButtonState::Pressed);
    send(app, code, ButtonState::Released);
}

#[test]
fn the_whole_toolbar_runs_headless_without_conflicting_systems() {
    let mut app = app();

    for _ in 0..5 {
        app.update();
    }

    assert_eq!(app.world().resource::<Shown>().0, TabId::Main);
    assert_eq!(app.world().resource::<Chosen>().0, None);
}

#[test]
fn choosing_a_tile_arms_the_brush_and_choosing_it_again_puts_it_away() {
    let mut app = app();
    app.update();

    press(&mut app, "tile_sand");
    assert_eq!(app.world().resource::<Chosen>().0, Some("tile_sand"));
    assert_eq!(
        app.world().resource::<Brush>().tool,
        Tool::Ground(Ground::Sand)
    );

    press(&mut app, "tile_sand");
    assert_eq!(app.world().resource::<Chosen>().0, None);
    assert_eq!(app.world().resource::<Brush>().tool, Tool::Inspect);
}

#[test]
fn a_button_that_needs_a_missing_system_never_becomes_the_power() {
    let mut app = app();
    app.update();

    press(&mut app, "tnt");
    assert_eq!(app.world().resource::<Chosen>().0, None);
}

#[test]
fn pause_flips_the_clock_without_taking_the_power() {
    let mut app = app();
    app.update();
    press(&mut app, "wolf");
    press(&mut app, "pause");

    assert!(app.world().resource::<WorldClock>().paused);
    assert_eq!(app.world().resource::<Chosen>().0, Some("wolf"));
}

#[test]
fn escape_unwinds_power_then_tab() {
    let mut app = app();
    app.update();
    app.world_mut().resource_mut::<Shown>().0 = TabId::Nature;
    press(&mut app, "rain");
    app.update();

    key(&mut app, KeyCode::Escape);
    assert_eq!(app.world().resource::<Chosen>().0, None);
    assert_eq!(app.world().resource::<Shown>().0, TabId::Nature);

    key(&mut app, KeyCode::Escape);
    assert_eq!(app.world().resource::<Shown>().0, TabId::Main);
}

#[test]
fn h_hides_the_interface_and_escape_brings_it_back() {
    let mut app = app();
    app.update();

    key(&mut app, KeyCode::KeyH);
    assert!(app.world().resource::<UiHidden>().0);

    key(&mut app, KeyCode::Escape);
    assert!(!app.world().resource::<UiHidden>().0);
}

#[test]
fn tab_walks_the_strip_in_order() {
    let mut app = app();
    app.update();

    key(&mut app, KeyCode::Tab);
    assert_eq!(app.world().resource::<Shown>().0, TabId::Creation);

    key(&mut app, KeyCode::Tab);
    assert_eq!(app.world().resource::<Shown>().0, TabId::Noosphere);
}

#[test]
fn a_saved_slot_brings_its_power_back_from_another_tab() {
    let mut app = app();
    app.update();
    app.world_mut().resource_mut::<Shown>().0 = TabId::Creation;
    press(&mut app, "tile_hills");
    app.update();

    send(&mut app, KeyCode::ControlLeft, ButtonState::Pressed);
    key(&mut app, KeyCode::Digit4);
    send(&mut app, KeyCode::ControlLeft, ButtonState::Released);

    key(&mut app, KeyCode::Escape);
    app.world_mut().resource_mut::<Shown>().0 = TabId::Units;
    app.update();

    key(&mut app, KeyCode::Digit4);
    assert_eq!(app.world().resource::<Chosen>().0, Some("tile_hills"));
    assert_eq!(app.world().resource::<Shown>().0, TabId::Creation);
}
