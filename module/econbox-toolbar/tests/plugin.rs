use bevy::app::Propagate;
use bevy::ecs::message::MessageCursor;
use bevy::ecs::system::RunSystemOnce;
use bevy::input::ButtonState;
use bevy::input::InputPlugin;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use bevy::prelude::*;
use bevy::ui::ComputedUiRenderTargetInfo;
use econbox_brush::{Bomb, Brush, BrushPlugin, ForcedBrush, Shape, Tool};
use econbox_camera::Panning;
use econbox_clock::{ClockPlugin, WorldClock};
use econbox_creature::CreaturePlugin;
use econbox_cursor::CursorPlugin;
use econbox_tile::{Ground, Overlay, Tile, Wall};
use econbox_tilemap::TileMap;
use econbox_toolbar::catalog::TabId;
use econbox_toolbar::size::Preview;
use econbox_toolbar::{
    BarSize, Chosen, Controls, Cue, KeepSize, Shown, StripOrder, ToolbarPlugin, UiHidden,
    canvas_scale, find,
};

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

    press(&mut app, "fuse");
    assert_eq!(app.world().resource::<Chosen>().0, None);
}

#[test]
fn explosives_walls_and_the_summit_take_the_power() {
    let mut app = app();
    app.update();
    let tool = |app: &App| app.world().resource::<Brush>().tool;

    press(&mut app, "tnt");
    assert_eq!(tool(&app), Tool::Drop(Bomb::Tnt));
    assert_eq!(app.world().resource::<ForcedBrush>().0, None);

    press(&mut app, "wall_order");
    assert_eq!(tool(&app), Tool::Wall(Wall::Order));
    assert_eq!(
        app.world().resource::<ForcedBrush>().0,
        Some((Shape::Square, 0))
    );

    press(&mut app, "tile_summit");
    assert_eq!(tool(&app), Tool::Ground(Ground::Summit));
    assert_eq!(app.world().resource::<Chosen>().0, Some("tile_summit"));
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

fn heard(app: &App, cursor: &mut MessageCursor<Cue>) -> Vec<Cue> {
    let messages = app.world().resource::<Messages<Cue>>();
    cursor.read(messages).copied().collect()
}

fn ctrl_key(app: &mut App, code: KeyCode) {
    send(app, KeyCode::ControlLeft, ButtonState::Pressed);
    key(app, code);
    send(app, KeyCode::ControlLeft, ButtonState::Released);
}

#[test]
fn tab_and_shift_tab_follow_a_reordered_strip() {
    let mut app = app();
    app.update();
    app.world_mut()
        .resource_mut::<StripOrder>()
        .place(TabId::Other, 0);

    key(&mut app, KeyCode::Tab);
    assert_eq!(app.world().resource::<Shown>().0, TabId::Other);

    key(&mut app, KeyCode::Tab);
    assert_eq!(app.world().resource::<Shown>().0, TabId::Creation);

    send(&mut app, KeyCode::ShiftLeft, ButtonState::Pressed);
    key(&mut app, KeyCode::Tab);
    send(&mut app, KeyCode::ShiftLeft, ButtonState::Released);
    assert_eq!(app.world().resource::<Shown>().0, TabId::Other);
}

#[test]
fn opening_a_tab_by_key_slides_and_closing_the_power_does_not() {
    let mut app = app();
    app.update();
    let mut cursor = app.world().resource::<Messages<Cue>>().get_cursor();

    key(&mut app, KeyCode::Tab);
    assert_eq!(heard(&app, &mut cursor), vec![Cue::Slide]);

    press(&mut app, "tile_sand");
    key(&mut app, KeyCode::Escape);
    assert!(heard(&app, &mut cursor).is_empty());

    key(&mut app, KeyCode::Escape);
    assert_eq!(app.world().resource::<Shown>().0, TabId::Main);
    assert_eq!(heard(&app, &mut cursor), vec![Cue::Slide]);
}

#[test]
fn ctrl_plus_and_minus_change_the_bar_size_within_its_range() {
    let mut app = app();
    app.update();

    ctrl_key(&mut app, KeyCode::Equal);
    assert_eq!(app.world().resource::<BarSize>().percent(), 110);

    for _ in 0..10 {
        ctrl_key(&mut app, KeyCode::Equal);
    }
    assert_eq!(app.world().resource::<BarSize>().percent(), 150);

    for _ in 0..12 {
        ctrl_key(&mut app, KeyCode::Minus);
    }
    assert_eq!(app.world().resource::<BarSize>().percent(), 50);

    key(&mut app, KeyCode::Equal);
    assert_eq!(app.world().resource::<BarSize>().percent(), 50);
}

#[test]
fn a_size_change_previews_the_bar_only_while_it_is_hidden() {
    let mut app = app();
    app.update();

    ctrl_key(&mut app, KeyCode::Equal);
    assert!(!app.world().resource::<Preview>().ghost);

    key(&mut app, KeyCode::KeyH);
    ctrl_key(&mut app, KeyCode::Minus);
    let preview = *app.world().resource::<Preview>();
    assert!(preview.ghost && preview.faded);

    key(&mut app, KeyCode::KeyH);
    assert_eq!(*app.world().resource::<Preview>(), Preview::default());
}

#[test]
fn a_kept_root_is_rescaled_once_per_target_while_the_bar_follows_its_size() {
    let mut app = app();
    app.insert_resource(BarSize::new(50));
    let window = app.world_mut().spawn(Window::default()).id();
    let target = || Propagate(ComputedUiRenderTargetInfo::default());
    let kept = app.world_mut().spawn((KeepSize, target())).id();
    let plain = app.world_mut().spawn(target()).id();

    app.update();
    app.update();

    let scale_of = |entity| {
        app.world()
            .get::<Propagate<ComputedUiRenderTargetInfo>>(entity)
            .expect("a target")
            .0
            .scale_factor()
    };
    let window = app.world().get::<Window>(window).expect("a window");
    let size = Vec2::new(window.width(), window.height());

    assert_eq!(
        app.world().resource::<UiScale>().0,
        canvas_scale(size, BarSize::new(50))
    );
    assert!((scale_of(kept) - 1.5).abs() < 1e-5, "{}", scale_of(kept));
    assert_eq!(scale_of(plain), 1.0);
}
