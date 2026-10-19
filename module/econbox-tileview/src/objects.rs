use crate::art::Pixmap;
use crate::look;
use crate::thing_art;
use crate::things::{self, Biome, Family, Kind, Pot, Variant, Wind};
use econbox_tile::{Flora, Ore, Overlay, Tile, Wall};
use econbox_tilemap::TileMap;
use std::collections::HashMap;
use std::sync::OnceLock;

pub const WALL_FRAMES: u32 = 6;
pub const WALL_SPEED: f32 = 4.0;
pub const WALL_SIZE: (u16, u16) = (8, 8);
pub const WALL_PIVOT: (f32, f32) = (4.0, 2.0);
pub const SHADOW_SPREAD: u16 = 3;

pub fn biome(overlay: Overlay) -> &'static Biome {
    match overlay {
        Overlay::Bare | Overlay::Grass | Overlay::Pumpkin => &things::BIOME_GRASS,
        Overlay::Jungle => &things::BIOME_JUNGLE,
        Overlay::Enchanted => &things::BIOME_ENCHANTED,
        Overlay::Infernal => &things::BIOME_INFERNAL,
        Overlay::Savanna => &things::BIOME_SAVANNA,
        Overlay::Birch => &things::BIOME_BIRCH,
        Overlay::Maple => &things::BIOME_MAPLE,
        Overlay::Swamp => &things::BIOME_SWAMP,
        Overlay::Blight => &things::BIOME_CORRUPTED,
        Overlay::Mushroom => &things::BIOME_MUSHROOM,
        Overlay::Candy => &things::BIOME_CANDY,
        Overlay::Garlic => &things::BIOME_GARLIC,
        Overlay::Lemon => &things::BIOME_LEMON,
        Overlay::Desert => &things::BIOME_DESERT,
        Overlay::Rockland => &things::BIOME_ROCKLANDS,
        Overlay::Crystal => &things::BIOME_CRYSTAL,
        Overlay::Frost => &things::BIOME_PERMAFROST,
        Overlay::Flower => &things::BIOME_FLOWER,
        Overlay::Starlight => &things::BIOME_CELESTIAL,
        Overlay::Void => &things::BIOME_SINGULARITY,
        Overlay::Paradox => &things::BIOME_PARADOX,
        Overlay::Clover => &things::BIOME_CLOVER,
    }
}

pub fn mineral(ore: Ore) -> &'static Kind {
    match ore {
        Ore::Stone => &things::MINERAL_STONE,
        Ore::Metal => &things::MINERAL_METALS,
        Ore::Silver => &things::MINERAL_SILVER,
        Ore::Mythril => &things::MINERAL_MYTHRIL,
        Ore::Adamantine => &things::MINERAL_ADAMANTINE,
        Ore::Gold => &things::MINERAL_GOLD,
    }
}

pub fn hash(x: u32, y: u32, salt: u32) -> u32 {
    let mut h =
        x.wrapping_mul(0x27d4_eb2d) ^ y.wrapping_mul(0x1656_67b1) ^ salt.wrapping_mul(0x9e37_79b9);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297a_2d39);
    h ^ (h >> 15)
}

pub fn pick(pots: &[Pot], roll: u32) -> Option<&'static Kind> {
    let total: u32 = pots.iter().map(|pot| u32::from(pot.weight)).sum();
    if total == 0 {
        return None;
    }
    let mut at = roll % total;
    for pot in pots {
        let weight = u32::from(pot.weight);
        if at < weight {
            return Some(pot.kind);
        }
        at -= weight;
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sprite {
    Kind {
        kind: &'static Kind,
        variant: usize,
        bare: bool,
    },
    Wall(Wall),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placed {
    pub sprite: Sprite,
    pub x: u32,
    pub y: u32,
    pub flip: bool,
    pub seed: u32,
}

impl Placed {
    pub fn variant(&self) -> Variant {
        match self.sprite {
            Sprite::Kind {
                kind,
                variant,
                bare,
            } => {
                let list = if bare && !kind.bare.is_empty() {
                    kind.bare
                } else {
                    kind.variants
                };
                list[variant % list.len()]
            }
            Sprite::Wall(_) => Variant {
                width: WALL_SIZE.0,
                height: WALL_SIZE.1,
                pivot_x: WALL_PIVOT.0,
                pivot_y: WALL_PIVOT.1,
                frames: WALL_FRAMES as u16,
            },
        }
    }

    pub fn kind(&self) -> Option<&'static Kind> {
        match self.sprite {
            Sprite::Kind { kind, .. } => Some(kind),
            Sprite::Wall(_) => None,
        }
    }
}

fn kind_at(tile: Tile, x: u32, y: u32) -> Option<&'static Kind> {
    let home = biome(tile.overlay);
    let roll = hash(x, y, 11);

    if let Some(flora) = tile.flora {
        let pots = match flora {
            Flora::Tree => home.trees,
            Flora::Sapling => home.plants,
            Flora::Bush => home.bushes,
        };
        let fallback = match flora {
            Flora::Tree => things::BIOME_GRASS.trees,
            Flora::Sapling => things::BIOME_GRASS.plants,
            Flora::Bush => things::BIOME_GRASS.bushes,
        };
        return pick(pots, roll).or_else(|| pick(fallback, roll));
    }

    tile.ore.map(mineral)
}

