//! WorldBox's tile fires: how they start, spread, burn the top layer away and leave scorched ground.

pub mod burn;

use bevy::prelude::*;
use econbox_age::{AgeAsset, Ages, Dice, Particles};
use econbox_clock::{Frames, WorldClock};
use econbox_tile::Overlay;
use econbox_tilemap::{Douse, Ignite, Painted, TileMap, TilePos};
use std::collections::{BTreeMap, BTreeSet};

pub const FIRE_SPREAD_TIME: f64 = 2.0;
pub const FIRE_STOP_TIME: f64 = 5.0;
pub const FIRE_TIME: f64 = 30.0;
pub const BURNED_STAGES: u8 = 15;
pub const TILE_HEALTH: i32 = 10;
pub const FIRE_INTERVAL: f32 = 1.0;
pub const BURNED_INTERVAL: f32 = 3.0;
pub const BURNED_RANDOM: f32 = 5.0;
pub const BURNING_TIME: f64 = 30.0;
pub const BURNING_HIT: f64 = 2.0;
pub const BURNING_HITS: u32 = 10;
pub const FLASH: u8 = 10;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Burning {
    pub since: f64,
    pub hits: u32,
    pub next: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Behaviour {
    pub timer: f32,
    pub interval: f32,
    pub random: f32,
}

impl Behaviour {
    pub const fn new(interval: f32, random: f32) -> Self {
        Self {
            timer: interval,
            interval,
            random,
        }
    }

    pub fn due(&mut self, elapsed: f32, dice: &mut Dice) -> bool {
        if self.timer > 0.0 {
            self.timer -= elapsed;
            if self.timer > 0.0 {
                return false;
            }
        }
        self.timer += self.interval + dice.float(0.0, self.random);
        true
    }
}

#[derive(Resource, Debug, Clone)]
pub struct Fires {
    pub width: u32,
    pub height: u32,
    pub time: f64,
    pub burning: BTreeMap<u32, f64>,
    pub burned: BTreeMap<u32, u8>,
    pub health: BTreeMap<u32, i32>,
    pub objects: BTreeMap<u32, Burning>,
    pub flashes: Vec<(u32, u8)>,
    pub changed: BTreeSet<u32>,
    pub edited: BTreeSet<u32>,
    pub fire: Behaviour,
    pub scorch: Behaviour,
    pub dice: Dice,
}

impl Default for Fires {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            time: 0.0,
            burning: BTreeMap::new(),
            burned: BTreeMap::new(),
            health: BTreeMap::new(),
            objects: BTreeMap::new(),
            flashes: Vec::new(),
            changed: BTreeSet::new(),
            edited: BTreeSet::new(),
            fire: Behaviour::new(FIRE_INTERVAL, 0.0),
            scorch: Behaviour::new(BURNED_INTERVAL, BURNED_RANDOM),
            dice: Dice::new(0x0f17e),
        }
    }
}

impl Fires {
    pub fn clear(&mut self) {
        let width = self.width;
        let height = self.height;
        *self = Self {
            width,
            height,
            ..Self::default()
        };
    }

    pub fn fit(&mut self, map: &TileMap) {
        if self.width != map.width || self.height != map.height {
            self.width = map.width;
            self.height = map.height;
            self.clear();
        }
    }

    pub fn index(&self, x: u32, y: u32) -> u32 {
        y * self.width + x
    }

    pub fn pos(&self, index: u32) -> TilePos {
        TilePos::new(index % self.width.max(1), index / self.width.max(1))
    }

    pub fn on_fire(&self, x: u32, y: u32) -> bool {
        self.burning.contains_key(&self.index(x, y))
    }

    pub fn burned_stage(&self, x: u32, y: u32) -> u8 {
        self.burned
            .get(&self.index(x, y))
            .copied()
            .unwrap_or_default()
    }

    pub fn can_grow(&self, x: u32, y: u32) -> bool {
        !self.on_fire(x, y) && self.burned_stage(x, y) == 0
    }

    pub fn has_fires(&self) -> bool {
        !self.burning.is_empty()
    }

    pub fn object_burning(&self, x: u32, y: u32) -> bool {
        self.objects.contains_key(&self.index(x, y))
    }

    pub fn elapsed(&self, index: u32) -> Option<f64> {
        self.burning.get(&index).map(|since| self.time - since)
    }

