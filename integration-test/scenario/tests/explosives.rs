use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use econbox_brush::{Bomb, BombDrop, Brush, BrushPlugin, Charge, Shape, Tool, Undo};
use econbox_camera::Panning;
use econbox_clock::{ClockPlugin, WorldClock};
use econbox_creature::{Creature, CreaturePlugin, Spawn, Species, tile_of};
use econbox_cursor::Hovered;
use econbox_tile::{Ground, Overlay, Tile, Wall};
use econbox_tilemap::{TileMap, TilePos};
use std::time::Duration;

fn centre() -> TilePos {
    TilePos::new(48, 48)
}

fn world(tool: Tool, radius: u32) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(ClockPlugin)
        .add_plugins(CreaturePlugin)
        .add_plugins(BrushPlugin)
        .init_resource::<Panning>()
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            50,
        )))
        .insert_resource(TileMap::filled(
            96,
            96,
            Tile::new(Ground::SoilLow, Overlay::Grass),
        ))
        .insert_resource(Hovered {
            pos: None,
            tile: None,
        })
        .insert_resource(Brush {
            tool,
            shape: Shape::Round,
            radius,
            flood: false,
        })
        .insert_resource(ButtonInput::<MouseButton>::default());
    app.update();
    app
}

fn pause(app: &mut App) {
    app.world_mut().resource_mut::<WorldClock>().toggle();
}

fn drop_in(app: &mut App, species: Species, pos: TilePos) {
    app.world_mut().write_message(Spawn { species, pos });
    app.update();
}

fn click(app: &mut App, at: TilePos) {
    app.world_mut().resource_mut::<Hovered>().pos = Some(at);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear();
}

fn plant(app: &mut App, bomb: Bomb, pos: TilePos) {
    app.world_mut().write_message(BombDrop { bomb, pos });
    app.update();
}

fn tile(app: &App, pos: TilePos) -> Tile {
    app.world()
        .resource::<TileMap>()
        .get(pos)
        .expect("on the map")
}

fn creatures(app: &mut App) -> Vec<Creature> {
    app.world_mut()
        .query::<&Creature>()
        .iter(app.world())
        .copied()
        .collect()
}

fn charges(app: &mut App) -> Vec<Charge> {
    app.world_mut()
        .query::<&Charge>()
        .iter(app.world())
        .copied()
        .collect()
}

fn grassy(app: &App, pos: TilePos) -> bool {
    tile(app, pos) == Tile::new(Ground::SoilLow, Overlay::Grass)
}

#[test]
fn a_grenade_craters_the_ground_and_kills_what_stands_there() {
    let mut app = world(Tool::Drop(Bomb::Grenade), 0);
    pause(&mut app);
    drop_in(&mut app, Species::Human, centre());
    drop_in(&mut app, Species::Elf, TilePos::new(80, 80));

    click(&mut app, centre());

    let alive = creatures(&mut app);

    assert_eq!(alive.len(), 1);
    assert_eq!(alive[0].species, Species::Elf);
    assert_eq!(tile(&app, centre()), Tile::bare(Ground::Sand));
    assert!(grassy(&app, TilePos::new(80, 80)));
}

#[test]
fn a_bomb_dropped_by_the_brush_can_be_taken_back_with_undo() {
    let mut app = world(Tool::Drop(Bomb::Tnt), 0);
    click(&mut app, centre());

    assert_eq!(charges(&mut app).len(), 1);
    assert!(grassy(&app, centre()));

    let mut app = world(Tool::Drop(Bomb::Atom), 0);
    click(&mut app, centre());

    assert!(!grassy(&app, centre()));

    app.world_mut().write_message(Undo);
    app.update();

    assert!(grassy(&app, centre()));
}

#[test]
fn a_wide_brush_drops_several_bombs_at_once() {
    let mut app = world(Tool::Drop(Bomb::Landmine), 16);
    pause(&mut app);
    click(&mut app, centre());

    assert_eq!(charges(&mut app).len(), econbox_brush::drops(16));
}

#[test]
fn a_timed_charge_waits_five_seconds_then_goes_off() {
    let mut app = world(Tool::Inspect, 0);
    plant(&mut app, Bomb::Timed, centre());

    assert_eq!(charges(&mut app).len(), 1);

    for _ in 0..80 {
        app.update();
    }

    assert_eq!(charges(&mut app).len(), 1);
    assert!(grassy(&app, centre()));

    for _ in 0..30 {
        app.update();
    }

    assert!(charges(&mut app).is_empty());
    assert_eq!(tile(&app, centre()).ground, Ground::ShallowWaters);
}