pub fn objects(tile: Tile, x: u32, y: u32) -> Vec<Placed> {
    let mut found = Vec::new();
    let seed = hash(x, y, 29);

    if let Some(wall) = tile.wall {
        found.push(Placed {
            sprite: Sprite::Wall(wall),
            x,
            y,
            flip: false,
            seed,
        });
    }

    if let Some(kind) = kind_at(tile, x, y) {
        let variant = (hash(x, y, 37) % kind.variants.len().max(1) as u32) as usize;
        found.push(Placed {
            sprite: Sprite::Kind {
                kind,
                variant,
                bare: false,
            },
            x,
            y,
            flip: kind.flip && hash(x, y, 53) & 1 == 1,
            seed,
        });
    }

    found
}

pub fn kind_of(tile: Tile, x: u32, y: u32) -> Option<&'static Kind> {
    kind_at(tile, x, y)
}

pub fn shadow(kind: &Kind, sprite: &Pixmap) -> Option<Pixmap> {
    let [bound_x, bound_y, distortion] = kind.shadow?;
    let width = u16::from(sprite.width);
    let height = u16::from(sprite.height);
    let mut mask = Pixmap::blank((width + SHADOW_SPREAD).min(255) as u8, sprite.height, 0, 0);
    let first = (bound_x * f32::from(width)) as i32;
    let squash = (f32::from(height) * bound_y) as i32;
    let mut points = Vec::new();

    for i in 0..i32::from(width) {
        for j in 0..i32::from(height) {
            let row = (i32::from(height) - 1 - j) as u8;
            if sprite.at(i as u8, row)[3] == 0 || i < first {
                continue;
            }
            let y = if j > squash {
                (j as f32 * distortion) as i32
            } else {
                j
            };
            points.push((i, y));
        }
    }

    let mut set = |i: i32, j: i32| {
        if i >= 0 && j >= 0 && i < i32::from(mask.width) && j < i32::from(height) {
            let row = (i32::from(height) - 1 - j) as u8;
            mask.paint(i as u8, row, [0, 0, 0, 255]);
        }
    };
    for &(i, j) in &points {
        set(i, j);
    }
    for &(i, j) in &points {
        set(i + 1, j);
        set(i + 2, j);
        set(i + 1, j + 1);
    }

    Some(mask)
}

pub struct Icons {
    icons: HashMap<&'static str, Vec<[u8; 4]>>,
}

pub fn icons() -> &'static Icons {
    static ICONS: OnceLock<Icons> = OnceLock::new();
    ICONS.get_or_init(|| Icons {
        icons: things::KINDS
            .iter()
            .map(|kind| (kind.id, thing_art::mini(kind)))
            .collect(),
    })
}

impl Icons {
    pub fn pixel(&self, kind: &Kind, x: u32, y_from_bottom: u32) -> Option<[u8; 4]> {
        let (w, h) = (u32::from(kind.mini.0), u32::from(kind.mini.1));
        if x >= w || y_from_bottom >= h {
            return None;
        }
        let icon = self.icons.get(kind.id)?;
        let row = h - 1 - y_from_bottom;
        icon.get((row * w + x) as usize)
            .copied()
            .filter(|pixel| pixel[3] > 0)
    }
}

