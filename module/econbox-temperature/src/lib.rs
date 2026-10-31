//! WorldBox tile temperature: snow caps on summits, freezing and thawing ages, and frozen tile health.

use bevy::prelude::*;
use econbox_age::WorldAge;
use econbox_clock::{Frames, WorldClock};
use econbox_noise::Randy;
use econbox_tile::Ground;
use econbox_tilemap::{Flash, NewWorld, Painted, TileMap, TilePos};

pub const TILE_HEALTH: i32 = 10;
pub const FAST_FREEZE_ODDS: f32 = 0.35;
pub const FAST_THAW_ODDS: f32 = 0.2;
pub const FREEZE_FLASH: i32 = 20;
pub const FREEZE_INTERVAL: f32 = 0.1;
pub const THAW_INTERVAL: f32 = 0.01;
pub const SUMMIT_WAIT: (f32, f32) = (10.0, 60.0);
pub const SUMMIT_SPARE: f32 = 0.8;
pub const DRIFT_DAMAGE: i32 = 5;
pub const CHECKS_PER_CHUNK: usize = 10;
pub const CHUNK_SIZE: u32 = 16;
pub const MOUNTAIN_STAY: f32 = 0.9;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TemperatureLaws {
    pub forever_cold: bool,
}

#[derive(Resource, Debug, Clone)]
pub struct Frost {
    pub health: Vec<i32>,
    pub order: Vec<usize>,
    pub next: usize,
    pub size: (u32, u32),
    pub summit_timer: f32,
    pub freeze_timer: f32,
    pub thaw_timer: f32,
    pub randy: Randy,
}

