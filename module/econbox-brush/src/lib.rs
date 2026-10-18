//! The painting tools: what the pointer puts on the map, and how to take it back.

mod blast;

pub use blast::{
    BOMBS, Bomb, BombDrop, BombRest, Charge, Detonate, Fuse, blasted, burst, drops, scatter,
};

use bevy::prelude::*;
use econbox_art::Icon;
use econbox_camera::Panning;
pub use econbox_camera::PointerOverUi;
use econbox_creature::{Effect, Spawn, Species, Strike};
use econbox_cursor::Hovered;
use econbox_tile::{Flora, Ground, Ore, Overlay, Tile, Wall};
use econbox_tilemap::{Painted, TileMap, TilePos};
use std::collections::{HashSet, VecDeque};

pub const LADDER: [Ground; 9] = [
    Ground::DeepOcean,
    Ground::CloseOcean,
    Ground::ShallowWaters,
    Ground::Sand,
    Ground::SoilLow,
    Ground::SoilHigh,
    Ground::Hills,
    Ground::Mountains,
    Ground::Summit,
];

pub const MIN_RADIUS: u32 = 0;
pub const MAX_RADIUS: u32 = 34;
pub const FLOOD_LIMIT: usize = 60_000;
pub const UNDO_LIMIT: usize = 64;
pub const SPAWN_GAP: f32 = 0.12;