pub const REACH_ICON: i32 = 2;

pub fn icon_pixel(map: &TileMap, x: u32, y: u32) -> Option<[u8; 4]> {
    let icons = icons();
    for dy in 0..=REACH_ICON {
        for dx in [0, -1, 1, -2, 2] {
            let ax = x as i32 + dx;
            let ay = y as i32 + dy;
            if ax < 0 || ay < 0 || ax as u32 >= map.width || ay as u32 >= map.height {
                continue;
            }
            let (ax, ay) = (ax as u32, ay as u32);
            let Some(kind) = kind_at(map.at(ax, ay), ax, ay) else {
                continue;
            };
            let [left, right, top, bottom] = kind.fundament.map(i32::from);
            if -dx < -left || -dx > right || dy > top || -dy > bottom {
                continue;
            }
            let ix = (x as i32 - (ax as i32 - left)) as u32;
            let iy = (bottom + dy) as u32;
            if let Some(pixel) = icons.pixel(kind, ix, iy) {
                return Some(pixel);
            }
        }
    }
    None
}

pub fn base_color(tile: Tile) -> [u8; 3] {
    look::of(tile).color()
}

pub fn wind_mode(kind: Option<&Kind>) -> u32 {
    match kind.map(|kind| kind.wind) {
        Some(Wind::Leaves) => 1,
        Some(Wind::Celestial) => 2,
        _ => 0,
    }
}

pub fn family(placed: &Placed) -> Option<Family> {
    placed.kind().map(|kind| kind.family)
}

#[cfg(test)]
mod tests {
    use super::*;
    use econbox_tile::Ground;

    #[test]
    fn a_grass_tree_is_one_of_the_three_green_trees() {
        let tile = Tile::new(Ground::SoilLow, Overlay::Grass).with_flora(Some(Flora::Tree));
        for x in 0..20 {
            let placed = objects(tile, x, 3);
            let kind = placed[0].kind().unwrap();
            assert!(kind.id.starts_with("tree_green_"), "{}", kind.id);
        }
    }

    #[test]
    fn each_ore_is_its_own_mineral_and_a_wall_comes_first() {
        let tile = Tile::bare(Ground::Hills)
            .with_ore(Some(Ore::Gold))
            .with_wall(Some(Wall::Iron));
        let placed = objects(tile, 4, 4);

        assert_eq!(placed[0].sprite, Sprite::Wall(Wall::Iron));
        assert_eq!(placed[1].kind().unwrap().id, "mineral_gold");
    }

    #[test]
    fn weighted_picks_follow_the_weights() {
        let pots = things::BIOME_SAVANNA.trees;
        let mut big = 0;
        for roll in 0..1200 {
            if pick(pots, roll).unwrap().id.contains("big") {
                big += 1;
            }
        }
        assert_eq!(big, 1200 * 2 / 12);
    }

    #[test]
    fn the_shadow_takes_the_right_half_and_squashes_the_top() {
        let kind = &things::TREE_GREEN_1;
        let sprite = thing_art::frame(kind, 4, 0, false);
        let mask = shadow(kind, &sprite).unwrap();

        assert_eq!(mask.width, sprite.width + SHADOW_SPREAD as u8);
        let top_rows = (0..(sprite.height / 2))
            .flat_map(|y| (0..mask.width).map(move |x| (x, y)))
            .filter(|&(x, y)| mask.filled(x, y))
            .count();
        assert_eq!(top_rows, 0);
        assert!(mask.opaque_count() > 0);
        assert!((0..mask.width / 3).all(|x| !mask.filled(x, mask.height - 1)));
    }

    #[test]
    fn a_tree_shows_its_three_by_two_icon_on_the_map() {
        let mut map = TileMap::filled(9, 9, Tile::new(Ground::SoilLow, Overlay::Grass));
        map.set(4, 5, map.at(4, 5).with_flora(Some(Flora::Tree)));

        for (x, y) in [(3, 5), (4, 5), (5, 5), (3, 4), (4, 4), (5, 4)] {
            assert!(icon_pixel(&map, x, y).is_some(), "{x},{y}");
        }
        for (x, y) in [(2, 5), (6, 5), (4, 3), (4, 6)] {
            assert!(icon_pixel(&map, x, y).is_none(), "{x},{y}");
        }
    }
}
