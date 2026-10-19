use bevy::prelude::*;
use econbox_brush::{Brush, BrushPlugin, Tool};
use econbox_camera::Panning;
use econbox_clock::{ClockPlugin, WorldClock};
use econbox_creature::{Census, Creature, CreaturePlugin, Spawn, Species, centre_of};
use econbox_cursor::Hovered;
use econbox_tile::{Ground, Overlay, Tile};
use econbox_tilemap::{TileMap, TilePos};

fn world(tile: Tile) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(ClockPlugin)
        .add_plugins(CreaturePlugin)
        .insert_resource(TileMap::filled(64, 64, tile));
    app.update();
    app
}

fn grassland() -> App {
    world(Tile::new(Ground::SoilLow, Overlay::Grass))
}

fn drop_in(app: &mut App, species: Species, pos: TilePos) {
    app.world_mut().write_message(Spawn { species, pos });
    app.update();
}

fn creatures(app: &mut App) -> Vec<Creature> {
    app.world_mut()
        .query::<&Creature>()
        .iter(app.world())
        .copied()
        .collect()
}

#[test]
fn clicking_a_species_puts_one_of_them_on_the_map() {
    let mut app = grassland();
    drop_in(&mut app, Species::Human, TilePos::new(20, 20));

    let alive = creatures(&mut app);

    assert_eq!(alive.len(), 1);
    assert_eq!(alive[0].species, Species::Human);
    assert_eq!(alive[0].health, Species::Human.health());
}

#[test]
fn a_creature_lands_on_the_tile_that_was_clicked() {
    let mut app = grassland();
    app.world_mut().resource_mut::<WorldClock>().toggle();
    let pos = TilePos::new(12, 34);
    drop_in(&mut app, Species::Orc, pos);

    let map = app.world().resource::<TileMap>().clone();
    let expected = centre_of(&map, pos);
    let at = app
        .world_mut()
        .query_filtered::<&Transform, With<Creature>>()
        .iter(app.world())
        .next()
        .copied()
        .expect("a creature");

    assert_eq!(at.translation.truncate(), expected);
}

#[test]
fn a_human_refuses_to_be_dropped_into_the_open_sea() {
    let mut app = world(Tile::bare(Ground::DeepOcean));
    drop_in(&mut app, Species::Human, TilePos::new(20, 20));

    assert!(creatures(&mut app).is_empty());
}

#[test]
fn a_fish_is_happy_in_the_sea_and_refused_on_the_grass() {
    let mut sea = world(Tile::bare(Ground::DeepOcean));
    drop_in(&mut sea, Species::Fish, TilePos::new(20, 20));
    assert_eq!(creatures(&mut sea).len(), 1);

    let mut land = grassland();
    drop_in(&mut land, Species::Fish, TilePos::new(20, 20));
    assert!(creatures(&mut land).is_empty());
}

#[test]
fn dropping_outside_the_map_puts_nothing_anywhere() {
    let mut app = grassland();
    drop_in(&mut app, Species::Human, TilePos::new(999, 999));

    assert!(creatures(&mut app).is_empty());
}

#[test]
fn a_held_click_can_stack_a_crowd_on_one_tile() {
    let mut app = grassland();

    for _ in 0..12 {
        drop_in(&mut app, Species::Sheep, TilePos::new(30, 30));
    }

    assert_eq!(creatures(&mut app).len(), 12);
    assert_eq!(app.world().resource::<Census>().alive, 12);
}

#[test]
fn every_species_can_be_placed_on_the_ground_it_belongs_to() {
    for species in econbox_creature::SPECIES {
        let tile = if species.swims() && !species.walks() {
            Tile::bare(Ground::DeepOcean)
        } else {
            Tile::new(Ground::SoilLow, Overlay::Grass)
        };

        let mut app = world(tile);
        drop_in(&mut app, species, TilePos::new(8, 8));

        assert_eq!(creatures(&mut app).len(), 1, "{}", species.name());
    }
}

#[test]
fn a_creature_wanders_off_the_tile_it_started_on() {
    let mut app = grassland();
    drop_in(&mut app, Species::Rabbit, TilePos::new(32, 32));

    let start = app
        .world_mut()
        .query_filtered::<&Transform, With<Creature>>()
        .iter(app.world())
        .next()
        .copied()
        .expect("a rabbit")
        .translation
        .truncate();

    for _ in 0..240 {
        app.update();
    }

    let now = app
        .world_mut()
        .query_filtered::<&Transform, With<Creature>>()
        .iter(app.world())
        .next()
        .copied()
        .expect("a rabbit")
        .translation
        .truncate();

    assert_ne!(now, start);
}

