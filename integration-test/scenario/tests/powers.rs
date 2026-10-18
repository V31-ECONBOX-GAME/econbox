use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use econbox_brush::{Brush, BrushPlugin, Power, Shape, Tool, Undo};
use econbox_camera::Panning;
use econbox_clock::{ClockPlugin, WorldClock};
use econbox_creature::{Creature, CreaturePlugin, Spawn, Species};
use econbox_cursor::Hovered;
use econbox_tile::{Ground, Overlay, Tile};
use econbox_tilemap::{TileMap, TilePos};
use std::time::Duration;

fn centre() -> TilePos {
    TilePos::new(32, 32)
}

fn world(power: Power, radius: u32) -> App {
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
            64,
            64,
            Tile::new(Ground::SoilLow, Overlay::Grass),
        ))
        .insert_resource(Hovered {
            pos: None,
            tile: None,
        })
        .insert_resource(Brush {
            tool: Tool::Power(power),
            shape: Shape::Round,
            radius,
            flood: false,
        })
        .insert_resource(ButtonInput::<MouseButton>::default());
    app.update();
    app
}

fn drop_in(app: &mut App, species: Species, pos: TilePos) {
    app.world_mut().write_message(Spawn { species, pos });
    app.update();
}

fn press(app: &mut App, at: TilePos) {
    app.world_mut().resource_mut::<Hovered>().pos = Some(at);
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    app.update();
}

fn keep_holding(app: &mut App, frames: usize) {
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .clear_just_pressed(MouseButton::Left);

    for _ in 0..frames {
        app.update();
    }
}

fn release(app: &mut App) {
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .release(MouseButton::Left);
    app.update();
}

fn creatures(app: &mut App) -> Vec<Creature> {
    app.world_mut()
        .query::<&Creature>()
        .iter(app.world())
        .copied()
        .collect()
}

fn tile(app: &App, pos: TilePos) -> Tile {
    app.world()
        .resource::<TileMap>()
        .get(pos)
        .expect("on the map")
}

fn pause(app: &mut App) {
    app.world_mut().resource_mut::<WorldClock>().toggle();
}

#[test]
fn lightning_kills_what_it_strikes_and_spares_what_it_misses() {
    let mut app = world(Power::Lightning, 2);
    pause(&mut app);
    drop_in(&mut app, Species::Human, centre());
    drop_in(&mut app, Species::Elf, TilePos::new(55, 55));

    press(&mut app, centre());
    app.update();

    let alive = creatures(&mut app);

    assert_eq!(alive.len(), 1);
    assert_eq!(alive[0].species, Species::Elf);
    assert_eq!(tile(&app, centre()).overlay, Overlay::Infernal);
    assert_eq!(tile(&app, TilePos::new(55, 55)).overlay, Overlay::Grass);
}

#[test]
fn a_meteor_strike_can_be_taken_back_with_undo() {
    let mut app = world(Power::Meteor, 4);
    let before = app.world().resource::<TileMap>().clone();

    press(&mut app, centre());
    release(&mut app);

    assert_ne!(*app.world().resource::<TileMap>(), before);

    app.world_mut().write_message(Undo);
    app.update();

    assert_eq!(*app.world().resource::<TileMap>(), before);
}

#[test]
fn a_volcano_leaves_a_lava_lake_that_nothing_survives() {
    let mut app = world(Power::Volcano, 4);
    drop_in(&mut app, Species::Wolf, centre());

    press(&mut app, centre());
    release(&mut app);

    assert_eq!(tile(&app, centre()).ground, Ground::Lava3);
    assert!(creatures(&mut app).is_empty());
}

#[test]
fn a_blessing_makes_a_wounded_creature_whole() {
    let mut app = world(Power::Bless, 3);
    pause(&mut app);
    drop_in(&mut app, Species::Cow, centre());

    for mut creature in app
        .world_mut()
        .query::<&mut Creature>()
        .iter_mut(app.world_mut())
    {
        creature.health = 1.0;
    }

    press(&mut app, centre());
    app.update();

    assert_eq!(creatures(&mut app)[0].health, Species::Cow.health());
}

#[test]
fn a_purge_takes_the_monsters_and_leaves_everyone_else() {
    let mut app = world(Power::Purge, 3);
    pause(&mut app);
    drop_in(&mut app, Species::Zombie, centre());
    drop_in(&mut app, Species::Human, centre());

    press(&mut app, centre());
    app.update();

    let alive: Vec<Species> = creatures(&mut app)
        .iter()
        .map(|creature| creature.species)
        .collect();

    assert_eq!(alive, [Species::Human]);
}

#[test]
fn holding_a_curse_lays_it_on_again_at_its_own_pace() {
    let mut app = world(Power::Curse, 4);
    pause(&mut app);
    drop_in(&mut app, Species::Ogre, centre());

    press(&mut app, centre());
    keep_holding(&mut app, 40);

    let health = creatures(&mut app)[0].health;
    let full = Species::Ogre.health();

    assert!(health < full * 0.5_f32.powi(4), "{health}");
    assert!(health > full * 0.5_f32.powi(8), "{health}");
}

#[test]
fn a_deluge_drowns_a_creature_that_cannot_swim() {
    let mut app = world(Power::Deluge, 4);
    drop_in(&mut app, Species::Human, centre());

    press(&mut app, centre());
    keep_holding(&mut app, 4);
    release(&mut app);

    assert!(tile(&app, centre()).is_water());

    for _ in 0..120 {
        app.update();
    }

    assert!(creatures(&mut app).is_empty());
}

#[test]
fn a_paused_world_does_not_drown_anyone() {
    let mut app = world(Power::Deluge, 4);
    drop_in(&mut app, Species::Human, centre());
    pause(&mut app);

    press(&mut app, centre());
    keep_holding(&mut app, 4);
    release(&mut app);

    for _ in 0..120 {
        app.update();
    }

    assert_eq!(creatures(&mut app).len(), 1);
}

#[test]
fn rain_slowly_greens_a_desert() {
    let mut app = world(Power::Rain, 4);
    app.insert_resource(TileMap::filled(64, 64, Tile::bare(Ground::Sand)));

    press(&mut app, centre());
    keep_holding(&mut app, 40);
    release(&mut app);

    let map = app.world().resource::<TileMap>();
    let greened = (28..=36)
        .flat_map(|y| (28..=36).map(move |x| (x, y)))
        .filter(|&(x, y)| map.at(x, y).overlay == Overlay::Grass)
        .count();

    assert!(greened > 10, "{greened}");
    assert_eq!(map.at(0, 0), Tile::bare(Ground::Sand));
}
