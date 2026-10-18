//! Trees and fruit bushes that spread over fertile biomes while the world clock runs.

use bevy::prelude::*;
use econbox_clock::WorldClock;
use econbox_tile::{Flora, Tile};
use econbox_tilemap::{Painted, TileMap, TilePos};

pub const SAMPLES: u32 = 64;
pub const CATCH_UP: u64 = 8;
pub const CROWD: usize = 3;
pub const WILD: f32 = 0.004;
pub const SPREAD: f32 = 0.06;
pub const RIPEN: f32 = 0.35;
pub const BUSH_ODDS: u32 = 5;

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Growth {
    pub seen: u64,
    pub salt: u32,
}

pub struct FloraPlugin;

impl Plugin for FloraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Growth>()
            .add_message::<Painted>()
            .add_systems(
                Update,
                grow.run_if(resource_exists::<TileMap>)
                    .run_if(resource_exists::<WorldClock>),
            );
    }
}

pub fn mix(x: u32, y: u32, salt: u32) -> u32 {
    let mut h =
        x.wrapping_mul(0x27d4_eb2d) ^ y.wrapping_mul(0x1656_67b1) ^ salt.wrapping_mul(0x9e37_79b9);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297a_2d39);
    h ^ (h >> 15)
}

fn roll(noise: u32) -> f32 {
    (noise >> 8) as f32 / (1 << 24) as f32
}

pub fn around(map: &TileMap, pos: TilePos) -> (usize, usize) {
    let mut trees = 0;
    let mut bushes = 0;

    for dy in -1_i32..=1 {
        for dx in -1_i32..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let (Ok(x), Ok(y)) = (
                u32::try_from(pos.x as i32 + dx),
                u32::try_from(pos.y as i32 + dy),
            ) else {
                continue;
            };

            match map.get(TilePos::new(x, y)).and_then(|tile| tile.flora) {
                Some(Flora::Bush) => bushes += 1,
                Some(_) => trees += 1,
                None => {}
            }
        }
    }

    (trees, bushes)
}

pub fn sprout(map: &TileMap, pos: TilePos, noise: u32) -> Option<Tile> {
    let tile = map.get(pos)?;

    if !tile.overlay.grows() {
        return None;
    }

    let chance = roll(noise);
    let fertility = tile.fertility();

    match tile.flora {
        Some(Flora::Sapling) => {
            (chance < RIPEN * fertility).then(|| tile.with_flora(Some(Flora::Tree)))
        }
        Some(_) => None,
        None => {
            let (trees, bushes) = around(map, pos);
            let near = trees + bushes;

            if near >= CROWD || chance >= fertility * (WILD + SPREAD * near as f32) {
                return None;
            }

            let flora = if bushes > trees || (near == 0 && noise.is_multiple_of(BUSH_ODDS)) {
                Flora::Bush
            } else {
                Flora::Sapling
            };

            Some(tile.with_flora(Some(flora)))
        }
    }
}

pub fn season(map: &TileMap, salt: u32) -> Vec<(TilePos, Tile)> {
    let count = (map.cells() as u32 / SAMPLES).max(1);

    (0..count)
        .filter_map(|draw| {
            let pos = TilePos::new(
                mix(draw, salt, 1) % map.width,
                mix(draw, salt, 2) % map.height,
            );
            let next = sprout(map, pos, mix(draw, salt, 3))?;

            (map.get(pos) != Some(next)).then_some((pos, next))
        })
        .collect()
}