#[test]
fn a_paused_world_holds_every_creature_still() {
    let mut app = grassland();
    drop_in(&mut app, Species::Wolf, TilePos::new(32, 32));
    app.world_mut().resource_mut::<WorldClock>().toggle();

    let before = creatures(&mut app);

    for _ in 0..120 {
        app.update();
    }

    assert_eq!(creatures(&mut app), before);
}

#[test]
fn a_creature_that_runs_out_of_health_is_taken_off_the_map() {
    let mut app = grassland();
    drop_in(&mut app, Species::Chicken, TilePos::new(10, 10));
    drop_in(&mut app, Species::Chicken, TilePos::new(11, 10));

    let doomed = app
        .world_mut()
        .query_filtered::<Entity, With<Creature>>()
        .iter(app.world())
        .next()
        .expect("a chicken");

    app.world_mut()
        .get_mut::<Creature>(doomed)
        .expect("alive")
        .health = 0.0;
    app.update();

    assert_eq!(creatures(&mut app).len(), 1);
    assert_eq!(app.world().resource::<Census>().alive, 1);
    assert!(app.world().get::<Creature>(doomed).is_none());
}

#[test]
fn hunger_wears_a_creature_down_once_the_food_is_gone() {
    let mut creature = Creature::new(Species::Chicken);
    creature.food = 0.0;

    assert!(creature.starving());
    assert!(creature.alive());

    creature.health = 0.0;

    assert!(!creature.alive());
}

#[test]
fn holding_the_button_with_a_spawn_tool_drops_a_creature_where_the_pointer_is() {
    let pos = TilePos::new(20, 20);
    let mut app = App::new();
    let mut buttons = ButtonInput::<MouseButton>::default();
    buttons.press(MouseButton::Left);

    app.add_plugins(MinimalPlugins)
        .add_plugins(ClockPlugin)
        .add_plugins(CreaturePlugin)
        .add_plugins(BrushPlugin)
        .init_resource::<Panning>()
        .insert_resource(TileMap::filled(
            64,
            64,
            Tile::new(Ground::SoilLow, Overlay::Grass),
        ))
        .insert_resource(Hovered {
            pos: Some(pos),
            tile: None,
        })
        .insert_resource(Brush {
            tool: Tool::Spawn(Species::Human),
            ..default()
        })
        .insert_resource(buttons);

    app.update();
    app.update();

    let alive = creatures(&mut app);

    assert_eq!(alive.len(), 1);
    assert_eq!(alive[0].species, Species::Human);
}

#[test]
fn a_spawn_tool_leaves_the_ground_exactly_as_it_found_it() {
    let mut app = App::new();
    let map = TileMap::filled(64, 64, Tile::new(Ground::SoilLow, Overlay::Grass));
    let before = map.clone();
    let mut buttons = ButtonInput::<MouseButton>::default();
    buttons.press(MouseButton::Left);

    app.add_plugins(MinimalPlugins)
        .add_plugins(ClockPlugin)
        .add_plugins(CreaturePlugin)
        .add_plugins(BrushPlugin)
        .init_resource::<Panning>()
        .insert_resource(map)
        .insert_resource(Hovered {
            pos: Some(TilePos::new(20, 20)),
            tile: None,
        })
        .insert_resource(Brush {
            tool: Tool::Spawn(Species::Wolf),
            radius: 8,
            ..default()
        })
        .insert_resource(buttons);

    for _ in 0..6 {
        app.update();
    }

    assert_eq!(*app.world().resource::<TileMap>(), before);
    assert!(!creatures(&mut app).is_empty());
}

#[test]
fn only_the_inspect_tool_lets_the_cursor_grab_the_map() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(ClockPlugin)
        .add_plugins(CreaturePlugin)
        .add_plugins(BrushPlugin)
        .init_resource::<Panning>()
        .insert_resource(TileMap::filled(
            16,
            16,
            Tile::new(Ground::SoilLow, Overlay::Grass),
        ))
        .insert_resource(ButtonInput::<MouseButton>::default())
        .insert_resource(Hovered::default());

    app.update();
    assert!(app.world().resource::<Panning>().can_drag_map);

    app.world_mut().resource_mut::<Brush>().tool = Tool::Spawn(Species::Cat);
    app.update();
    assert!(!app.world().resource::<Panning>().can_drag_map);
}