impl Default for Frost {
    fn default() -> Self {
        Self {
            health: Vec::new(),
            order: Vec::new(),
            next: 0,
            size: (0, 0),
            summit_timer: 0.0,
            freeze_timer: FREEZE_INTERVAL,
            thaw_timer: THAW_INTERVAL,
            randy: Randy::new(0x00f0_57ed),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Touched {
    pub cells: Vec<TilePos>,
    pub flashes: Vec<TilePos>,
}

impl Touched {
    fn mark(&mut self, pos: TilePos) {
        self.cells.push(pos);
        self.flashes.push(pos);
    }
}

impl Frost {
    pub fn fit(&mut self, map: &TileMap) {
        let size = (map.width, map.height);

        if self.size == size && self.health.len() == map.cells() {
            return;
        }

        self.size = size;
        self.health = vec![TILE_HEALTH; map.cells()];
        self.order.clear();
        self.next = 0;
        self.freeze_timer = FREEZE_INTERVAL;
        self.thaw_timer = THAW_INTERVAL;
    }

    fn checks(&self, map: &TileMap) -> usize {
        map.width.div_ceil(CHUNK_SIZE) as usize * CHECKS_PER_CHUNK
    }

    fn next_tiles(&mut self, map: &TileMap) -> Vec<usize> {
        if self.order.is_empty() || self.next + 1 >= self.order.len() {
            if self.order.is_empty() {
                self.order = (0..map.cells()).collect();
            }
            self.next = 0;
        }

        let len = self.order.len();
        let count = self.checks(map).min(len - self.next);
        let mut picked = Vec::with_capacity(count);

        for _ in 0..count {
            let swap = self.randy.index_from(self.next, len);
            self.order.swap(self.next, swap);
            picked.push(self.order[self.next]);
            self.next += 1;
        }

        picked
    }
}

fn pos_of(map: &TileMap, index: usize) -> TilePos {
    TilePos::new(index as u32 % map.width, index as u32 / map.width)
}

fn around(map: &TileMap, pos: TilePos, all: bool) -> Vec<TilePos> {
    let steps: &[(i32, i32)] = if all {
        &[
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ]
    } else {
        &[(0, -1), (-1, 0), (1, 0), (0, 1)]
    };

    steps
        .iter()
        .filter_map(|(dx, dy)| {
            let x = u32::try_from(pos.x as i32 + dx).ok()?;
            let y = u32::try_from(pos.y as i32 + dy).ok()?;
            (x < map.width && y < map.height).then_some(TilePos::new(x, y))
        })
        .collect()
}

fn fast(ground: Ground) -> bool {
    ground == Ground::ShallowWaters
}

pub fn freeze(map: &mut TileMap, frost: &mut Frost, pos: TilePos, out: &mut Touched) -> bool {
    frost.fit(map);
    let Some(tile) = map.get(pos) else {
        return false;
    };

    if !tile.can_freeze() {
        return false;
    }

    map.set(pos.x, pos.y, tile.with_frozen(true));

    if fast(tile.ground) {
        for near in around(map, pos, false) {
            let Some(next) = map.get(near) else { continue };
            if fast(next.ground) && next.can_freeze() && frost.randy.random_chance(FAST_FREEZE_ODDS)
            {
                freeze(map, frost, near, out);
            }
        }
    }

    let index = map.index(pos.x, pos.y);
    frost.health[index] = TILE_HEALTH;
    out.mark(pos);
    true
}

pub fn thaw(map: &mut TileMap, frost: &mut Frost, pos: TilePos, damage: i32, out: &mut Touched) {
    frost.fit(map);
    let Some(tile) = map.get(pos) else { return };

    if !tile.frozen || tile.forever_frozen() {
        return;
    }

    let index = map.index(pos.x, pos.y);

    if frost.health[index] > 0 {
        frost.health[index] -= damage;
        if frost.health[index] > 0 {
            return;
        }
    }

    map.set(pos.x, pos.y, tile.with_frozen(false));
    frost.health[index] = TILE_HEALTH;
    out.mark(pos);

    if !fast(tile.ground) {
        return;
    }

    for near in around(map, pos, false) {
        let Some(next) = map.get(near) else { continue };
        if next.frozen && !next.forever_frozen() && frost.randy.random_chance(FAST_THAW_ODDS) {
            thaw(map, frost, near, damage, out);
        }
    }
}

pub fn is_frozen(map: &TileMap, pos: TilePos) -> bool {
    map.get(pos)
        .is_some_and(|tile| tile.frozen || tile.forever_frozen())
}

pub fn due(timer: &mut f32, dt: f32, interval: f32) -> bool {
    if *timer > 0.0 {
        *timer -= dt;
        if *timer > 0.0 {
            return false;
        }
    }

    *timer += interval;
    true
}

pub fn freeze_summits(
    map: &mut TileMap,
    frost: &mut Frost,
    age: econbox_age::Age,
    dt: f32,
    out: &mut Touched,
) {
    if age.sunny() {
        return;
    }

    if frost.summit_timer > 0.0 {
        frost.summit_timer -= dt;
        return;
    }

    frost.summit_timer = frost.randy.random_float(SUMMIT_WAIT.0, SUMMIT_WAIT.1);
    let summits: Vec<TilePos> = (0..map.cells())
        .filter(|&index| map.tiles[index].ground == Ground::Summit)
        .map(|index| pos_of(map, index))
        .collect();

    for pos in summits {
        if !frost.randy.random_chance(SUMMIT_SPARE)
            && map.get(pos).is_some_and(|tile| tile.can_freeze())
        {
            freeze(map, frost, pos, out);
        }
    }
}

pub fn freeze_world(map: &mut TileMap, frost: &mut Frost, out: &mut Touched) {
    for index in frost.next_tiles(map) {
        let pos = pos_of(map, index);
        if map.get(pos).is_some_and(|tile| tile.can_freeze()) {
            freeze(map, frost, pos, out);
        }
    }
}

fn mountain_thaws(map: &TileMap, frost: &mut Frost, pos: TilePos, age: econbox_age::Age) -> bool {
    if age.thaws_mountains() {
        return true;
    }

    let summit = map
        .get(pos)
        .is_some_and(|tile| tile.ground == Ground::Summit);

    if summit && !age.sunny() {
        return false;
    }

    if frost.randy.random_chance(MOUNTAIN_STAY) {
        return false;
    }

    around(map, pos, true)
        .into_iter()
        .any(|near| !is_frozen(map, near))
}

pub fn thaw_world(map: &mut TileMap, frost: &mut Frost, age: econbox_age::Age, out: &mut Touched) {
    for index in frost.next_tiles(map) {
        let pos = pos_of(map, index);
        let Some(tile) = map.get(pos) else { continue };

        if !tile.frozen {
            continue;
        }

        let mountain = matches!(tile.ground, Ground::Mountains | Ground::Summit);

        if !mountain || mountain_thaws(map, frost, pos, age) {
            thaw(
                map,
                frost,
                pos,
                DRIFT_DAMAGE + age.temperature_damage_bonus(),
                out,
            );
        }
    }
}

pub fn step(
    map: &mut TileMap,
    frost: &mut Frost,
    age: econbox_age::Age,
    laws: TemperatureLaws,
    dt: f32,
    out: &mut Touched,
) {
    frost.fit(map);

    if due(&mut frost.freeze_timer, dt, FREEZE_INTERVAL) {
        freeze_summits(map, frost, age, dt, out);
        if age.freezes_world() {
            freeze_world(map, frost, out);
        }
    }

    if due(&mut frost.thaw_timer, dt, THAW_INTERVAL) && age.thaws_world() && !laws.forever_cold {
        thaw_world(map, frost, age, out);
    }
}

pub struct TemperaturePlugin;

impl Plugin for TemperaturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldClock>()
            .init_resource::<Frames>()
            .init_resource::<WorldAge>()
            .init_resource::<Frost>()
            .init_resource::<TemperatureLaws>()
            .add_message::<Painted>()
            .add_message::<Flash>()
            .add_message::<NewWorld>()
            .add_systems(
                Update,
                (thaw_all, temper)
                    .chain()
                    .run_if(resource_exists::<TileMap>),
            );
    }
}

fn thaw_all(mut fresh: MessageReader<NewWorld>, mut frost: ResMut<Frost>) {
    if fresh.read().count() > 0 {
        *frost = Frost::default();
    }
}

fn temper(
    frames: Res<Frames>,
    clock: Res<WorldClock>,
    age: Res<WorldAge>,
    laws: Res<TemperatureLaws>,
    mut frost: ResMut<Frost>,
    mut map: ResMut<TileMap>,
    mut painted: MessageWriter<Painted>,
    mut flashes: MessageWriter<Flash>,
) {
    if !clock.running() || frames.due == 0 {
        return;
    }

    let mut out = Touched::default();

    for _ in 0..frames.due {
        step(&mut map, &mut frost, age.0, *laws, clock.step(), &mut out);
    }

    for pos in out.flashes {
        flashes.write(Flash {
            pos,
            strength: FREEZE_FLASH,
        });
    }

    if !out.cells.is_empty() {
        painted.write(Painted::new(out.cells));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use econbox_age::Age;
    use econbox_tile::{Overlay, Tile, Wall};

    fn map(ground: Ground) -> TileMap {
        TileMap::filled(32, 32, Tile::bare(ground))
    }

    #[test]
    fn soil_freezes_into_its_snow_top_and_flashes() {
        let mut map = map(Ground::SoilLow);
        let mut frost = Frost::default();
        let mut out = Touched::default();

        assert!(freeze(&mut map, &mut frost, TilePos::new(3, 3), &mut out));
        assert!(map.at(3, 3).frozen);
        assert_eq!(out.flashes, [TilePos::new(3, 3)]);
        assert!(!freeze(&mut map, &mut frost, TilePos::new(3, 3), &mut out));
    }

    #[test]
    fn deep_water_lava_walls_and_permafrost_never_freeze() {
        let mut frost = Frost::default();
        let mut out = Touched::default();

        for tile in [
            Tile::bare(Ground::DeepOcean),
            Tile::bare(Ground::Lava3),
            Tile::bare(Ground::SoilLow).with_wall(Some(Wall::Order)),
            Tile::new(Ground::SoilLow, Overlay::Frost),
            Tile::new(Ground::SoilLow, Overlay::Infernal),
        ] {
            let mut map = TileMap::filled(4, 4, tile);
            assert!(
                !freeze(&mut map, &mut frost, TilePos::new(1, 1), &mut out),
                "{tile:?}"
            );
        }
    }

    #[test]
    fn shallow_water_ice_creeps_into_its_neighbours() {
        let mut map = map(Ground::ShallowWaters);
        let mut frost = Frost::default();
        let mut out = Touched::default();
        freeze(&mut map, &mut frost, TilePos::new(16, 16), &mut out);

        let frozen = map.tiles.iter().filter(|tile| tile.frozen).count();
        assert!(frozen > 1);
        assert!(frozen < map.cells());
    }

    #[test]
    fn a_frozen_tile_takes_ten_points_of_thaw_to_melt() {
        let mut map = map(Ground::Hills);
        let mut frost = Frost::default();
        let mut out = Touched::default();
        let pos = TilePos::new(5, 5);
        freeze(&mut map, &mut frost, pos, &mut out);

        thaw(&mut map, &mut frost, pos, 5, &mut out);
        assert!(map.at(5, 5).frozen);
        thaw(&mut map, &mut frost, pos, 5, &mut out);
        assert!(!map.at(5, 5).frozen);

        freeze(&mut map, &mut frost, pos, &mut out);
        thaw(&mut map, &mut frost, pos, 99, &mut out);
        assert!(!map.at(5, 5).frozen);
    }

    #[test]
    fn summits_gather_snow_outside_the_age_of_sun() {
        let mut map = map(Ground::Summit);
        let mut frost = Frost::default();
        let mut out = Touched::default();
        freeze_summits(&mut map, &mut frost, Age::Hope, 0.02, &mut out);

        let share = map.tiles.iter().filter(|tile| tile.frozen).count() as f32 / map.cells() as f32;
        assert!((0.12..0.3).contains(&share), "{share}");
        assert!(frost.summit_timer >= 10.0 && frost.summit_timer < 60.0);

        let mut sunny = self::map(Ground::Summit);
        let mut frost = Frost::default();
        freeze_summits(&mut sunny, &mut frost, Age::Sun, 0.02, &mut out);
        assert!(sunny.tiles.iter().all(|tile| !tile.frozen));
    }

    #[test]
    fn an_ice_age_freezes_the_land_bit_by_bit() {
        let mut map = map(Ground::SoilLow);
        let mut frost = Frost::default();
        let mut out = Touched::default();

        for _ in 0..200 {
            step(
                &mut map,
                &mut frost,
                Age::Ice,
                TemperatureLaws::default(),
                0.02,
                &mut out,
            );
        }

        let frozen = map.tiles.iter().filter(|tile| tile.frozen).count();
        assert!(frozen > 100, "{frozen}");
    }

    #[test]
    fn the_age_of_hope_melts_the_lowlands_but_keeps_its_summits_white() {
        let mut map = map(Ground::SoilLow);
        for x in 0..32 {
            map.set(x, 0, Tile::bare(Ground::Summit));
        }
        let mut frost = Frost::default();
        let mut out = Touched::default();

        for index in 0..map.cells() {
            let pos = pos_of(&map, index);
            freeze(&mut map, &mut frost, pos, &mut out);
        }

        for _ in 0..400 {
            step(
                &mut map,
                &mut frost,
                Age::Hope,
                TemperatureLaws::default(),
                0.02,
                &mut out,
            );
        }

        assert!((0..32).all(|x| map.at(x, 0).frozen));
        assert!((1..32).all(|y| (0..32).all(|x| !map.at(x, y).frozen)));
    }

    #[test]
    fn forever_cold_stops_the_thaw() {
        let mut map = map(Ground::SoilLow);
        let mut frost = Frost::default();
        let mut out = Touched::default();
        freeze(&mut map, &mut frost, TilePos::new(1, 1), &mut out);

        for _ in 0..2000 {
            step(
                &mut map,
                &mut frost,
                Age::Hope,
                TemperatureLaws { forever_cold: true },
                0.02,
                &mut out,
            );
        }

        assert!(map.at(1, 1).frozen);
    }

    #[test]
    fn a_256_map_checks_one_hundred_and_sixty_tiles_per_pass() {
        let map = TileMap::filled(256, 256, Tile::bare(Ground::Sand));
        let mut frost = Frost::default();
        frost.fit(&map);

        assert_eq!(frost.next_tiles(&map).len(), 160);
        assert_eq!(frost.next, 160);
    }

    #[test]
    fn behaviour_timers_carry_their_overshoot_like_worldbox() {
        let mut timer = THAW_INTERVAL;
        let fired = (0..10)
            .filter(|_| due(&mut timer, 0.02, THAW_INTERVAL))
            .count();
        assert_eq!(fired, 10);

        let mut timer = FREEZE_INTERVAL;
        let fired = (0..50)
            .filter(|_| due(&mut timer, 0.02, FREEZE_INTERVAL))
            .count();
        assert_eq!(fired, 10);
    }
}