const SEEDS: [Overlay; 5] = [
    Overlay::Grass,
    Overlay::Grass,
    Overlay::Jungle,
    Overlay::Enchanted,
    Overlay::Pumpkin,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Power {
    Sponge,
    Deluge,
    Frost,
    Seed,
    Bloom,
    Rain,
    Drought,
    Mound,
    Pit,
    Level,
    Smooth,
    Quake,
    Lightning,
    Fire,
    Acid,
    Bomb,
    Meteor,
    Volcano,
    Bless,
    Curse,
    Purge,
}

pub const POWERS: [Power; 21] = [
    Power::Sponge,
    Power::Deluge,
    Power::Frost,
    Power::Seed,
    Power::Bloom,
    Power::Rain,
    Power::Drought,
    Power::Mound,
    Power::Pit,
    Power::Level,
    Power::Smooth,
    Power::Quake,
    Power::Lightning,
    Power::Fire,
    Power::Acid,
    Power::Bomb,
    Power::Meteor,
    Power::Volcano,
    Power::Bless,
    Power::Curse,
    Power::Purge,
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Touch {
    pub rim: f32,
    pub noise: u32,
}

impl Power {
    pub fn name(self) -> &'static str {
        match self {
            Self::Sponge => "sponge",
            Self::Deluge => "deluge",
            Self::Frost => "frost",
            Self::Seed => "seed",
            Self::Bloom => "bloom",
            Self::Rain => "rain",
            Self::Drought => "drought",
            Self::Mound => "mound",
            Self::Pit => "pit",
            Self::Level => "level",
            Self::Smooth => "smooth",
            Self::Quake => "quake",
            Self::Lightning => "lightning",
            Self::Fire => "fire",
            Self::Acid => "acid",
            Self::Bomb => "bomb",
            Self::Meteor => "meteor",
            Self::Volcano => "volcano",
            Self::Bless => "bless",
            Self::Curse => "curse",
            Self::Purge => "purge",
        }
    }

    pub fn icon(self) -> Icon {
        match self {
            Self::Sponge => Icon::Sponge,
            Self::Deluge => Icon::Wave,
            Self::Frost => Icon::Snowflake,
            Self::Seed => Icon::Seed,
            Self::Bloom => Icon::Flower,
            Self::Rain => Icon::Cloud,
            Self::Drought => Icon::Sun,
            Self::Mound => Icon::Mound,
            Self::Pit => Icon::Pit,
            Self::Level => Icon::Level,
            Self::Smooth => Icon::Trowel,
            Self::Quake => Icon::Crack,
            Self::Lightning => Icon::Bolt,
            Self::Fire => Icon::Flame,
            Self::Acid => Icon::Flask,
            Self::Bomb => Icon::Bomb,
            Self::Meteor => Icon::Meteor,
            Self::Volcano => Icon::Volcano,
            Self::Bless => Icon::Heart,
            Self::Curse => Icon::Skull,
            Self::Purge => Icon::Halo,
        }
    }

    pub fn pulse(self) -> Option<f32> {
        Some(match self {
            Self::Lightning => 0.15,
            Self::Quake => 0.25,
            Self::Bomb => 0.3,
            Self::Curse => 0.4,
            Self::Meteor => 0.5,
            Self::Volcano => 0.7,
            _ => return None,
        })
    }

    pub fn reach(self) -> Option<u32> {
        Some(match self {
            Self::Lightning => 2,
            Self::Quake => 8,
            Self::Meteor => 5,
            Self::Volcano => 5,
            _ => return None,
        })
    }

    pub fn repeats(self) -> bool {
        !matches!(
            self,
            Self::Lightning | Self::Quake | Self::Meteor | Self::Volcano
        )
    }

    pub fn rate(self) -> Option<f32> {
        Some(match self {
            Self::Sponge | Self::Deluge | Self::Purge => 10.0,
            Self::Fire => 6.0,
            Self::Smooth => 5.0,
            Self::Seed | Self::Acid => 4.0,
            Self::Bloom | Self::Rain | Self::Drought => 3.0,
            _ => return None,
        })
    }

    pub fn blow(self, dt: f32) -> Option<Effect> {
        Some(match self {
            Self::Lightning | Self::Bomb | Self::Meteor | Self::Volcano => Effect::Kill,
            Self::Quake => Effect::Hurt(12.0),
            Self::Curse => Effect::Weaken,
            Self::Fire => Effect::Hurt(30.0 * dt),
            Self::Acid => Effect::Hurt(45.0 * dt),
            Self::Bless => Effect::Heal,
            Self::Purge => Effect::Purge,
            _ => return None,
        })
    }

    pub fn touched(self, tile: Tile, touch: Touch) -> Option<Tile> {
        let Tile {
            ground, overlay, ..
        } = tile;
        let soil = matches!(ground, Ground::SoilLow | Ground::SoilHigh);
        let green = !matches!(overlay, Overlay::Bare | Overlay::Infernal);
        let rung = rung(ground);

        let next = match self {
            Self::Sponge if ground.is_water() => Tile::bare(Ground::Sand),
            Self::Deluge if ground.is_lava() => Tile::bare(Ground::Hills),
            Self::Deluge if ground.takes_overlay() => Tile::bare(Ground::ShallowWaters),
            Self::Frost if ground.is_lava() => Tile::bare(Ground::Hills),
            Self::Seed if soil && overlay == Overlay::Bare => {
                Tile::new(ground, SEEDS[touch.noise as usize % SEEDS.len()])
            }
            Self::Bloom if soil && overlay == Overlay::Bare => Tile::new(ground, Overlay::Grass),
            Self::Bloom if soil && overlay == Overlay::Grass => Tile::new(ground, Overlay::Jungle),
            Self::Rain if overlay == Overlay::Infernal => Tile::bare(ground),
            Self::Rain if soil && overlay == Overlay::Bare => Tile::new(ground, Overlay::Grass),
            Self::Rain if ground == Ground::Sand => Tile::bare(Ground::SoilLow),
            Self::Drought if overlay == Overlay::Jungle => Tile::new(ground, Overlay::Grass),
            Self::Drought if green => Tile::bare(ground),
            Self::Drought if soil || ground == Ground::ShallowWaters => Tile::bare(Ground::Sand),
            Self::Mound => {
                let peak = cone(1.0 - touch.rim);
                tile.with_ground(LADDER[rung?.max(peak)])
            }
            Self::Pit => {
                let floor = cone(touch.rim);
                tile.with_ground(LADDER[rung?.min(floor)])
            }
            Self::Quake if touch.noise.is_multiple_of(3) && ground.is_land() => {
                Tile::bare(step(ground, -1))
            }
            Self::Lightning | Self::Fire if green => Tile::new(ground, Overlay::Infernal),
            Self::Acid if ground.is_land() => Tile::bare(ground),
            Self::Bomb if touch.rim < 0.4 && ground.is_land() => Tile::bare(ground),
            Self::Meteor if touch.rim < 0.35 && ground.is_land() => Tile::bare(step(ground, -1)),
            Self::Bomb | Self::Meteor if green => Tile::new(ground, Overlay::Infernal),
            Self::Volcano if touch.rim < 0.5 => Tile::bare(Ground::Lava3),
            Self::Volcano if touch.rim < 0.75 => Tile::bare(Ground::Lava2),
            Self::Volcano if ground.is_land() => Tile::bare(Ground::Sand),
            Self::Curse if ground.takes_overlay() => Tile::new(ground, Overlay::Infernal),
            Self::Purge if overlay == Overlay::Infernal => Tile::new(ground, Overlay::Grass),
            _ => return None,
        };

        (next != tile).then_some(next)
    }
}

fn rung(ground: Ground) -> Option<usize> {
    LADDER.iter().position(|&rung| rung == ground)
}

fn cone(height: f32) -> usize {
    (height.clamp(0.0, 1.0) * (LADDER.len() - 1) as f32).round() as usize
}

pub fn levelled(tile: Tile, ground: Ground) -> Option<Tile> {
    let next = tile.with_ground(ground);

    (next != tile).then_some(next)
}

pub fn smoothed(map: &TileMap, pos: TilePos) -> Option<Tile> {
    let tile = map.get(pos)?;
    rung(tile.ground)?;

    let mut rungs: Vec<usize> = (-1_i32..=1)
        .flat_map(|dy| (-1_i32..=1).map(move |dx| (dx, dy)))
        .filter_map(|(dx, dy)| {
            let x = u32::try_from(pos.x as i32 + dx).ok()?;
            let y = u32::try_from(pos.y as i32 + dy).ok()?;
            rung(map.get(TilePos::new(x, y))?.ground)
        })
        .collect();
    rungs.sort_unstable();

    levelled(tile, LADDER[rungs[rungs.len() / 2]])
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

fn luck(noise: u32) -> f32 {
    (mix(noise, 0x5bd1_e995, 7) >> 8) as f32 / (1 << 24) as f32
}

pub type Change = (TilePos, Tile, Tile);

pub fn unleash(
    map: &TileMap,
    brush: &Brush,
    power: Power,
    centre: TilePos,
    salt: u32,
    dt: f32,
) -> Vec<Change> {
    let targets = if power.pulse().is_some() {
        cells(map, centre, brush.shape, brush.radius)
    } else {
        brush.targets(map, centre)
    };
    let reach = brush.radius.max(1) as f32;
    let dose = power.rate().map(|rate| rate * dt);
    let level = map.get(centre).map(|tile| tile.ground);

    targets
        .into_iter()
        .filter_map(|pos| {
            let tile = map.get(pos)?;
            let noise = mix(pos.x, pos.y, salt);

            if dose.is_some_and(|dose| luck(noise) >= dose) {
                return None;
            }

            let dx = pos.x as f32 - centre.x as f32;
            let dy = pos.y as f32 - centre.y as f32;
            let touch = Touch {
                rim: (dx.hypot(dy) / reach).min(1.0),
                noise,
            };
            let next = match power {
                Power::Level => levelled(tile, level?),
                Power::Smooth => smoothed(map, pos),
                power => power.touched(tile, touch),
            }?;

            Some((pos, tile, next))
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Shape {
    #[default]
    Round,
    Square,
    Diamond,
    Scatter,
}

impl Shape {
    pub fn name(self) -> &'static str {
        match self {
            Self::Round => "round",
            Self::Square => "square",
            Self::Diamond => "diamond",
            Self::Scatter => "scatter",
        }
    }

    pub fn covers(self, dx: i32, dy: i32, reach: i32) -> bool {
        match self {
            Self::Round => round_covers(dx, dy, reach),
            Self::Square => dx.abs() <= reach && dy.abs() <= reach,
            Self::Diamond => dx.abs() + dy.abs() <= reach,
            Self::Scatter => {
                (dx == 0 && dy == 0)
                    || (dx * dx + dy * dy <= reach * reach
                        && mix(dx as u32, dy as u32, 0x51ab) % 100 < 38)
            }
        }
    }
}

fn round_covers(dx: i32, dy: i32, reach: i32) -> bool {
    let mut row = 0;
    let mut edge = reach;
    let mut error = 1 - reach;

    while row <= edge {
        if (dy.abs() == edge && dx.abs() <= row) || (dy.abs() == row && dx.abs() <= edge) {
            return true;
        }

        if error < 0 {
            error += 2 * row + 3;
        } else {
            error += 2 * (row - edge) + 5;
            edge -= 1;
        }

        row += 1;
    }

    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Circles,
    Squares,
    Diamonds,
    Specials,
}

impl Group {
    pub fn name(self) -> &'static str {
        match self {
            Self::Circles => "Circle",
            Self::Squares => "Square",
            Self::Diamonds => "Diamond",
            Self::Specials => "Special",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Preset {
    pub id: &'static str,
    pub group: Group,
    pub shape: Shape,
    pub radius: u32,
    pub flood: bool,
}

const fn preset(id: &'static str, group: Group, shape: Shape, radius: u32) -> Preset {
    Preset {
        id,
        group,
        shape,
        radius,
        flood: false,
    }
}

pub const GROUPS: [Group; 4] = [
    Group::Circles,
    Group::Squares,
    Group::Diamonds,
    Group::Specials,
];

pub const PRESETS: [Preset; 26] = [
    preset("circ_0", Group::Circles, Shape::Round, 0),
    preset("circ_1", Group::Circles, Shape::Round, 1),
    preset("circ_2", Group::Circles, Shape::Round, 2),
    preset("circ_3", Group::Circles, Shape::Round, 3),
    preset("circ_4", Group::Circles, Shape::Round, 4),
    preset("circ_5", Group::Circles, Shape::Round, 5),
    preset("circ_6", Group::Circles, Shape::Round, 6),
    preset("circ_7", Group::Circles, Shape::Round, 7),
    preset("circ_10", Group::Circles, Shape::Round, 10),
    preset("circ_15", Group::Circles, Shape::Round, 15),
    preset("sqr_1", Group::Squares, Shape::Square, 1),
    preset("sqr_2", Group::Squares, Shape::Square, 2),
    preset("sqr_4", Group::Squares, Shape::Square, 4),
    preset("sqr_10", Group::Squares, Shape::Square, 10),
    preset("sqr_15", Group::Squares, Shape::Square, 15),
    preset("diamond_1", Group::Diamonds, Shape::Diamond, 5),
    preset("diamond_2", Group::Diamonds, Shape::Diamond, 6),
    preset("diamond_4", Group::Diamonds, Shape::Diamond, 11),
    preset("diamond_5", Group::Diamonds, Shape::Diamond, 19),
    preset("diamond_7", Group::Diamonds, Shape::Diamond, 24),
    preset("scatter_1", Group::Specials, Shape::Scatter, 6),
    preset("scatter_2", Group::Specials, Shape::Scatter, 10),
    preset("scatter_3", Group::Specials, Shape::Scatter, 16),
    preset("scatter_4", Group::Specials, Shape::Scatter, 22),
    preset("scatter_5", Group::Specials, Shape::Scatter, 34),
    Preset {
        id: "fill",
        group: Group::Specials,
        shape: Shape::Round,
        radius: 0,
        flood: true,
    },
];

pub const DEFAULT_PRESET: usize = 5;

pub fn named(id: &str) -> Option<(Shape, u32)> {
    if let Some(preset) = PRESETS
        .iter()
        .find(|preset| preset.id == id && !preset.flood)
    {
        return Some((preset.shape, preset.radius));
    }

    let (shape, size) = id.split_once('_')?;
    let radius = size.parse().ok()?;

    match shape {
        "circ" => Some((Shape::Round, radius)),
        "sqr" => Some((Shape::Square, radius)),
        "diamond" => Some((Shape::Diamond, radius)),
        _ => None,
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ForcedBrush(pub Option<(Shape, u32)>);

impl ForcedBrush {
    pub fn over(&self, brush: Brush) -> Brush {
        match self.0 {
            Some((shape, radius)) => Brush {
                shape,
                radius,
                flood: false,
                ..brush
            },
            None => brush,
        }
    }
}

pub fn mask(shape: Shape, radius: u32) -> (u32, Vec<bool>) {
    let reach = radius.min(MAX_RADIUS) as i32;
    let side = (reach * 2 + 1) as u32;
    let mut out = Vec::with_capacity((side * side) as usize);

    for dy in -reach..=reach {
        for dx in -reach..=reach {
            out.push(shape.covers(dx, dy, reach));
        }
    }

    (side, out)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tool {
    #[default]
    Inspect,
    Ground(Ground),
    Overlay(Overlay),
    Raise,
    Lower,
    Strip,
    Spawn(Species),
    Power(Power),
    Plant(Flora),
    Fell,
    Ore(Ore),
    Mine,
    Wall(Wall),
    Drop(Bomb),
}

const TREE_SHARE: u32 = 45;
const BUSH_SHARE: u32 = 22;
const ORE_SHARE: u32 = 35;

impl Tool {
    pub fn edits(self) -> bool {
        !matches!(self, Self::Inspect | Self::Spawn(_) | Self::Drop(_))
    }

    pub fn holds(self) -> bool {
        !matches!(self, Self::Inspect)
    }

    pub fn name(self) -> String {
        match self {
            Self::Inspect => "inspect".to_string(),
            Self::Ground(ground) => ground.name().to_string(),
            Self::Overlay(overlay) => overlay.name().to_string(),
            Self::Raise => "raise".to_string(),
            Self::Lower => "lower".to_string(),
            Self::Strip => "strip".to_string(),
            Self::Spawn(species) => species.name().to_string(),
            Self::Power(power) => power.name().to_string(),
            Self::Plant(flora) => flora.name().to_string(),
            Self::Fell => "fell".to_string(),
            Self::Ore(ore) => ore.name().to_string(),
            Self::Mine => "mine".to_string(),
            Self::Wall(wall) => wall.name().to_string(),
            Self::Drop(bomb) => bomb.name().to_string(),
        }
    }

    pub fn share(self) -> Option<u32> {
        match self {
            Self::Plant(Flora::Bush) => Some(BUSH_SHARE),
            Self::Plant(_) => Some(TREE_SHARE),
            Self::Ore(_) => Some(ORE_SHARE),
            _ => None,
        }
    }

    pub fn lands(self, pos: TilePos) -> bool {
        self.share()
            .is_none_or(|share| mix(pos.x, pos.y, 0x5eed_7ee5) % 100 < share)
    }

    pub fn swatch(self) -> Option<[u8; 3]> {
        match self {
            Self::Ground(ground) => Some(ground.color()),
            Self::Overlay(overlay) => overlay.color(false),
            Self::Spawn(species) => Some(species.color()),
            Self::Plant(Flora::Bush) => Some([0xd8, 0x52, 0x3c]),
            Self::Plant(_) => Some([0x2c, 0x7c, 0x3a]),
            Self::Ore(ore) => Some(ore.color()),
            Self::Wall(wall) => Some(wall.color()),
            _ => None,
        }
    }

    pub fn icon(self) -> Option<(Icon, Option<[u8; 3]>)> {
        Some(match self {
            Self::Ground(ground) if ground.is_water() => (Icon::Droplet, Some(ground.color())),
            Self::Ground(ground) => (Icon::Block, Some(ground.color())),
            Self::Overlay(overlay) => (Icon::Sprout, overlay.color(false)),
            Self::Raise => (Icon::Raise, None),
            Self::Lower => (Icon::Lower, None),
            Self::Strip => (Icon::Eraser, None),
            Self::Inspect => (Icon::Lens, None),
            Self::Power(power) => (power.icon(), None),
            Self::Plant(_) => (Icon::Sprout, self.swatch()),
            Self::Fell => (Icon::Sprig, None),
            Self::Ore(_) => (Icon::Block, self.swatch()),
            Self::Mine => (Icon::Peaks, None),
            Self::Wall(_) => (Icon::Block, self.swatch()),
            Self::Drop(_) => (Icon::Bomb, None),
            Self::Spawn(_) => return None,
        })
    }

    pub fn applied(self, tile: Tile) -> Option<Tile> {
        let next = match self {
            Self::Inspect | Self::Spawn(_) | Self::Power(_) | Self::Drop(_) => return None,
            Self::Ground(ground) => tile.with_ground(ground),
            Self::Overlay(overlay) => tile.with_overlay(overlay),
            Self::Raise => tile.with_ground(step(tile.ground, 1)),
            Self::Lower => tile.with_ground(step(tile.ground, -1)),
            Self::Strip => Tile::bare(tile.ground).with_ore(tile.ore),
            Self::Plant(_) if tile.flora.is_some_and(|flora| flora != Flora::Sapling) => {
                return None;
            }
            Self::Plant(flora) => tile.with_flora(Some(flora)),
            Self::Fell => tile.with_flora(None),
            Self::Ore(ore) => tile.with_ore(Some(ore)),
            Self::Mine => tile.with_ore(None),
            Self::Wall(wall) => tile.with_wall(Some(wall)),
        };

        (next != tile).then_some(next)
    }
}

pub fn step(ground: Ground, by: i32) -> Ground {
    let Some(rung) = LADDER.iter().position(|&rung| rung == ground) else {
        return ground;
    };
    let next = (rung as i32 + by).clamp(0, LADDER.len() as i32 - 1);

    LADDER[next as usize]
}

pub fn cells(map: &TileMap, centre: TilePos, shape: Shape, radius: u32) -> Vec<TilePos> {
    let reach = radius.clamp(MIN_RADIUS, MAX_RADIUS) as i32;
    let mut out = Vec::new();

    for dy in -reach..=reach {
        for dx in -reach..=reach {
            if !shape.covers(dx, dy, reach) {
                continue;
            }

            let x = centre.x as i32 + dx;
            let y = centre.y as i32 + dy;

            if x < 0 || y < 0 || x as u32 >= map.width || y as u32 >= map.height {
                continue;
            }

            out.push(TilePos::new(x as u32, y as u32));
        }
    }

    out
}

pub fn flood(map: &TileMap, origin: TilePos) -> Vec<TilePos> {
    let Some(start) = map.get(origin) else {
        return Vec::new();
    };

    let mut seen = vec![false; map.cells()];
    let mut queue = VecDeque::from([origin]);
    let mut out = Vec::new();
    seen[map.index(origin.x, origin.y)] = true;

    while let Some(pos) = queue.pop_front() {
        out.push(pos);

        if out.len() >= FLOOD_LIMIT {
            break;
        }

        for (dx, dy) in [(1_i32, 0_i32), (-1, 0), (0, 1), (0, -1)] {
            let x = pos.x as i32 + dx;
            let y = pos.y as i32 + dy;

            if x < 0 || y < 0 || x as u32 >= map.width || y as u32 >= map.height {
                continue;
            }

            let (x, y) = (x as u32, y as u32);
            let index = map.index(x, y);

            if seen[index] || map.at(x, y).ground != start.ground {
                continue;
            }

            seen[index] = true;
            queue.push_back(TilePos::new(x, y));
        }
    }

    out
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Brush {
    pub tool: Tool,
    pub shape: Shape,
    pub radius: u32,
    pub flood: bool,
}

impl Default for Brush {
    fn default() -> Self {
        let preset = PRESETS[DEFAULT_PRESET];

        Self {
            tool: Tool::default(),
            shape: preset.shape,
            radius: preset.radius,
            flood: preset.flood,
        }
    }
}

impl Brush {
    pub fn preset(&self) -> Option<usize> {
        PRESETS.iter().position(|preset| {
            preset.flood == self.flood
                && (self.flood || (preset.shape == self.shape && preset.radius == self.radius))
        })
    }

    pub fn wear(&mut self, preset: Preset) {
        self.shape = preset.shape;
        self.radius = preset.radius;
        self.flood = preset.flood;
    }

    pub fn step(&mut self, by: i32) -> bool {
        let at = self.preset().unwrap_or(DEFAULT_PRESET) as i32;
        let next = (at + by).clamp(0, PRESETS.len() as i32 - 1) as usize;

        if next as i32 == at {
            return false;
        }

        self.wear(PRESETS[next]);
        true
    }

    pub fn grow(&mut self) {
        self.radius = (self.radius + 1).min(MAX_RADIUS);
    }

    pub fn shrink(&mut self) {
        self.radius = self.radius.saturating_sub(1);
    }

    pub fn label(&self) -> String {
        format!(
            "{} {}{}",
            self.shape.name(),
            self.radius,
            if self.flood { " flood" } else { "" }
        )
    }

    pub fn targets(&self, map: &TileMap, centre: TilePos) -> Vec<TilePos> {
        if self.flood {
            flood(map, centre)
        } else {
            cells(map, centre, self.shape, self.radius)
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Stroke {
    pub before: Vec<(TilePos, Tile)>,
}

#[derive(Debug, Default)]
struct Open {
    stroke: Stroke,
    seen: HashSet<(u32, u32)>,
}

#[derive(Resource, Debug, Default)]
pub struct UndoStack {
    done: Vec<Stroke>,
    open: Option<Open>,
}

impl UndoStack {
    pub fn begin(&mut self) {
        if self.open.is_none() {
            self.open = Some(Open::default());
        }
    }

    pub fn record(&mut self, pos: TilePos, tile: Tile) {
        let Some(open) = &mut self.open else { return };

        if open.seen.insert((pos.x, pos.y)) {
            open.stroke.before.push((pos, tile));
        }
    }

    pub fn commit(&mut self) {
        let Some(open) = self.open.take() else { return };

        if open.stroke.before.is_empty() {
            return;
        }

        self.done.push(open.stroke);

        if self.done.len() > UNDO_LIMIT {
            self.done.remove(0);
        }
    }

    pub fn take(&mut self) -> Option<Stroke> {
        self.done.pop()
    }

    pub fn depth(&self) -> usize {
        self.done.len()
    }
}

#[derive(Message, Debug, Clone, Copy)]
pub struct Undo;

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct SpawnRest(pub f32);

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct PowerRest(pub f32);

pub struct BrushPlugin;

impl Plugin for BrushPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Brush>()
            .init_resource::<UndoStack>()
            .init_resource::<PointerOverUi>()
            .init_resource::<SpawnRest>()
            .init_resource::<PowerRest>()
            .init_resource::<ForcedBrush>()
            .init_resource::<BombRest>()
            .add_message::<Painted>()
            .add_message::<Undo>()
            .add_message::<BombDrop>()
            .add_message::<Detonate>()
            .add_systems(
                Update,
                (
                    hold_pan,
                    paint,
                    drop_in,
                    blast::bombard,
                    blast::land,
                    blast::tick,
                    blast::explode,
                    rewind,
                )
                    .chain()
                    .run_if(resource_exists::<TileMap>),
            );
    }
}

fn hold_pan(brush: Res<Brush>, mut panning: ResMut<Panning>) {
    let with_left = !brush.tool.holds();

    if panning.with_left != with_left {
        panning.with_left = with_left;
    }
}

fn many(keys: Option<Res<ButtonInput<KeyCode>>>) -> bool {
    keys.is_some_and(|keys| keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]))
}

fn paint(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    hovered: Res<Hovered>,
    brush: Res<Brush>,
    forced: Option<Res<ForcedBrush>>,
    mut armed: Local<bool>,
    over_ui: Res<PointerOverUi>,
    time: Res<Time>,
    mut rest: ResMut<PowerRest>,
    mut salt: Local<u32>,
    mut map: ResMut<TileMap>,
    mut undo: ResMut<UndoStack>,
    mut painted: MessageWriter<Painted>,
    mut strike: MessageWriter<Strike>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        *armed = !over_ui.0;
    }

    if !buttons.pressed(MouseButton::Left) {
        undo.commit();
        return;
    }

    if over_ui.0 || !*armed || !brush.tool.edits() {
        return;
    }

    let brush = forced.map_or(*brush, |forced| forced.over(*brush));

    let Some(centre) = hovered.pos else { return };
    let dt = time.delta_secs();

    let changes: Vec<Change> = match brush.tool {
        Tool::Power(power) => {
            if let Some(gap) = power.pulse() {
                let fresh = buttons.just_pressed(MouseButton::Left);

                if fresh {
                    rest.0 = 0.0;
                }

                rest.0 = (rest.0 - dt).max(0.0);

                if rest.0 > 0.0 || !(power.repeats() || fresh || many(keys)) {
                    return;
                }

                rest.0 = gap;
            }

            *salt = salt.wrapping_add(1);

            if let Some(effect) = power.blow(dt) {
                strike.write(Strike {
                    centre,
                    reach: brush.radius as f32,
                    effect,
                });
            }

            unleash(&map, &brush, power, centre, *salt, dt)
        }
        tool => brush
            .targets(&map, centre)
            .into_iter()
            .filter(|pos| (brush.radius == 0 && !brush.flood) || tool.lands(*pos))
            .filter_map(|pos| {
                let tile = map.get(pos)?;
                Some((pos, tile, tool.applied(tile)?))
            })
            .collect(),
    };

    let mut changed = Vec::with_capacity(changes.len());

    undo.begin();

    for (pos, tile, next) in changes {
        undo.record(pos, tile);
        map.set(pos.x, pos.y, next);
        changed.push(pos);
    }

    if !changed.is_empty() {
        painted.write(Painted::new(changed));
    }
}

fn drop_in(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut armed: Local<bool>,
    hovered: Res<Hovered>,
    brush: Res<Brush>,
    over_ui: Res<PointerOverUi>,
    time: Res<Time>,
    mut rest: ResMut<SpawnRest>,
    mut spawn: MessageWriter<Spawn>,
) {
    let Tool::Spawn(species) = brush.tool else {
        return;
    };

    if buttons.just_pressed(MouseButton::Left) {
        rest.0 = 0.0;
        *armed = !over_ui.0;
    }

    rest.0 = (rest.0 - time.delta_secs()).max(0.0);

    if over_ui.0 || !*armed || !buttons.pressed(MouseButton::Left) || rest.0 > 0.0 {
        return;
    }

    if !buttons.just_pressed(MouseButton::Left) && !many(keys) {
        return;
    }

    let Some(pos) = hovered.pos else { return };

    spawn.write(Spawn { species, pos });
    rest.0 = SPAWN_GAP;
}

fn rewind(
    mut asked: MessageReader<Undo>,
    mut undo: ResMut<UndoStack>,
    mut map: ResMut<TileMap>,
    mut painted: MessageWriter<Painted>,
) {
    for _ in 0..asked.read().count() {
        let Some(stroke) = undo.take() else { break };
        let mut cells = Vec::with_capacity(stroke.before.len());

        for (pos, tile) in stroke.before.iter().rev() {
            map.set(pos.x, pos.y, *tile);
            cells.push(*pos);
        }

        painted.write(Painted::new(cells));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map() -> TileMap {
        TileMap::filled(32, 32, Tile::new(Ground::SoilLow, Overlay::Grass))
    }

    #[test]
    fn the_inspect_tool_leaves_the_map_alone() {
        let tile = Tile::new(Ground::SoilLow, Overlay::Grass);

        assert!(!Tool::Inspect.edits());
        assert!(!Tool::Inspect.holds());
        assert_eq!(Tool::Inspect.applied(tile), None);
    }

    #[test]
    fn painting_the_same_ground_twice_changes_nothing_the_second_time() {
        let tile = Tile::bare(Ground::Sand);

        assert_eq!(Tool::Ground(Ground::Sand).applied(tile), None);
        assert!(Tool::Ground(Ground::Mountains).applied(tile).is_some());
    }

    #[test]
    fn water_refuses_an_overlay_so_the_brush_reports_no_change() {
        let sea = Tile::bare(Ground::DeepOcean);

        assert_eq!(Tool::Overlay(Overlay::Grass).applied(sea), None);
    }

    #[test]
    fn raising_walks_up_the_elevation_ladder_and_stops_at_the_peak() {
        assert_eq!(step(Ground::DeepOcean, 1), Ground::CloseOcean);
        assert_eq!(step(Ground::Sand, 1), Ground::SoilLow);
        assert_eq!(step(Ground::Mountains, 1), Ground::Summit);
        assert_eq!(step(Ground::Summit, 1), Ground::Summit);
        assert_eq!(step(Ground::Summit, -1), Ground::Mountains);
    }

    #[test]
    fn lowering_walks_back_down_and_stops_at_the_deep() {
        assert_eq!(step(Ground::Mountains, -1), Ground::Hills);
        assert_eq!(step(Ground::CloseOcean, -1), Ground::DeepOcean);
        assert_eq!(step(Ground::DeepOcean, -1), Ground::DeepOcean);
    }

    #[test]
    fn lava_is_off_the_ladder_and_does_not_move() {
        assert_eq!(step(Ground::Lava2, 1), Ground::Lava2);
        assert_eq!(step(Ground::Lava3, -1), Ground::Lava3);
    }

    #[test]
    fn raising_keeps_the_overlay_where_the_new_ground_can_hold_it() {
        let grass = Tile::new(Ground::SoilLow, Overlay::Grass);
        let raised = Tool::Raise.applied(grass).expect("a change");

        assert_eq!(raised.ground, Ground::SoilHigh);
        assert_eq!(raised.overlay, Overlay::Grass);

        let rock = Tool::Raise
            .applied(Tile::new(Ground::SoilHigh, Overlay::Grass))
            .expect("a change");

        assert_eq!(rock.ground, Ground::Hills);
        assert_eq!(rock.overlay, Overlay::Bare);
    }

    #[test]
    fn stripping_takes_the_overlay_and_leaves_the_ground() {
        let jungle = Tile::new(Ground::SoilLow, Overlay::Jungle);
        let stripped = Tool::Strip.applied(jungle).expect("a change");

        assert_eq!(stripped.ground, Ground::SoilLow);
        assert_eq!(stripped.overlay, Overlay::Bare);
        assert_eq!(Tool::Strip.applied(stripped), None);
    }

    #[test]
    fn a_round_brush_covers_fewer_cells_than_a_square_one() {
        let map = map();
        let centre = TilePos::new(16, 16);
        let round = cells(&map, centre, Shape::Round, 4).len();
        let square = cells(&map, centre, Shape::Square, 4).len();

        assert_eq!(square, 9 * 9);
        assert!(round < square, "{round} vs {square}");
    }

    #[test]
    fn round_brushes_follow_the_filled_midpoint_circle() {
        let map = map();
        let centre = TilePos::new(16, 16);

        assert_eq!(cells(&map, centre, Shape::Round, 1).len(), 5);
        assert_eq!(cells(&map, centre, Shape::Round, 2).len(), 21);
        assert!(Shape::Round.covers(1, 1, 2));
        assert!(!Shape::Round.covers(2, 2, 2));
    }

    #[test]
    fn a_diamond_brush_stops_where_the_steps_add_up_to_its_reach() {
        assert!(Shape::Diamond.covers(3, 2, 5));
        assert!(!Shape::Diamond.covers(3, 3, 5));
        assert_eq!(
            cells(&map(), TilePos::new(16, 16), Shape::Diamond, 2).len(),
            13
        );
    }

    #[test]
    fn a_scatter_brush_is_sparse_and_always_holds_its_centre() {
        let (side, cells) = mask(Shape::Scatter, 10);
        let hit = cells.iter().filter(|hit| **hit).count();
        let (_, disc) = mask(Shape::Round, 10);
        let full = disc.iter().filter(|hit| **hit).count();

        assert!(cells[(side * side / 2) as usize]);
        assert!(hit > full / 5 && hit < full / 2, "{hit} of {full}");
        assert_eq!(mask(Shape::Scatter, 10).1, cells);
    }

    #[test]
    fn every_preset_has_its_own_id_and_the_default_is_a_five_wide_circle() {
        let mut ids: Vec<&str> = PRESETS.iter().map(|preset| preset.id).collect();
        ids.sort_unstable();
        ids.dedup();

        assert_eq!(ids.len(), PRESETS.len());
        assert_eq!(PRESETS[DEFAULT_PRESET].id, "circ_5");
        assert_eq!(Brush::default().preset(), Some(DEFAULT_PRESET));
        assert!(PRESETS.iter().all(|preset| preset.radius <= MAX_RADIUS));
    }

    #[test]
    fn stepping_through_presets_stops_at_both_ends() {
        let mut brush = Brush::default();

        while brush.step(1) {}
        assert_eq!(brush.preset(), Some(PRESETS.len() - 1));
        assert!(brush.flood);

        while brush.step(-1) {}
        assert_eq!(brush.preset(), Some(0));
        assert_eq!(brush.radius, 0);
        assert!(!brush.flood);
    }

    #[test]
    fn wearing_a_preset_sets_shape_radius_and_fill() {
        let mut brush = Brush::default();
        brush.wear(PRESETS[17]);

        assert_eq!(brush.shape, Shape::Diamond);
        assert_eq!(brush.radius, 11);
        assert_eq!(brush.preset(), Some(17));
    }

    #[test]
    fn a_brush_of_no_radius_is_a_single_cell() {
        let map = map();

        assert_eq!(cells(&map, TilePos::new(0, 0), Shape::Round, 0).len(), 1);
    }

    #[test]
    fn a_brush_at_the_edge_is_clipped_to_the_map() {
        let map = map();
        let corner = cells(&map, TilePos::new(0, 0), Shape::Square, 3);

        assert_eq!(corner.len(), 4 * 4);
        assert!(corner.iter().all(|pos| pos.x < 32 && pos.y < 32));
    }

    #[test]
    fn flooding_a_uniform_map_reaches_every_cell() {
        let map = map();

        assert_eq!(flood(&map, TilePos::new(5, 5)).len(), map.cells());
    }

    #[test]
    fn flooding_stops_at_a_different_ground() {
        let mut map = map();

        for y in 0..32 {
            map.set(16, y, Tile::bare(Ground::Mountains));
        }

        let left = flood(&map, TilePos::new(0, 0));

        assert_eq!(left.len(), 16 * 32);
        assert!(left.iter().all(|pos| pos.x < 16));
    }

    #[test]
    fn flooding_outside_the_map_finds_nothing() {
        let map = map();

        assert!(flood(&map, TilePos::new(99, 99)).is_empty());
    }

    #[test]
    fn the_brush_radius_stays_inside_its_range() {
        let mut brush = Brush::default();

        for _ in 0..100 {
            brush.grow();
        }
        assert_eq!(brush.radius, MAX_RADIUS);

        for _ in 0..100 {
            brush.shrink();
        }
        assert_eq!(brush.radius, MIN_RADIUS);
    }

    #[test]
    fn an_undo_stroke_remembers_each_cell_once() {
        let mut undo = UndoStack::default();
        let tile = Tile::bare(Ground::Sand);
        undo.begin();

        for _ in 0..5 {
            undo.record(TilePos::new(1, 1), tile);
        }
        undo.record(TilePos::new(2, 1), tile);
        undo.commit();

        let stroke = undo.take().expect("a stroke");

        assert_eq!(stroke.before.len(), 2);
    }

    #[test]
    fn an_empty_stroke_is_not_worth_undoing() {
        let mut undo = UndoStack::default();
        undo.begin();
        undo.commit();

        assert_eq!(undo.depth(), 0);
        assert!(undo.take().is_none());
    }

    #[test]
    fn the_undo_stack_forgets_the_oldest_stroke_once_it_is_full() {
        let mut undo = UndoStack::default();

        for step in 0..UNDO_LIMIT + 10 {
            undo.begin();
            undo.record(TilePos::new(step as u32, 0), Tile::default());
            undo.commit();
        }

        assert_eq!(undo.depth(), UNDO_LIMIT);
    }

    #[test]
    fn a_stroke_undone_puts_every_tile_back() {
        let mut map = map();
        let brush = Brush {
            tool: Tool::Ground(Ground::Mountains),
            shape: Shape::Square,
            radius: 2,
            flood: false,
        };
        let centre = TilePos::new(10, 10);
        let before = map.clone();
        let mut undo = UndoStack::default();
        undo.begin();

        for pos in brush.targets(&map, centre) {
            let tile = map.get(pos).expect("inside");
            if let Some(next) = brush.tool.applied(tile) {
                undo.record(pos, tile);
                map.set(pos.x, pos.y, next);
            }
        }
        undo.commit();

        assert_ne!(map, before);

        let stroke = undo.take().expect("a stroke");
        for (pos, tile) in stroke.before.iter().rev() {
            map.set(pos.x, pos.y, *tile);
        }

        assert_eq!(map, before);
    }

    #[test]
    fn a_flood_brush_ignores_the_radius() {
        let map = map();
        let brush = Brush {
            tool: Tool::Ground(Ground::Sand),
            shape: Shape::Round,
            radius: 1,
            flood: true,
        };

        assert_eq!(brush.targets(&map, TilePos::new(0, 0)).len(), map.cells());
    }

    #[test]
    fn only_the_inspect_tool_gives_the_left_button_back_to_the_camera() {
        assert!(Tool::Ground(Ground::Sand).holds());
        assert!(Tool::Raise.holds());
        assert!(Tool::Spawn(Species::Human).holds());
        assert!(!Tool::Inspect.holds());
    }

    #[test]
    fn a_spawn_tool_places_a_creature_instead_of_editing_the_ground() {
        let tool = Tool::Spawn(Species::Human);

        assert!(!tool.edits());
        assert_eq!(tool.applied(Tile::bare(Ground::Sand)), None);
        assert_eq!(tool.name(), "human");
        assert_eq!(tool.swatch(), Some(Species::Human.color()));
    }

    #[test]
    fn every_tool_but_a_spawn_has_an_icon_of_its_own() {
        for ground in econbox_tile::GROUNDS {
            assert!(Tool::Ground(ground).icon().is_some(), "{}", ground.name());
        }

        assert_eq!(Tool::Raise.icon().map(|found| found.0), Some(Icon::Raise));
        assert_eq!(Tool::Lower.icon().map(|found| found.0), Some(Icon::Lower));
        assert_eq!(Tool::Inspect.icon().map(|found| found.0), Some(Icon::Lens));
        assert_eq!(Tool::Strip.icon().map(|found| found.0), Some(Icon::Eraser));
        assert!(Tool::Spawn(Species::Cat).icon().is_none());
    }

    #[test]
    fn water_gets_a_droplet_and_solid_ground_gets_a_slab() {
        assert_eq!(
            Tool::Ground(Ground::DeepOcean).icon(),
            Some((Icon::Droplet, Some(Ground::DeepOcean.color())))
        );
        assert_eq!(
            Tool::Ground(Ground::Mountains).icon(),
            Some((Icon::Block, Some(Ground::Mountains.color())))
        );
    }

    fn touch(rim: f32) -> Touch {
        Touch { rim, noise: 0 }
    }

    fn brush(power: Power, radius: u32) -> Brush {
        Brush {
            tool: Tool::Power(power),
            shape: Shape::Round,
            radius,
            flood: false,
        }
    }

    #[test]
    fn every_power_has_a_name_and_an_icon_of_its_own() {
        let mut names: Vec<&str> = POWERS.iter().map(|power| power.name()).collect();
        let mut icons: Vec<String> = POWERS
            .iter()
            .map(|power| format!("{:?}", power.icon()))
            .collect();
        names.sort_unstable();
        names.dedup();
        icons.sort_unstable();
        icons.dedup();

        assert_eq!(names.len(), POWERS.len());
        assert_eq!(icons.len(), POWERS.len());
        assert!(POWERS.iter().all(|power| Tool::Power(*power).edits()));
        assert!(
            POWERS
                .iter()
                .all(|power| Tool::Power(*power).icon().is_some())
        );
    }

    #[test]
    fn a_sponge_drinks_the_sea_and_a_deluge_drowns_the_lowlands() {
        let sea = Tile::bare(Ground::DeepOcean);
        let meadow = Tile::new(Ground::SoilLow, Overlay::Grass);

        assert_eq!(
            Power::Sponge.touched(sea, touch(0.0)),
            Some(Tile::bare(Ground::Sand))
        );
        assert_eq!(Power::Sponge.touched(meadow, touch(0.0)), None);
        assert_eq!(
            Power::Deluge.touched(meadow, touch(0.0)),
            Some(Tile::bare(Ground::ShallowWaters))
        );
        assert_eq!(
            Power::Deluge.touched(Tile::bare(Ground::Mountains), touch(0.0)),
            None
        );
    }

    #[test]
    fn frost_and_a_deluge_both_turn_lava_to_stone() {
        let lava = Tile::bare(Ground::Lava3);

        assert_eq!(
            Power::Frost.touched(lava, touch(0.0)),
            Some(Tile::bare(Ground::Hills))
        );
        assert_eq!(
            Power::Deluge.touched(lava, touch(0.0)),
            Some(Tile::bare(Ground::Hills))
        );
        assert_eq!(
            Power::Frost.touched(Tile::bare(Ground::Sand), touch(0.0)),
            None
        );
    }

    #[test]
    fn seeds_only_take_on_bare_soil() {
        let soil = Tile::bare(Ground::SoilLow);
        let planted = Power::Seed.touched(soil, touch(0.0)).expect("a sprout");

        assert_ne!(planted.overlay, Overlay::Bare);
        assert_eq!(
            Power::Seed.touched(Tile::bare(Ground::Sand), touch(0.0)),
            None
        );
        assert_eq!(Power::Seed.touched(planted, touch(0.0)), None);
    }

    #[test]
    fn different_seeds_grow_different_plants() {
        let soil = Tile::bare(Ground::SoilHigh);
        let mut grown: Vec<Overlay> = (0..20)
            .filter_map(|noise| Power::Seed.touched(soil, Touch { rim: 0.0, noise }))
            .map(|tile| tile.overlay)
            .collect();
        grown.sort_unstable();
        grown.dedup();

        assert!(grown.len() >= 3, "{grown:?}");
    }

    #[test]
    fn bloom_turns_bare_soil_to_grass_and_grass_to_jungle() {
        let soil = Tile::bare(Ground::SoilLow);
        let grass = Power::Bloom.touched(soil, touch(0.0)).expect("grass");
        let jungle = Power::Bloom.touched(grass, touch(0.0)).expect("jungle");

        assert_eq!(grass.overlay, Overlay::Grass);
        assert_eq!(jungle.overlay, Overlay::Jungle);
        assert_eq!(Power::Bloom.touched(jungle, touch(0.0)), None);
    }

    #[test]
    fn rain_greens_the_desert_and_drought_takes_it_back() {
        let sand = Tile::bare(Ground::Sand);
        let soil = Power::Rain.touched(sand, touch(0.0)).expect("soil");
        let grass = Power::Rain.touched(soil, touch(0.0)).expect("grass");

        assert_eq!(soil, Tile::bare(Ground::SoilLow));
        assert_eq!(grass.overlay, Overlay::Grass);

        let bare = Power::Drought.touched(grass, touch(0.0)).expect("bare");
        let dust = Power::Drought.touched(bare, touch(0.0)).expect("sand");

        assert_eq!(bare, Tile::bare(Ground::SoilLow));
        assert_eq!(dust, Tile::bare(Ground::Sand));
        assert_eq!(
            Power::Drought.touched(Tile::bare(Ground::ShallowWaters), touch(0.0)),
            Some(Tile::bare(Ground::Sand))
        );
    }

    #[test]
    fn rain_washes_the_ash_off_scorched_ground() {
        let scorched = Tile::new(Ground::SoilLow, Overlay::Infernal);

        assert_eq!(
            Power::Rain.touched(scorched, touch(0.0)),
            Some(Tile::bare(Ground::SoilLow))
        );
    }

    #[test]
    fn a_mound_peaks_at_the_centre_and_fades_towards_the_rim() {
        let plain = Tile::bare(Ground::Sand);
        let top = Power::Mound.touched(plain, touch(0.0)).expect("a peak");
        let shoulder = Power::Mound.touched(plain, touch(0.5)).expect("a shoulder");

        assert_eq!(top.ground, Ground::Summit);
        assert!(shoulder.ground.elevation() < top.ground.elevation());
        assert_eq!(Power::Mound.touched(plain, touch(1.0)), None);
        assert_eq!(Power::Mound.touched(top, touch(0.0)), None);
    }

    #[test]
    fn a_pit_bottoms_out_at_the_centre_and_never_raises_anything() {
        let hill = Tile::bare(Ground::Hills);

        assert_eq!(
            Power::Pit.touched(hill, touch(0.0)).map(|tile| tile.ground),
            Some(Ground::DeepOcean)
        );
        assert_eq!(
            Power::Pit.touched(Tile::bare(Ground::DeepOcean), touch(0.0)),
            None
        );
        assert_eq!(
            Power::Pit.touched(Tile::bare(Ground::Sand), touch(1.0)),
            None
        );
    }

    #[test]
    fn lightning_and_fire_scorch_plants_but_leave_bare_ground() {
        let forest = Tile::new(Ground::SoilHigh, Overlay::Jungle);
        let rock = Tile::bare(Ground::Hills);

        for power in [Power::Lightning, Power::Fire] {
            assert_eq!(
                power.touched(forest, touch(0.0)),
                Some(Tile::new(Ground::SoilHigh, Overlay::Infernal))
            );
            assert_eq!(power.touched(rock, touch(0.0)), None);
        }
    }

    #[test]
    fn acid_eats_the_plants_off_the_land() {
        let grass = Tile::new(Ground::SoilLow, Overlay::Grass);

        assert_eq!(
            Power::Acid.touched(grass, touch(0.0)),
            Some(Tile::bare(Ground::SoilLow))
        );
        assert_eq!(
            Power::Acid.touched(Tile::bare(Ground::DeepOcean), touch(0.0)),
            None
        );
    }

    #[test]
    fn a_meteor_digs_a_crater_and_scorches_what_is_around_it() {
        let grass = Tile::new(Ground::SoilHigh, Overlay::Grass);

        assert_eq!(
            Power::Meteor.touched(grass, touch(0.0)),
            Some(Tile::bare(Ground::SoilLow))
        );
        assert_eq!(
            Power::Meteor.touched(grass, touch(0.8)),
            Some(Tile::new(Ground::SoilHigh, Overlay::Infernal))
        );
    }

    #[test]
    fn a_bomb_blasts_the_centre_bare_and_scorches_the_ring() {
        let grass = Tile::new(Ground::SoilLow, Overlay::Grass);

        assert_eq!(
            Power::Bomb.touched(grass, touch(0.1)),
            Some(Tile::bare(Ground::SoilLow))
        );
        assert_eq!(
            Power::Bomb.touched(grass, touch(0.9)),
            Some(Tile::new(Ground::SoilLow, Overlay::Infernal))
        );
    }

    #[test]
    fn a_volcano_pours_lava_at_its_heart_and_ash_sand_at_its_feet() {
        let grass = Tile::new(Ground::SoilLow, Overlay::Grass);

        assert_eq!(
            Power::Volcano.touched(grass, touch(0.0)),
            Some(Tile::bare(Ground::Lava3))
        );
        assert_eq!(
            Power::Volcano.touched(grass, touch(0.6)),
            Some(Tile::bare(Ground::Lava2))
        );
        assert_eq!(
            Power::Volcano.touched(grass, touch(0.9)),
            Some(Tile::bare(Ground::Sand))
        );
    }

    #[test]
    fn a_quake_only_ever_cracks_the_land_downwards() {
        let soil = Tile::new(Ground::SoilHigh, Overlay::Grass);
        let cracked: Vec<Tile> = (0..30)
            .filter_map(|noise| Power::Quake.touched(soil, Touch { rim: 0.0, noise }))
            .collect();

        assert!(!cracked.is_empty());
        assert!(cracked.len() < 30);
        assert!(
            cracked
                .iter()
                .all(|tile| *tile == Tile::bare(Ground::SoilLow))
        );
    }

    #[test]
    fn a_curse_blights_the_soil_and_a_purge_cleans_it() {
        let soil = Tile::bare(Ground::SoilLow);
        let cursed = Power::Curse.touched(soil, touch(0.0)).expect("blight");

        assert_eq!(cursed.overlay, Overlay::Infernal);
        assert_eq!(
            Power::Purge
                .touched(cursed, touch(0.0))
                .map(|tile| tile.overlay),
            Some(Overlay::Grass)
        );
        assert_eq!(Power::Bless.touched(soil, touch(0.0)), None);
    }

    #[test]
    fn creature_powers_carry_the_blow_their_name_promises() {
        assert_eq!(Power::Lightning.blow(0.1), Some(Effect::Kill));
        assert_eq!(Power::Bless.blow(0.1), Some(Effect::Heal));
        assert_eq!(Power::Curse.blow(0.1), Some(Effect::Weaken));
        assert_eq!(Power::Purge.blow(0.1), Some(Effect::Purge));
        assert_eq!(Power::Fire.blow(0.5), Some(Effect::Hurt(15.0)));
        assert_eq!(Power::Sponge.blow(0.1), None);
    }

    #[test]
    fn a_forced_brush_overrides_the_shape_and_size_but_not_the_tool() {
        let brush = Brush {
            tool: Tool::Power(Power::Volcano),
            shape: Shape::Square,
            radius: 15,
            flood: true,
        };
        let worn = ForcedBrush(Some((Shape::Round, 5))).over(brush);

        assert_eq!(worn.tool, brush.tool);
        assert_eq!(
            (worn.shape, worn.radius, worn.flood),
            (Shape::Round, 5, false)
        );
        assert_eq!(ForcedBrush(None).over(brush), brush);
    }

    #[test]
    fn brushes_are_found_by_their_ids() {
        assert_eq!(named("circ_10"), Some((Shape::Round, 10)));
        assert_eq!(named("sqr_0"), Some((Shape::Square, 0)));
        assert_eq!(named("diamond_4"), Some((Shape::Diamond, 11)));
        assert_eq!(named("fill"), None);
        assert_eq!(named("blob_3"), None);
    }

    #[test]
    fn strikes_wait_for_a_fresh_click_while_washes_keep_flowing() {
        assert!(!Power::Lightning.repeats());
        assert!(!Power::Volcano.repeats());
        assert!(Power::Bomb.repeats());
        assert!(Power::Rain.repeats());
        assert!(
            POWERS
                .iter()
                .filter(|power| !power.repeats())
                .all(|power| power.pulse().is_some())
        );
    }

    #[test]
    fn a_power_either_pulses_or_flows_but_never_both() {
        for power in POWERS {
            assert!(
                !(power.pulse().is_some() && power.rate().is_some()),
                "{}",
                power.name()
            );
        }

        assert!(Power::Lightning.pulse().is_some());
        assert!(Power::Meteor.pulse().is_some());
        assert!(Power::Rain.pulse().is_none());
    }

    #[test]
    fn a_gradual_power_does_nothing_in_a_frame_that_took_no_time() {
        let map = TileMap::filled(16, 16, Tile::bare(Ground::Sand));

        assert!(
            unleash(
                &map,
                &brush(Power::Rain, 3),
                Power::Rain,
                TilePos::new(8, 8),
                1,
                0.0
            )
            .is_empty()
        );
        assert!(
            !unleash(
                &map,
                &brush(Power::Rain, 3),
                Power::Rain,
                TilePos::new(8, 8),
                1,
                1.0
            )
            .is_empty()
        );
    }

    #[test]
    fn levelling_flattens_the_brush_to_the_ground_under_the_pointer() {
        let mut map = TileMap::filled(16, 16, Tile::bare(Ground::Mountains));
        map.set(8, 8, Tile::bare(Ground::Sand));
        let changes = unleash(
            &map,
            &brush(Power::Level, 2),
            Power::Level,
            TilePos::new(8, 8),
            1,
            0.016,
        );

        assert!(!changes.is_empty());
        assert!(
            changes
                .iter()
                .all(|(_, _, next)| next.ground == Ground::Sand)
        );
    }

    #[test]
    fn smoothing_wears_a_lone_spike_down_to_its_neighbours() {
        let mut map = TileMap::filled(16, 16, Tile::bare(Ground::SoilLow));
        map.set(8, 8, Tile::bare(Ground::Mountains));

        assert_eq!(
            smoothed(&map, TilePos::new(8, 8)),
            Some(Tile::bare(Ground::SoilLow))
        );
        assert_eq!(smoothed(&map, TilePos::new(3, 3)), None);
    }

    #[test]
    fn a_blast_ignores_the_flood_fill_and_keeps_to_its_radius() {
        let map = TileMap::filled(32, 32, Tile::new(Ground::SoilLow, Overlay::Grass));
        let mut wide = brush(Power::Bomb, 2);
        wide.flood = true;
        let hit = unleash(&map, &wide, Power::Bomb, TilePos::new(16, 16), 1, 0.016);

        assert_eq!(
            hit.len(),
            cells(&map, TilePos::new(16, 16), Shape::Round, 2).len()
        );
    }

    #[test]
    fn a_palette_tool_carries_the_colour_it_paints() {
        assert_eq!(
            Tool::Ground(Ground::Sand).swatch(),
            Some(Ground::Sand.color())
        );
        assert_eq!(Tool::Overlay(Overlay::Bare).swatch(), None);
        assert_eq!(Tool::Raise.swatch(), None);
        assert_eq!(
            Tool::Spawn(Species::Fox).swatch(),
            Some(Species::Fox.color())
        );
    }

    fn forest() -> Tile {
        Tile::new(Ground::SoilLow, Overlay::Grass).with_flora(Some(Flora::Tree))
    }

    #[test]
    fn the_axe_fells_trees_and_leaves_the_grass() {
        let felled = Tool::Fell.applied(forest()).expect("a stump");

        assert_eq!(felled, Tile::new(Ground::SoilLow, Overlay::Grass));
        assert_eq!(Tool::Fell.applied(felled), None);
    }

    #[test]
    fn the_sickle_clears_plants_and_trees_but_not_the_ore() {
        let vein = Tile::bare(Ground::Hills).with_ore(Some(Ore::Gold));

        assert_eq!(
            Tool::Strip.applied(forest()),
            Some(Tile::bare(Ground::SoilLow))
        );
        assert_eq!(Tool::Strip.applied(vein), None);
    }

    #[test]
    fn the_pickaxe_breaks_deposits_and_nothing_else() {
        let vein = Tile::bare(Ground::Mountains).with_ore(Some(Ore::Adamantine));

        assert_eq!(
            Tool::Mine.applied(vein),
            Some(Tile::bare(Ground::Mountains))
        );
        assert_eq!(Tool::Mine.applied(forest()), None);
    }

    #[test]
    fn deposits_are_buried_only_in_rock() {
        let hill = Tile::bare(Ground::Hills);

        assert_eq!(
            Tool::Ore(Ore::Silver)
                .applied(hill)
                .and_then(|tile| tile.ore),
            Some(Ore::Silver)
        );
        assert_eq!(
            Tool::Ore(Ore::Silver).applied(Tile::bare(Ground::Sand)),
            None
        );
        assert_eq!(
            Tool::Ore(Ore::Gold)
                .applied(hill.with_ore(Some(Ore::Stone)))
                .and_then(|tile| tile.ore),
            Some(Ore::Gold)
        );
    }

    #[test]
    fn tree_food_raises_saplings_and_plants_only_on_fertile_ground() {
        let meadow = Tile::new(Ground::SoilHigh, Overlay::Clover);
        let sapling = meadow.with_flora(Some(Flora::Sapling));
        let bush = meadow.with_flora(Some(Flora::Bush));

        assert_eq!(
            Tool::Plant(Flora::Tree)
                .applied(meadow)
                .and_then(|tile| tile.flora),
            Some(Flora::Tree)
        );
        assert_eq!(
            Tool::Plant(Flora::Tree)
                .applied(sapling)
                .and_then(|tile| tile.flora),
            Some(Flora::Tree)
        );
        assert_eq!(Tool::Plant(Flora::Tree).applied(bush), None);
        assert_eq!(
            Tool::Plant(Flora::Bush).applied(Tile::new(Ground::SoilLow, Overlay::Void)),
            None
        );
    }

    #[test]
    fn planting_and_burying_scatter_over_the_brush_instead_of_carpeting_it() {
        let landed = |tool: Tool| {
            (0..64)
                .flat_map(|y| (0..64).map(move |x| TilePos::new(x, y)))
                .filter(|pos| tool.lands(*pos))
                .count()
        };
        let all = 64 * 64;

        assert_eq!(landed(Tool::Ground(Ground::Sand)), all);
        assert!((all * 35 / 100..all * 55 / 100).contains(&landed(Tool::Plant(Flora::Tree))));
        assert!((all * 15 / 100..all * 30 / 100).contains(&landed(Tool::Plant(Flora::Bush))));
        assert!((all * 25 / 100..all * 45 / 100).contains(&landed(Tool::Ore(Ore::Gold))));
    }

    #[test]
    fn reshaping_the_land_keeps_what_grows_on_it() {
        let raised = Tool::Raise.applied(forest()).expect("a climb");

        assert_eq!(raised.ground, Ground::SoilHigh);
        assert_eq!(raised.flora, Some(Flora::Tree));
        assert_eq!(Power::Mound.touched(forest(), touch(1.0)), None);
        assert_eq!(levelled(forest(), Ground::SoilLow), None);
        assert_eq!(
            Tool::Overlay(Overlay::Maple)
                .applied(forest())
                .and_then(|tile| tile.flora),
            Some(Flora::Tree)
        );
    }

    #[test]
    fn fire_burns_the_forest_down() {
        assert_eq!(
            Power::Fire.touched(forest(), touch(0.0)),
            Some(Tile::new(Ground::SoilLow, Overlay::Infernal))
        );
    }

    #[test]
    fn the_new_tools_are_named_and_drawn() {
        for tool in [
            Tool::Plant(Flora::Tree),
            Tool::Plant(Flora::Bush),
            Tool::Fell,
            Tool::Ore(Ore::Mythril),
            Tool::Mine,
        ] {
            assert!(tool.edits(), "{tool:?}");
            assert!(tool.icon().is_some(), "{tool:?}");
            assert!(!tool.name().is_empty(), "{tool:?}");
        }
    }

    #[test]
    fn a_wall_brush_raises_walls_on_land_and_leaves_the_sea_alone() {
        let soil = Tile::new(Ground::SoilLow, Overlay::Grass);
        let walled = Tool::Wall(Wall::Green).applied(soil).expect("a wall");

        assert_eq!(walled.wall, Some(Wall::Green));
        assert_eq!(walled.overlay, Overlay::Grass);
        assert_eq!(Tool::Wall(Wall::Green).applied(walled), None);
        assert_eq!(
            Tool::Wall(Wall::Green).applied(Tile::bare(Ground::DeepOcean)),
            None
        );
        assert_eq!(Tool::Wall(Wall::Iron).name(), "wall_iron");
        assert_eq!(Tool::Wall(Wall::Iron).swatch(), Some(Wall::Iron.color()));
    }

    #[test]
    fn a_bomb_tool_drops_bombs_instead_of_painting() {
        let tool = Tool::Drop(Bomb::Grenade);

        assert!(!tool.edits());
        assert!(tool.holds());
        assert_eq!(tool.applied(Tile::bare(Ground::Sand)), None);
        assert_eq!(tool.name(), "grenade");
        assert_eq!(tool.icon().map(|found| found.0), Some(Icon::Bomb));
    }
}