#[test]
fn a_paused_world_holds_the_timer_of_a_charge() {
    let mut app = world(Tool::Inspect, 0);
    pause(&mut app);
    plant(&mut app, Bomb::Timed, centre());

    for _ in 0..200 {
        app.update();
    }

    assert_eq!(charges(&mut app).len(), 1);
    assert!(grassy(&app, centre()));
}

#[test]
fn a_landmine_waits_until_something_steps_on_it() {
    let mut app = world(Tool::Inspect, 0);
    pause(&mut app);
    plant(&mut app, Bomb::Landmine, centre());

    for _ in 0..40 {
        app.update();
    }

    assert_eq!(charges(&mut app).len(), 1);

    drop_in(&mut app, Species::Wolf, centre());
    app.update();
    app.update();

    assert!(charges(&mut app).is_empty());
    assert!(creatures(&mut app).is_empty());
    assert!(!grassy(&app, centre()));
}

#[test]
fn a_blast_sets_off_the_dynamite_lying_inside_it() {
    let mut app = world(Tool::Inspect, 0);
    let far = TilePos::new(centre().x + 13, centre().y);
    plant(
        &mut app,
        Bomb::Tnt,
        TilePos::new(centre().x + 4, centre().y),
    );
    plant(&mut app, Bomb::Tnt, far);

    for _ in 0..40 {
        app.update();
    }

    assert_eq!(charges(&mut app).len(), 2);

    plant(&mut app, Bomb::Grenade, centre());

    for _ in 0..20 {
        app.update();
    }

    assert!(charges(&mut app).is_empty());
    assert_eq!(tile(&app, far).ground, Ground::ShallowWaters);
}

#[test]
fn a_water_bomb_goes_off_when_the_sea_reaches_it() {
    let mut app = world(Tool::Inspect, 0);
    plant(&mut app, Bomb::Water, centre());

    assert_eq!(charges(&mut app).len(), 1);

    app.world_mut().resource_mut::<TileMap>().set(
        centre().x,
        centre().y,
        Tile::bare(Ground::ShallowWaters),
    );

    for _ in 0..10 {
        app.update();
    }

    assert!(charges(&mut app).is_empty());
    assert_eq!(
        tile(&app, TilePos::new(centre().x + 3, centre().y)),
        Tile::bare(Ground::ShallowWaters)
    );
}

#[test]
fn a_wall_built_with_the_brush_keeps_a_creature_penned_in() {
    let mut app = world(Tool::Wall(Wall::Order), 0);
    pause(&mut app);
    let (cx, cy) = (centre().x as i32, centre().y as i32);

    for dy in -3..=3_i32 {
        for dx in -3..=3_i32 {
            if dx.abs() == 3 || dy.abs() == 3 {
                click(&mut app, TilePos::new((cx + dx) as u32, (cy + dy) as u32));
            }
        }
    }

    assert_eq!(
        tile(&app, TilePos::new(centre().x + 3, centre().y)).wall,
        Some(Wall::Order)
    );

    drop_in(&mut app, Species::Rabbit, centre());
    pause(&mut app);

    for _ in 0..600 {
        app.update();
    }

    let (map, mut rabbits) = (
        app.world().resource::<TileMap>().clone(),
        app.world_mut().query::<(&Creature, &Transform)>(),
    );
    let (rabbit, transform) = rabbits.single(app.world()).expect("one rabbit");
    let at = tile_of(&map, transform.translation.truncate()).expect("on the map");

    assert!(rabbit.alive());
    assert!(
        (at.x as i32 - cx).abs() < 3 && (at.y as i32 - cy).abs() < 3,
        "{at:?}"
    );
}

#[test]
fn raising_a_mountain_builds_a_summit_and_lowering_takes_it_back() {
    let mut app = world(Tool::Raise, 0);
    app.world_mut().resource_mut::<TileMap>().set(
        centre().x,
        centre().y,
        Tile::bare(Ground::Mountains),
    );

    click(&mut app, centre());
    assert_eq!(tile(&app, centre()).ground, Ground::Summit);

    click(&mut app, centre());
    assert_eq!(tile(&app, centre()).ground, Ground::Summit);

    app.world_mut().resource_mut::<Brush>().tool = Tool::Lower;
    click(&mut app, centre());
    assert_eq!(tile(&app, centre()).ground, Ground::Mountains);
}
