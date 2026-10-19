use bevy::ecs::system::RunSystemOnce;
use bevy::input::ButtonState;
use bevy::input::InputPlugin;
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use econbox_brush::{Brush, BrushPlugin, Tool};
use econbox_camera::{Panning, TILE_SIZE, WorldBounds};
use econbox_clock::{ClockPlugin, WorldClock};
use econbox_clouds::{Cloud, CloudKind, CloudShadow, CloudsPlugin};
use econbox_creature::{CreaturePlugin, centre_of};
use econbox_cursor::{CursorPlugin, Hovered};
use econbox_tile::{Ground, Overlay, Tile};
use econbox_tilemap::{TileMap, TilePos};
use econbox_toolbar::{Chosen, Controls, ToolbarPlugin, find};

const SIDE: u32 = 48;

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin))
        .add_plugins((ClockPlugin, CreaturePlugin, BrushPlugin, CursorPlugin))
        .init_resource::<Panning>()
        .init_resource::<UiScale>()
        .insert_resource(WorldBounds::from_tiles(SIDE, SIDE))
        .insert_resource(TileMap::filled(
            SIDE,
            SIDE,
            Tile::new(Ground::SoilLow, Overlay::Grass),
        ))
        .add_plugins((ToolbarPlugin, CloudsPlugin));
    app.update();
    app
}

fn press(app: &mut App, id: &'static str) {
    let def = find(id).expect("a button");
    app.world_mut()
        .run_system_once(move |mut controls: Controls| controls.press(def))
        .expect("press runs");
}

fn mouse(app: &mut App, state: ButtonState) {
    app.world_mut().write_message(MouseButtonInput {
        button: MouseButton::Left,
        state,
        window: Entity::PLACEHOLDER,
    });
    app.update();
}

fn click(app: &mut App, pos: TilePos) {
    app.world_mut().resource_mut::<Hovered>().pos = Some(pos);
    mouse(app, ButtonState::Pressed);
    mouse(app, ButtonState::Released);
    app.update();
}

fn sky(app: &mut App) -> Vec<(Cloud, Vec2)> {
    let mut clouds = app.world_mut().query::<(&Cloud, &Transform, &Children)>();
    let mut shadows = app
        .world_mut()
        .query_filtered::<&Transform, With<CloudShadow>>();
    let world = app.world();

    clouds
        .iter(world)
        .map(|(cloud, at, children)| {
            let shadow = children
                .iter()
                .find_map(|child| shadows.get(world, child).ok())
                .expect("a shadow");
            (
                cloud.clone(),
                (at.translation + shadow.translation).truncate(),
            )
        })
        .collect()
}

#[test]
fn every_cloud_button_takes_the_left_button_as_a_special_power() {
    let mut app = app();

    for id in [
        "cloud",
        "cloud_rain",
        "cloud_lightning",
        "cloud_fire",
        "cloud_acid",
        "cloud_lava",
        "cloud_ash",
        "cloud_magic",
        "cloud_snow",
        "cloud_rage",
    ] {
        press(&mut app, id);
        app.update();

        assert_eq!(app.world().resource::<Chosen>().0, Some(id));
        assert_eq!(app.world().resource::<Brush>().tool, Tool::Special);
        assert!(!app.world().resource::<Panning>().can_drag_map, "{id}");
    }

    press(&mut app, "cloud_rage");
    app.update();
    assert_eq!(app.world().resource::<Chosen>().0, None);
    assert!(app.world().resource::<Panning>().can_drag_map);
}

#[test]
fn a_click_calls_the_cloud_so_its_shadow_falls_on_the_tile_even_while_paused() {
    let mut app = app();
    app.world_mut().resource_mut::<WorldClock>().paused = true;
    press(&mut app, "cloud_lava");

    let pos = TilePos::new(17, 30);
    click(&mut app, pos);

    let called = sky(&mut app);
    assert_eq!(called.len(), 1);
    assert_eq!(called[0].0.kind, CloudKind::Lava);

    let map = app.world().resource::<TileMap>();
    let off = (called[0].1 - centre_of(map, pos)).abs();
    assert!(off.max_element() <= TILE_SIZE * 0.5, "{off}");

    for _ in 0..30 {
        app.update();
    }

    assert_eq!(sky(&mut app).len(), 1);
    assert_eq!(map_untouched(&app), Some(Ground::SoilLow));
}

#[test]
fn the_plain_cloud_button_calls_worldbox_cloud_of_life() {
    let mut app = app();
    app.world_mut().resource_mut::<WorldClock>().paused = true;
    press(&mut app, "cloud");
    click(&mut app, TilePos::new(5, 5));
    press(&mut app, "cloud_snow");
    click(&mut app, TilePos::new(30, 40));

    let mut kinds: Vec<CloudKind> = sky(&mut app)
        .into_iter()
        .map(|(cloud, _)| cloud.kind)
        .collect();
    kinds.sort_by_key(|kind| kind.id());

    assert_eq!(kinds, [CloudKind::Normal, CloudKind::Snow]);
}

fn map_untouched(app: &App) -> Option<Ground> {
    let map = app.world().resource::<TileMap>();
    let grounds: Vec<Ground> = (0..map.height)
        .flat_map(|y| (0..map.width).map(move |x| (x, y)))
        .map(|(x, y)| map.at(x, y).ground)
        .collect();

    grounds
        .iter()
        .all(|ground| *ground == grounds[0])
        .then_some(grounds[0])
}