    fn neighbours(&self, index: u32) -> Vec<u32> {
        let TilePos { x, y } = self.pos(index);
        let mut found = Vec::with_capacity(4);
        if x > 0 {
            found.push(index - 1);
        }
        if x + 1 < self.width {
            found.push(index + 1);
        }
        if y + 1 < self.height {
            found.push(index + self.width);
        }
        if y > 0 {
            found.push(index - self.width);
        }
        found
    }

    pub fn flash(&mut self, index: u32, value: u8) {
        self.flashes.push((index, value));
    }

    pub fn start_fire(&mut self, map: &mut TileMap, x: u32, y: u32, force: bool) -> bool {
        if x >= map.width || y >= map.height {
            return false;
        }
        self.fit(map);
        let index = self.index(x, y);
        let tile = map.at(x, y);
        let kind = burn::of(tile);

        if !force && (!kind.burnable || self.burning.contains_key(&index)) {
            return false;
        }
        if kind.liquid {
            return false;
        }

        if tile.frozen {
            let mut thawed = tile;
            thawed.frozen = false;
            map.set(x, y, thawed);
            self.edited.insert(index);
            self.health.remove(&index);
        }

        let mut lit = false;
        if burn::object_burns(map.at(x, y)) {
            self.ignite_object(index);
            lit = true;
        }

        let kind = burn::of(map.at(x, y));
        if kind.burnable || lit || force {
            lit = true;
            self.burning.insert(index, self.time);
            self.changed.insert(index);
            if kind.burnable {
                let health = self.health.entry(index).or_insert(TILE_HEALTH);
                *health -= kind.burn_rate;
                let spent = *health <= 0;
                self.set_burned(map, index);
                self.flash(index, FLASH);
                if spent {
                    self.decrease(map, index);
                }
            }
        }
        lit
    }

    fn ignite_object(&mut self, index: u32) {
        let time = self.time;
        self.objects.entry(index).or_insert(Burning {
            since: time,
            hits: 0,
            next: time + BURNING_HIT,
        });
        self.changed.insert(index);
    }

    fn decrease(&mut self, map: &mut TileMap, index: u32) {
        let TilePos { x, y } = self.pos(index);
        let mut tile = map.at(x, y);
        if tile.wall.is_some() {
            tile.wall = None;
        } else if tile.overlay != Overlay::Bare {
            tile.overlay = Overlay::Bare;
        } else {
            return;
        }
        map.set(x, y, tile);
        self.health.remove(&index);
        self.edited.insert(index);
    }

    pub fn set_burned(&mut self, map: &TileMap, index: u32) {
        let TilePos { x, y } = self.pos(index);
        if !burn::of(map.at(x, y)).can_be_set_on_fire {
            return;
        }
        let stage = BURNED_STAGES - self.dice.range(0, 10) as u8;
        self.burned.insert(index, stage);
        self.changed.insert(index);
    }

    pub fn stop_fire(&mut self, map: &TileMap, index: u32) {
        if self.burning.remove(&index).is_some() {
            self.changed.insert(index);
            self.set_burned(map, index);
        }
    }

    pub fn remove_burn(&mut self, index: u32) {
        if self.burned.remove(&index).is_some() {
            self.changed.insert(index);
        }
    }

    pub fn update(&mut self, map: &mut TileMap, elapsed: f32, age: &AgeAsset) {
        self.fit(map);
        self.time += f64::from(elapsed);

        if self.fire.due(elapsed, &mut self.dice) {
            self.update_fire(map, age);
        }
        if self.scorch.due(elapsed, &mut self.dice) {
            self.update_burned();
        }
        self.update_objects(map);
    }

    pub fn update_fire(&mut self, map: &mut TileMap, age: &AgeAsset) {
        if self.burning.is_empty() {
            return;
        }
        let spread = !age.particles.contains(&Particles::Rain);
        let snapshot: Vec<(u32, f64)> = self.burning.iter().map(|(&i, &t)| (i, t)).collect();

        for (index, since) in snapshot {
            let elapsed = self.time - since;
            if elapsed >= FIRE_SPREAD_TIME && spread {
                for next in self.neighbours(index) {
                    let TilePos { x, y } = self.pos(next);
                    let odds = burn::of(map.at(x, y)).fire_chance * age.fire_spread_rate_bonus;
                    if self.dice.chance(odds) && self.start_fire(map, x, y, false) {
                        self.flash(next, FLASH);
                    }
                }
            }

            let odds = if elapsed > FIRE_STOP_TIME {
                (0.1 + elapsed / FIRE_TIME * 0.3) as f32
            } else {
                0.0
            };
            let TilePos { x, y } = self.pos(index);
            let stop =
                burn::of(map.at(x, y)).ocean || elapsed >= FIRE_TIME || self.dice.chance(odds);
            if stop {
                self.stop_fire(map, index);
            }
        }
    }