fn grow(
    clock: Res<WorldClock>,
    mut growth: ResMut<Growth>,
    mut map: ResMut<TileMap>,
    mut painted: MessageWriter<Painted>,
) {
    if clock.ticks == growth.seen {
        return;
    }

    let due = clock.ticks.saturating_sub(growth.seen).min(CATCH_UP);
    growth.seen = clock.ticks;
    let mut cells = Vec::new();

    for _ in 0..due {
        growth.salt = growth.salt.wrapping_add(1);

        for (pos, tile) in season(&map, growth.salt) {
            map.set(pos.x, pos.y, tile);
            cells.push(pos);
        }
    }

    if !cells.is_empty() {
        painted.write(Painted::new(cells));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use econbox_tile::{Ground, Overlay};

    fn meadow(width: u32, height: u32, overlay: Overlay) -> TileMap {
        TileMap::filled(width, height, Tile::new(Ground::SoilLow, overlay))
    }

    fn wooded(map: &TileMap) -> usize {
        map.tiles.iter().filter(|tile| tile.flora.is_some()).count()
    }

    fn seasons(map: &mut TileMap, count: u32) {
        for salt in 0..count {
            for (pos, tile) in season(map, salt) {
                map.set(pos.x, pos.y, tile);
            }
        }
    }

    #[test]
    fn a_meadow_grows_a_scattered_wood_over_the_years() {
        let mut map = meadow(64, 64, Overlay::Grass);
        seasons(&mut map, 600);
        let share = wooded(&map) as f32 / map.cells() as f32;

        assert!(share > 0.1 && share < 0.7, "{share}");
        assert!(map.tiles.iter().any(|tile| tile.flora == Some(Flora::Tree)));
        assert!(map.tiles.iter().any(|tile| tile.flora == Some(Flora::Bush)));
    }

    #[test]
    fn barren_biomes_and_open_water_stay_bare() {
        for overlay in [
            Overlay::Desert,
            Overlay::Void,
            Overlay::Bare,
            Overlay::Infernal,
        ] {
            let mut map = meadow(32, 32, overlay);
            seasons(&mut map, 300);

            assert_eq!(wooded(&map), 0, "{}", overlay.name());
        }

        let mut sea = TileMap::filled(32, 32, Tile::bare(Ground::DeepOcean));
        seasons(&mut sea, 300);

        assert_eq!(wooded(&sea), 0);
    }

    #[test]
    fn a_rich_biome_outgrows_a_poor_one() {
        let mut rich = meadow(64, 64, Overlay::Grass);
        let mut poor = meadow(64, 64, Overlay::Crystal);
        seasons(&mut rich, 200);
        seasons(&mut poor, 200);

        assert!(
            wooded(&rich) > wooded(&poor),
            "{} vs {}",
            wooded(&rich),
            wooded(&poor)
        );
    }

    #[test]
    fn saplings_ripen_into_trees_and_grown_plants_stay_put() {
        let mut map = meadow(3, 3, Overlay::Grass);
        let centre = TilePos::new(1, 1);
        let grass = map.at(1, 1);
        map.set(1, 1, grass.with_flora(Some(Flora::Sapling)));

        assert_eq!(
            sprout(&map, centre, 0).and_then(|tile| tile.flora),
            Some(Flora::Tree)
        );
        assert_eq!(sprout(&map, centre, u32::MAX), None);

        map.set(1, 1, grass.with_flora(Some(Flora::Bush)));

        assert_eq!(sprout(&map, centre, 0), None);
    }

    #[test]
    fn a_crowded_patch_leaves_room_between_its_trees() {
        let mut map = meadow(3, 3, Overlay::Grass);
        let tree = map.at(0, 0).with_flora(Some(Flora::Tree));

        for (x, y) in [(0, 0), (1, 0), (2, 0)] {
            map.set(x, y, tree);
        }

        assert_eq!(around(&map, TilePos::new(1, 1)), (3, 0));
        assert_eq!(sprout(&map, TilePos::new(1, 1), 0), None);
    }

    #[test]
    fn bushes_seed_bushes_and_trees_seed_saplings() {
        let mut map = meadow(3, 3, Overlay::Grass);
        let grass = map.at(0, 0);
        map.set(0, 0, grass.with_flora(Some(Flora::Bush)));

        assert_eq!(
            sprout(&map, TilePos::new(1, 1), 0).and_then(|tile| tile.flora),
            Some(Flora::Bush)
        );

        map.set(0, 0, grass.with_flora(Some(Flora::Tree)));

        assert_eq!(
            sprout(&map, TilePos::new(1, 1), 0).and_then(|tile| tile.flora),
            Some(Flora::Sapling)
        );
    }

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(FloraPlugin)
            .init_resource::<WorldClock>()
            .insert_resource(meadow(64, 64, Overlay::Clover));
        app
    }

    #[test]
    fn the_forest_only_grows_while_the_clock_runs() {
        let mut app = app();
        app.update();

        for _ in 0..40 {
            app.update();
        }

        assert_eq!(wooded(app.world().resource::<TileMap>()), 0);

        for tick in 1..=200 {
            app.world_mut().resource_mut::<WorldClock>().ticks = tick;
            app.update();
        }

        assert!(wooded(app.world().resource::<TileMap>()) > 0);
    }

    #[test]
    fn growth_reports_the_cells_it_changed_for_repainting() {
        let mut app = app();
        let mut reported = 0;

        for tick in 1..=200 {
            app.world_mut().resource_mut::<WorldClock>().ticks = tick;
            app.update();

            let messages = app.world().resource::<Messages<Painted>>();
            reported += messages
                .iter_current_update_messages()
                .map(|painted| painted.cells.len())
                .sum::<usize>();
        }

        assert!(reported >= wooded(app.world().resource::<TileMap>()));
        assert!(reported > 0);
    }

    #[test]
    fn a_long_frame_catches_up_only_a_few_seasons() {
        let mut app = app();
        app.world_mut().resource_mut::<WorldClock>().ticks = 1_000_000;
        app.update();

        assert_eq!(app.world().resource::<Growth>().salt, CATCH_UP as u32);
        assert_eq!(app.world().resource::<Growth>().seen, 1_000_000);
    }
}