    pub fn update_burned(&mut self) {
        let burning = &self.burning;
        let mut gone = Vec::new();
        for (&index, stage) in self.burned.iter_mut() {
            if burning.contains_key(&index) {
                continue;
            }
            *stage = stage.saturating_sub(1);
            self.changed.insert(index);
            if *stage == 0 {
                gone.push(index);
            }
        }
        for index in gone {
            self.burned.remove(&index);
        }
    }

    pub fn update_objects(&mut self, map: &mut TileMap) {
        let time = self.time;
        let mut done = Vec::new();
        for (&index, burning) in self.objects.iter_mut() {
            while burning.hits < BURNING_HITS && time >= burning.next {
                burning.hits += 1;
                burning.next += BURNING_HIT;
            }
            if burning.hits >= BURNING_HITS {
                done.push((index, true));
            } else if time - burning.since >= BURNING_TIME {
                done.push((index, false));
            }
        }
        for (index, destroyed) in done {
            self.objects.remove(&index);
            self.changed.insert(index);
            if destroyed {
                let TilePos { x, y } = self.pos(index);
                let mut tile = map.at(x, y);
                if tile.flora.take().is_some() {
                    map.set(x, y, tile);
                    self.edited.insert(index);
                }
            }
        }
    }

    pub fn forget(&mut self, map: &TileMap, x: u32, y: u32) {
        if x >= self.width || y >= self.height {
            return;
        }
        let index = self.index(x, y);
        self.health.remove(&index);
        if map.at(x, y).flora.is_none() && self.objects.remove(&index).is_some() {
            self.changed.insert(index);
        }
    }

    pub fn take_edited(&mut self) -> Vec<TilePos> {
        let edited = std::mem::take(&mut self.edited);
        edited.into_iter().map(|index| self.pos(index)).collect()
    }
}

pub struct FirePlugin;

impl Plugin for FirePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Fires>()
            .init_resource::<Frames>()
            .add_message::<Ignite>()
            .add_message::<Douse>()
            .add_message::<Painted>()
            .add_systems(First, settle)
            .add_systems(Update, (forget, ignite, douse, advance).chain());
    }
}

fn settle(mut fires: ResMut<Fires>) {
    if !fires.changed.is_empty() || !fires.flashes.is_empty() {
        fires.changed.clear();
        fires.flashes.clear();
    }
}

fn forget(
    mut painted: MessageReader<Painted>,
    map: Option<Res<TileMap>>,
    mut fires: ResMut<Fires>,
) {
    let Some(map) = map else {
        painted.clear();
        return;
    };
    fires.fit(&map);
    for message in painted.read() {
        for pos in &message.cells {
            fires.forget(&map, pos.x, pos.y);
        }
    }
}

fn ignite(
    mut asked: MessageReader<Ignite>,
    map: Option<ResMut<TileMap>>,
    mut fires: ResMut<Fires>,
    mut painted: MessageWriter<Painted>,
) {
    let Some(mut map) = map else {
        asked.clear();
        return;
    };
    let requests: Vec<Ignite> = asked.read().copied().collect();
    if requests.is_empty() {
        return;
    }
    for request in requests {
        fires.start_fire(&mut map, request.pos.x, request.pos.y, true);
    }
    let edited = fires.take_edited();
    if !edited.is_empty() {
        painted.write(Painted::new(edited));
    }
}

fn douse(mut asked: MessageReader<Douse>, map: Option<Res<TileMap>>, mut fires: ResMut<Fires>) {
    let Some(map) = map else {
        asked.clear();
        return;
    };
    fires.fit(&map);
    for request in asked.read() {
        let TilePos { x, y } = request.pos;
        if x >= map.width || y >= map.height {
            continue;
        }
        let index = fires.index(x, y);
        fires.stop_fire(&map, index);
        fires.remove_burn(index);
    }
}

fn advance(
    frames: Res<Frames>,
    clock: Option<Res<WorldClock>>,
    ages: Option<Res<Ages>>,
    map: Option<ResMut<TileMap>>,
    mut fires: ResMut<Fires>,
    mut painted: MessageWriter<Painted>,
) {
    let Some(mut map) = map else {
        return;
    };
    let clock = clock.map(|clock| *clock).unwrap_or_default();
    if !clock.running() || frames.due == 0 {
        return;
    }
    if !fires.has_fires() && fires.burned.is_empty() && fires.objects.is_empty() {
        return;
    }
    let age = ages
        .map(|ages| ages.asset())
        .unwrap_or_else(|| econbox_age::Age::Hope.asset());
    for _ in 0..frames.due {
        fires.update(map.as_mut(), clock.step(), age);
    }
    let edited = fires.take_edited();
    if !edited.is_empty() {
        painted.write(Painted::new(edited));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use econbox_age::Age;
    use econbox_tile::{Flora, Ground, Tile, Wall};

    fn meadow(size: u32) -> TileMap {
        TileMap::filled(size, size, Tile::new(Ground::SoilLow, Overlay::Grass))
    }

    fn run(fires: &mut Fires, map: &mut TileMap, seconds: f32, age: Age) {
        let steps = (seconds / 0.02).round() as usize;
        for _ in 0..steps {
            fires.update(map, 0.02, age.asset());
        }
    }

    #[test]
    fn a_forced_fire_takes_on_bare_sand_but_a_plain_spark_does_not() {
        let mut map = TileMap::filled(4, 4, Tile::bare(Ground::Sand));
        let mut fires = Fires::default();

        assert!(!fires.start_fire(&mut map, 1, 1, false));
        assert!(fires.start_fire(&mut map, 1, 1, true));
        assert!(fires.on_fire(1, 1));
        assert!(fires.burned_stage(1, 1) == 0);
        assert!(!fires.can_grow(1, 1));
    }

    #[test]
    fn water_and_lava_never_burn() {
        let mut map = TileMap::filled(3, 3, Tile::bare(Ground::ShallowWaters));
        map.set(2, 2, Tile::bare(Ground::Lava3));
        let mut fires = Fires::default();

        assert!(!fires.start_fire(&mut map, 0, 0, true));
        assert!(!fires.start_fire(&mut map, 2, 2, true));
    }

    #[test]
    fn grass_loses_five_health_per_fire_and_scorches_six_to_fifteen_stages() {
        let mut map = meadow(3);
        let mut fires = Fires::default();

        assert!(fires.start_fire(&mut map, 1, 1, false));
        let stage = fires.burned_stage(1, 1);
        assert!((6..=15).contains(&stage), "{stage}");
        assert_eq!(fires.health[&fires.index(1, 1)], 5);
        assert_eq!(map.at(1, 1).overlay, Overlay::Grass);
        assert_eq!(fires.flashes, vec![(fires.index(1, 1), FLASH)]);

        assert!(fires.start_fire(&mut map, 1, 1, true));
        assert_eq!(map.at(1, 1).overlay, Overlay::Bare);
        assert_eq!(fires.take_edited(), vec![TilePos::new(1, 1)]);
    }

    #[test]
    fn a_fire_waits_two_seconds_before_it_spreads_through_grass() {
        let mut map = meadow(41);
        let mut fires = Fires::default();
        fires.start_fire(&mut map, 20, 20, true);

        run(&mut fires, &mut map, 1.5, Age::Hope);
        assert_eq!(fires.burning.len(), 1);

        run(&mut fires, &mut map, 60.0, Age::Hope);
        assert!(fires.burned.len() > 1, "{}", fires.burned.len());
    }

    #[test]
    fn every_fire_is_out_within_thirty_seconds() {
        let mut map = TileMap::filled(5, 5, Tile::bare(Ground::Sand));
        let mut fires = Fires::default();
        fires.start_fire(&mut map, 2, 2, true);

        run(&mut fires, &mut map, 31.5, Age::Hope);
        assert!(!fires.has_fires());
        assert_eq!(fires.burning.len(), 0);
        assert!(fires.burned_stage(2, 2) > 0);
    }

    #[test]
    fn the_age_of_tears_keeps_fires_from_spreading() {
        let mut map = meadow(9);
        let mut fires = Fires::default();
        fires.start_fire(&mut map, 4, 4, true);

        run(&mut fires, &mut map, 40.0, Age::Tears);
        assert_eq!(fires.burned.len(), 1);
    }

    #[test]
    fn scorch_marks_fade_a_stage_every_three_to_eight_seconds() {
        let mut map = TileMap::filled(3, 3, Tile::bare(Ground::Sand));
        let mut fires = Fires::default();
        let index = fires_index(&mut fires, &map, 1, 1);
        fires.burned.insert(index, 2);

        run(&mut fires, &mut map, 2.9, Age::Hope);
        assert_eq!(fires.burned_stage(1, 1), 2);
        run(&mut fires, &mut map, 16.0, Age::Hope);
        assert_eq!(fires.burned_stage(1, 1), 0);
        assert!(fires.can_grow(1, 1));
    }

    fn fires_index(fires: &mut Fires, map: &TileMap, x: u32, y: u32) -> u32 {
        fires.fit(map);
        fires.index(x, y)
    }

    #[test]
    fn a_tree_on_fire_burns_down_after_ten_hits_two_seconds_apart() {
        let mut map = TileMap::filled(3, 3, Tile::bare(Ground::Sand));
        map.set(
            1,
            1,
            Tile::new(Ground::SoilLow, Overlay::Grass).with_flora(Some(Flora::Tree)),
        );
        let mut fires = Fires::default();
        fires.start_fire(&mut map, 1, 1, true);
        assert!(fires.object_burning(1, 1));
        fires.take_edited();

        fires.time += 19.9;
        fires.update_objects(&mut map);
        assert_eq!(map.at(1, 1).flora, Some(Flora::Tree));

        fires.time += 0.2;
        fires.update_objects(&mut map);
        assert_eq!(map.at(1, 1).flora, None);
        assert!(!fires.object_burning(1, 1));
        assert_eq!(fires.take_edited(), vec![TilePos::new(1, 1)]);
    }

    #[test]
    fn only_a_forced_fire_reaches_a_tree_on_ground_that_does_not_burn() {
        let mut map = TileMap::filled(3, 1, Tile::bare(Ground::Sand));
        let mut wooded = Tile::bare(Ground::Sand);
        wooded.flora = Some(Flora::Tree);
        map.set(1, 0, wooded);
        let mut fires = Fires::default();

        assert!(!fires.start_fire(&mut map, 1, 0, false));
        assert!(fires.start_fire(&mut map, 1, 0, true));
        assert!(fires.object_burning(1, 0));
    }

    #[test]
    fn wild_walls_burn_away() {
        let mut map = TileMap::filled(2, 2, Tile::bare(Ground::SoilLow));
        map.set(
            0,
            0,
            Tile {
                wall: Some(Wall::Wild),
                ..Tile::bare(Ground::SoilLow)
            },
        );
        let mut fires = Fires::default();
        fires.start_fire(&mut map, 0, 0, false);
        fires.burning.clear();
        fires.start_fire(&mut map, 0, 0, false);

        assert_eq!(map.at(0, 0).wall, None);
        assert_eq!(fires.burned_stage(0, 0), 0);
    }

    #[test]
    fn behaviours_fire_on_their_interval_plus_a_random_delay() {
        let mut dice = Dice::new(3);
        let mut every = Behaviour::new(1.0, 0.0);
        let fired: usize = (0..520).filter(|_| every.due(0.02, &mut dice)).count();
        assert_eq!(fired, 10);
    }

    #[test]
    fn ignite_messages_start_forced_fires() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(FirePlugin)
            .insert_resource(TileMap::filled(4, 4, Tile::bare(Ground::Sand)));
        app.world_mut().write_message(Ignite {
            pos: TilePos::new(2, 3),
        });
        app.update();

        let fires = app.world().resource::<Fires>();
        assert!(fires.on_fire(2, 3));
        assert!(!fires.changed.is_empty());

        app.update();
        let fires = app.world().resource::<Fires>();
        assert!(fires.changed.is_empty() && fires.flashes.is_empty());
    }

    #[test]
    fn rain_puts_the_fire_out_and_washes_the_scorch_away() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(FirePlugin)
            .insert_resource(TileMap::filled(4, 4, Tile::bare(Ground::Sand)));
        let pos = TilePos::new(1, 1);
        app.world_mut().write_message(Ignite { pos });
        app.update();
        assert!(app.world().resource::<Fires>().on_fire(1, 1));

        app.world_mut().write_message(Douse { pos });
        app.update();
        let fires = app.world().resource::<Fires>();
        assert!(!fires.on_fire(1, 1));
        assert_eq!(fires.burned_stage(1, 1), 0);
    }
}
