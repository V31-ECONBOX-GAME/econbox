//! The tile vocabulary, with the exact palette WorldBox renders.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum Ground {
    #[default]
    DeepOcean,
    CloseOcean,
    ShallowWaters,
    Sand,
    SoilLow,
    SoilHigh,
    Hills,
    Mountains,
    Summit,
    Lava2,
    Lava3,
}

pub const MAX_HEIGHT: u8 = 255;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layer {
    Ocean,
    Ground,
    Block,
    Lava,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rank {
    Nothing,
    Low,
    High,
}

pub const GENERATOR_GROUNDS: [Ground; 9] = [
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

pub const GROUNDS: [Ground; 11] = [
    Ground::DeepOcean,
    Ground::CloseOcean,
    Ground::ShallowWaters,
    Ground::Sand,
    Ground::SoilLow,
    Ground::SoilHigh,
    Ground::Hills,
    Ground::Mountains,
    Ground::Summit,
    Ground::Lava2,
    Ground::Lava3,
];

impl Ground {
    pub fn is_water(self) -> bool {
        matches!(
            self,
            Self::DeepOcean | Self::CloseOcean | Self::ShallowWaters
        )
    }

    pub fn is_land(self) -> bool {
        !self.is_water()
    }

    pub fn is_lava(self) -> bool {
        matches!(self, Self::Lava2 | Self::Lava3)
    }

    pub fn takes_overlay(self) -> bool {
        matches!(self, Self::Sand | Self::SoilLow | Self::SoilHigh)
    }

    pub fn takes_ore(self) -> bool {
        matches!(self, Self::Hills | Self::Mountains)
    }

    pub fn is_high(self) -> bool {
        matches!(
            self,
            Self::SoilHigh | Self::Hills | Self::Mountains | Self::Summit
        )
    }

    pub fn takes_wall(self) -> bool {
        self.is_land() && !self.is_lava()
    }

    pub fn elevation(self) -> u8 {
        match self {
            Self::DeepOcean => 0,
            Self::CloseOcean => 1,
            Self::ShallowWaters => 2,
            Self::Sand => 3,
            Self::SoilLow => 4,
            Self::SoilHigh => 5,
            Self::Hills => 6,
            Self::Mountains => 7,
            Self::Summit => 8,
            Self::Lava2 => 4,
            Self::Lava3 => 5,
        }
    }

    pub fn height_min(self) -> Option<u8> {
        match self {
            Self::DeepOcean => Some(0),
            Self::CloseOcean => Some(30),
            Self::ShallowWaters => Some(70),
            Self::Sand => Some(98),
            Self::SoilLow => Some(108),
            Self::SoilHigh => Some(128),
            Self::Hills => Some(199),
            Self::Mountains => Some(210),
            Self::Summit => Some(230),
            Self::Lava2 | Self::Lava3 => None,
        }
    }

    pub fn additional_height(self) -> &'static [i32] {
        match self {
            Self::SoilHigh => &[15, 16, 17, 14, 13, 12, 11, 10],
            Self::Hills => &[2, -6],
            Self::Mountains | Self::Summit => &[2, 4],
            _ => &[],
        }
    }

    pub fn from_height(height: u8) -> Self {
        GENERATOR_GROUNDS
            .into_iter()
            .fold(None, |found, ground| match (found, ground.height_min()) {
                (None, Some(_)) => Some(ground),
                (Some(_), Some(min)) if height >= min => Some(ground),
                (found, _) => found,
            })
            .unwrap_or(Self::DeepOcean)
    }

    pub fn layer(self) -> Layer {
        match self {
            Self::DeepOcean | Self::CloseOcean | Self::ShallowWaters => Layer::Ocean,
            Self::Sand | Self::SoilLow | Self::SoilHigh | Self::Hills => Layer::Ground,
            Self::Mountains | Self::Summit => Layer::Block,
            Self::Lava2 | Self::Lava3 => Layer::Lava,
        }
    }

    pub fn rank(self) -> Rank {
        match self {
            Self::SoilLow => Rank::Low,
            Self::SoilHigh => Rank::High,
            _ => Rank::Nothing,
        }
    }

    pub fn is_soil(self) -> bool {
        matches!(self, Self::SoilLow | Self::SoilHigh)
    }

    pub fn is_rock(self) -> bool {
        matches!(self, Self::Hills | Self::Mountains | Self::Summit)
    }

    pub fn decrease_to(self) -> Option<Self> {
        match self {
            Self::SoilLow => Some(Self::Sand),
            Self::SoilHigh => Some(Self::SoilLow),
            Self::Hills => Some(Self::SoilHigh),
            Self::Mountains => Some(Self::Hills),
            Self::Summit => Some(Self::Mountains),
            Self::Lava2 | Self::Lava3 => Some(Self::Sand),
            _ => None,
        }
    }

    pub fn increase_to(self) -> Option<Self> {
        match self {
            Self::ShallowWaters => Some(Self::Sand),
            Self::Sand => Some(Self::SoilLow),
            Self::SoilLow => Some(Self::SoilHigh),
            Self::SoilHigh => Some(Self::Hills),
            Self::Hills => Some(Self::Mountains),
            Self::Mountains => Some(Self::Summit),
            Self::Lava2 | Self::Lava3 => Some(Self::Hills),
            _ => None,
        }
    }

    pub fn freeze_to(self) -> Option<&'static str> {
        match self {
            Self::ShallowWaters => Some("ice"),
            Self::Sand => Some("snow_sand"),
            Self::SoilLow => Some("frozen_low"),
            Self::SoilHigh => Some("frozen_high"),
            Self::Hills => Some("snow_hills"),
            Self::Mountains => Some("snow_block"),
            Self::Summit => Some("snow_summit"),
            _ => None,
        }
    }

    pub fn frozen_color(self) -> Option<[u8; 3]> {
        match self {
            Self::ShallowWaters => Some([0xa7, 0xd6, 0xf4]),
            Self::Sand => Some([0xaf, 0xf5, 0xf1]),
            Self::SoilLow => Some([0xba, 0xd5, 0xd3]),
            Self::SoilHigh => Some([0xd3, 0xe4, 0xe3]),
            Self::Hills => Some([0xe2, 0xed, 0xec]),
            Self::Mountains | Self::Summit => Some([0xfc, 0xfd, 0xfd]),
            _ => None,
        }
    }

    pub fn color(self) -> [u8; 3] {
        match self {
            Self::DeepOcean => [0x33, 0x70, 0xcc],
            Self::CloseOcean => [0x40, 0x84, 0xe2],
            Self::ShallowWaters => [0x55, 0xae, 0xf0],
            Self::Sand => [0xf7, 0xe8, 0x98],
            Self::SoilLow => [0xe2, 0x93, 0x4b],
            Self::SoilHigh => [0xb6, 0x6f, 0x3a],
            Self::Hills => [0x5b, 0x5e, 0x5c],
            Self::Mountains => [0x41, 0x45, 0x45],
            Self::Summit => [0x33, 0x33, 0x33],
            Self::Lava2 => [0xff, 0xac, 0x00],
            Self::Lava3 => [0xff, 0xde, 0x00],
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::DeepOcean => "deep_ocean",
            Self::CloseOcean => "close_ocean",
            Self::ShallowWaters => "shallow_waters",
            Self::Sand => "sand",
            Self::SoilLow => "soil_low",
            Self::SoilHigh => "soil_high",
            Self::Hills => "hills",
            Self::Mountains => "mountains",
            Self::Summit => "summit",
            Self::Lava2 => "lava2",
            Self::Lava3 => "lava3",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum Overlay {
    #[default]
    Bare,
    Grass,
    Jungle,
    Enchanted,
    Infernal,
    Pumpkin,
    Savanna,
    Birch,
    Maple,
    Swamp,
    Blight,
    Mushroom,
    Candy,
    Garlic,
    Lemon,
    Desert,
    Rockland,
    Crystal,
    Frost,
    Flower,
    Starlight,
    Void,
    Paradox,
    Clover,
}

pub const OVERLAYS: [Overlay; 24] = [
    Overlay::Bare,
    Overlay::Grass,
    Overlay::Jungle,
    Overlay::Enchanted,
    Overlay::Infernal,
    Overlay::Pumpkin,
    Overlay::Savanna,
    Overlay::Birch,
    Overlay::Maple,
    Overlay::Swamp,
    Overlay::Blight,
    Overlay::Mushroom,
    Overlay::Candy,
    Overlay::Garlic,
    Overlay::Lemon,
    Overlay::Desert,
    Overlay::Rockland,
    Overlay::Crystal,
    Overlay::Frost,
    Overlay::Flower,
    Overlay::Starlight,
    Overlay::Void,
    Overlay::Paradox,
    Overlay::Clover,
];

impl Overlay {
    pub const fn color(self, high: bool) -> Option<[u8; 3]> {
        let (low, top) = match self {
            Self::Bare => return None,
            Self::Grass => ([0x7e, 0xaf, 0x46], [0x5f, 0x83, 0x3c]),
            Self::Jungle => ([0x46, 0xa0, 0x52], [0x1f, 0x70, 0x20]),
            Self::Enchanted => ([0x8c, 0xdc, 0x6a], [0x76, 0xb1, 0x53]),
            Self::Infernal => ([0x9c, 0x36, 0x26], [0x68, 0x37, 0x2d]),
            Self::Pumpkin => ([0x8f, 0x93, 0x39], [0x69, 0x6c, 0x02]),
            Self::Savanna => ([0xc3, 0xb0, 0x52], [0xa0, 0x8c, 0x3c]),
            Self::Birch => ([0xb4, 0xcc, 0x8a], [0x94, 0xac, 0x6e]),
            Self::Maple => ([0xcf, 0x7a, 0x3a], [0xa8, 0x5a, 0x2a]),
            Self::Swamp => ([0x56, 0x70, 0x4a], [0x3f, 0x56, 0x38]),
            Self::Blight => ([0x6f, 0x4a, 0x64], [0x52, 0x34, 0x48]),
            Self::Mushroom => ([0xa8, 0x66, 0x5a], [0x7f, 0x4a, 0x42]),
            Self::Candy => ([0xee, 0x9f, 0xcf], [0xd2, 0x7c, 0xb2]),
            Self::Garlic => ([0xd9, 0xd2, 0xb4], [0xbd, 0xb5, 0x94]),
            Self::Lemon => ([0xd8, 0xd2, 0x4c], [0xb2, 0xad, 0x32]),
            Self::Desert => ([0xe3, 0xc5, 0x85], [0xc9, 0xa8, 0x66]),
            Self::Rockland => ([0x8e, 0x8f, 0x86], [0x72, 0x73, 0x6b]),
            Self::Crystal => ([0x7f, 0xd8, 0xde], [0x5c, 0xb4, 0xc0]),
            Self::Frost => ([0xd6, 0xea, 0xf2], [0xb4, 0xcf, 0xdc]),
            Self::Flower => ([0xa6, 0xd0, 0x5e], [0x86, 0xb0, 0x48]),
            Self::Starlight => ([0x8f, 0x86, 0xe0], [0x6f, 0x66, 0xc0]),
            Self::Void => ([0x3f, 0x36, 0x56], [0x2a, 0x23, 0x40]),
            Self::Paradox => ([0xc4, 0x6a, 0xd8], [0x9e, 0x4c, 0xb4]),
            Self::Clover => ([0x3f, 0xae, 0x5a], [0x2c, 0x8a, 0x44]),
        };

        Some(if high { top } else { low })
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Bare => "bare",
            Self::Grass => "grass",
            Self::Jungle => "jungle",
            Self::Enchanted => "enchanted",
            Self::Infernal => "infernal",
            Self::Pumpkin => "pumpkin",
            Self::Savanna => "savanna",
            Self::Birch => "birch",
            Self::Maple => "maple",
            Self::Swamp => "swamp",
            Self::Blight => "corrupted",
            Self::Mushroom => "mushroom",
            Self::Candy => "candy",
            Self::Garlic => "garlic",
            Self::Lemon => "lemon",
            Self::Desert => "desert",
            Self::Rockland => "rocklands",
            Self::Crystal => "crystal",
            Self::Frost => "permafrost",
            Self::Flower => "flower",
            Self::Starlight => "celestial",
            Self::Void => "singularity",
            Self::Paradox => "paradox",
            Self::Clover => "clover",
        }
    }

    pub fn fertility(self) -> f32 {
        match self {
            Self::Grass => 1.0,
            Self::Clover => 0.9,
            Self::Jungle => 0.85,
            Self::Birch => 0.8,
            Self::Maple => 0.75,
            Self::Enchanted => 0.7,
            Self::Flower => 0.7,
            Self::Lemon => 0.65,
            Self::Candy => 0.6,
            Self::Savanna => 0.55,
            Self::Starlight => 0.55,
            Self::Swamp => 0.5,
            Self::Garlic => 0.5,
            Self::Mushroom => 0.45,
            Self::Pumpkin => 0.4,
            Self::Paradox => 0.35,
            Self::Crystal => 0.3,
            Self::Rockland => 0.2,
            Self::Desert => 0.15,
            Self::Blight => 0.1,
            Self::Frost => 0.1,
            Self::Infernal => 0.05,
            Self::Void => 0.02,
            Self::Bare => 0.0,
        }
    }

    pub fn is_living(self) -> bool {
        !matches!(
            self,
            Self::Bare | Self::Infernal | Self::Blight | Self::Void
        )
    }

    pub fn grows(self) -> bool {
        self.fertility() >= 0.25
    }

    pub fn speck(self) -> u32 {
        match self {
            Self::Bare => 0,
            Self::Desert | Self::Frost => 6,
            Self::Rockland | Self::Void => 8,
            Self::Infernal | Self::Garlic => 10,
            Self::Pumpkin | Self::Savanna | Self::Crystal => 12,
            Self::Enchanted | Self::Birch | Self::Candy | Self::Lemon => 14,
            Self::Grass | Self::Blight => 16,
            Self::Maple | Self::Starlight => 18,
            Self::Mushroom | Self::Paradox => 20,
            Self::Clover => 22,
            Self::Swamp => 24,
            Self::Flower => 26,
            Self::Jungle => 30,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Flora {
    Sapling,
    Tree,
    Bush,
    Plant,
}

pub const FLORAS: [Flora; 4] = [Flora::Sapling, Flora::Tree, Flora::Bush, Flora::Plant];

const BERRY: [u8; 3] = [0xc8, 0x34, 0x3c];

impl Flora {
    pub fn name(self) -> &'static str {
        match self {
            Self::Sapling => "sapling",
            Self::Tree => "tree",
            Self::Bush => "fruit_bush",
            Self::Plant => "plant",
        }
    }

    pub fn tint(self, cover: [u8; 3]) -> [u8; 3] {
        match self {
            Self::Sapling => scale(cover, 0.82),
            Self::Tree => scale(cover, 0.58),
            Self::Bush => scale(cover, 0.7),
            Self::Plant => scale(cover, 0.9),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Ore {
    Stone,
    Metal,
    Silver,
    Mythril,
    Adamantine,
    Gold,
}

pub const ORES: [Ore; 6] = [
    Ore::Stone,
    Ore::Metal,
    Ore::Silver,
    Ore::Mythril,
    Ore::Adamantine,
    Ore::Gold,
];

impl Ore {
    pub const fn color(self) -> [u8; 3] {
        match self {
            Self::Stone => [0x8a, 0x8c, 0x88],
            Self::Metal => [0xb0, 0x78, 0x48],
            Self::Silver => [0xd8, 0xdc, 0xe4],
            Self::Mythril => [0x6a, 0xd8, 0xc8],
            Self::Adamantine => [0x9a, 0x5c, 0xd8],
            Self::Gold => [0xf2, 0xc2, 0x3a],
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Stone => "stone",
            Self::Metal => "metal",
            Self::Silver => "silver",
            Self::Mythril => "mythril",
            Self::Adamantine => "adamantine",
            Self::Gold => "gold",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Wall {
    Order,
    Evil,
    Ancient,
    Wild,
    Green,
    Iron,
    Light,
}

pub const WALLS: [Wall; 7] = [
    Wall::Order,
    Wall::Evil,
    Wall::Ancient,
    Wall::Wild,
    Wall::Green,
    Wall::Iron,
    Wall::Light,
];

impl Wall {
    pub fn name(self) -> &'static str {
        match self {
            Self::Order => "wall_order",
            Self::Evil => "wall_evil",
            Self::Ancient => "wall_ancient",
            Self::Wild => "wall_wild",
            Self::Green => "wall_green",
            Self::Iron => "wall_iron",
            Self::Light => "wall_light",
        }
    }

    pub fn color(self) -> [u8; 3] {
        match self {
            Self::Order => [0x9a, 0xa0, 0xa8],
            Self::Evil => [0x6a, 0x3c, 0x58],
            Self::Ancient => [0xb8, 0xa2, 0x74],
            Self::Wild => [0x9a, 0x68, 0x3a],
            Self::Green => [0x58, 0x9c, 0x48],
            Self::Iron => [0x6c, 0x78, 0x88],
            Self::Light => [0xf0, 0xe0, 0x9a],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Tile {
    pub ground: Ground,
    pub overlay: Overlay,
    pub flora: Option<Flora>,
    pub ore: Option<Ore>,
    pub wall: Option<Wall>,
    pub frozen: bool,
}

impl Tile {
    pub fn new(ground: Ground, overlay: Overlay) -> Self {
        Self {
            ground,
            overlay: if ground.takes_overlay() {
                overlay
            } else {
                Overlay::Bare
            },
            flora: None,
            ore: None,
            wall: None,
            frozen: false,
        }
    }

    pub fn bare(ground: Ground) -> Self {
        Self::new(ground, Overlay::Bare)
    }

    pub fn with_ground(self, ground: Ground) -> Self {
        Self::new(ground, self.overlay)
            .with_flora(self.flora)
            .with_ore(self.ore)
            .with_wall(self.wall)
    }

    pub fn with_overlay(self, overlay: Overlay) -> Self {
        Self::new(self.ground, overlay)
            .with_flora(self.flora)
            .with_ore(self.ore)
            .with_wall(self.wall)
    }

    pub fn with_wall(self, wall: Option<Wall>) -> Self {
        Self {
            wall: wall.filter(|_| self.ground.takes_wall()),
            ..self
        }
    }

    pub fn with_frozen(self, frozen: bool) -> Self {
        Self {
            frozen: frozen && self.ground.freeze_to().is_some(),
            ..self
        }
    }

    pub fn blocks(self) -> bool {
        self.wall.is_some()
    }

    pub fn with_flora(self, flora: Option<Flora>) -> Self {
        Self {
            flora: flora.filter(|_| self.overlay.grows()),
            ..self
        }
    }

    pub fn with_ore(self, ore: Option<Ore>) -> Self {
        Self {
            ore: ore.filter(|_| self.ground.takes_ore()),
            ..self
        }
    }

    pub fn is_water(self) -> bool {
        self.ground.is_water()
    }

    pub fn is_land(self) -> bool {
        self.ground.is_land()
    }

    pub fn is_lava(self) -> bool {
        self.ground.is_lava()
    }

    pub fn fertility(self) -> f32 {
        self.overlay.fertility()
    }

    pub fn cover(self) -> [u8; 3] {
        if let Some(frozen) = self.ground.frozen_color().filter(|_| self.frozen) {
            return frozen;
        }

        self.overlay
            .color(self.ground.is_high())
            .unwrap_or_else(|| self.ground.color())
    }

    pub fn color(self) -> [u8; 3] {
        if let Some(wall) = self.wall {
            return wall.color();
        }

        match (self.flora, self.ore) {
            (Some(flora), _) => flora.tint(self.cover()),
            (None, Some(ore)) => ore.color(),
            (None, None) => self.cover(),
        }
    }

    pub fn name(self) -> String {
        if let Some(top) = self.ground.freeze_to().filter(|_| self.frozen) {
            return format!("{}:{top}", self.ground.name());
        }

        match self.overlay {
            Overlay::Bare => self.ground.name().to_string(),
            overlay => format!(
                "{}:{}_{}",
                self.ground.name(),
                overlay.name(),
                if self.ground.is_high() { "high" } else { "low" }
            ),
        }
    }

    pub fn object(self) -> Option<&'static str> {
        self.wall
            .map(Wall::name)
            .or_else(|| self.flora.map(Flora::name))
            .or_else(|| self.ore.map(Ore::name))
    }
}

const HASH_A: u32 = 0x27d4_eb2d;
const HASH_B: u32 = 0x1656_67b1;

fn hash(x: u32, y: u32) -> u32 {
    let mut h = x.wrapping_mul(HASH_A) ^ y.wrapping_mul(HASH_B);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297a_2d39);
    h ^ (h >> 15)
}

fn nudge(channel: u8, by: i32) -> u8 {
    (i32::from(channel) + by).clamp(0, 255) as u8
}

fn scale(color: [u8; 3], by: f32) -> [u8; 3] {
    color.map(|channel| (f32::from(channel) * by).clamp(0.0, 255.0) as u8)
}

impl Tile {
    pub fn speck_chance(self) -> u32 {
        if self.wall.is_some() {
            return 0;
        }

        match (self.flora, self.ore) {
            (Some(Flora::Tree), _) => 34,
            (Some(_), _) => 20,
            (None, Some(_)) => 0,
            (None, None) => self.overlay.speck(),
        }
    }

    pub fn grain(self) -> i32 {
        match self.ground {
            Ground::DeepOcean | Ground::CloseOcean => 4,
            Ground::ShallowWaters => 7,
            Ground::Sand => 9,
            Ground::Hills | Ground::Mountains | Ground::Summit => 11,
            _ => 14,
        }
    }

    pub fn textured(self, x: u32, y: u32) -> [u8; 3] {
        let noise = hash(x, y);
        let base = self.color();

        if self.wall.is_some() {
            let by = (noise % 9) as i32 - 4;
            return base.map(|channel| nudge(channel, by));
        }

        if self.flora == Some(Flora::Bush) && noise.is_multiple_of(3) {
            return BERRY;
        }

        if self.ore.is_some() && noise.is_multiple_of(4) {
            return scale(base, 1.18);
        }

        let speck = self.speck_chance();

        if speck > 0 && noise % 100 < speck {
            return scale(base, if noise.is_multiple_of(3) { 0.66 } else { 0.80 });
        }

        let grain = self.grain();
        let step = (noise >> 9) % (grain as u32 * 2 + 1);
        let by = step as i32 - grain;

        [
            nudge(base[0], by),
            nudge(base[1], by),
            nudge(base[2], (by as f32 * 0.7) as i32),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_names_match_the_ones_worldbox_saves() {
        assert_eq!(Tile::bare(Ground::DeepOcean).name(), "deep_ocean");
        assert_eq!(Tile::bare(Ground::ShallowWaters).name(), "shallow_waters");
        assert_eq!(
            Tile::new(Ground::SoilHigh, Overlay::Grass).name(),
            "soil_high:grass_high"
        );
        assert_eq!(
            Tile::new(Ground::SoilLow, Overlay::Jungle).name(),
            "soil_low:jungle_low"
        );
        assert_eq!(
            Tile::new(Ground::SoilLow, Overlay::Infernal).name(),
            "soil_low:infernal_low"
        );
        assert_eq!(
            Tile::new(Ground::SoilLow, Overlay::Blight).name(),
            "soil_low:corrupted_low"
        );
        assert_eq!(
            Tile::new(Ground::SoilHigh, Overlay::Frost).name(),
            "soil_high:permafrost_high"
        );
        assert_eq!(
            Tile::new(Ground::SoilLow, Overlay::Starlight).name(),
            "soil_low:celestial_low"
        );
        assert_eq!(
            Tile::new(Ground::SoilLow, Overlay::Void).name(),
            "soil_low:singularity_low"
        );
        assert_eq!(
            Tile::new(Ground::SoilLow, Overlay::Rockland).name(),
            "soil_low:rocklands_low"
        );
    }

    #[test]
    fn the_colours_match_the_ones_worldbox_renders() {
        assert_eq!(Tile::bare(Ground::DeepOcean).color(), [0x33, 0x70, 0xcc]);
        assert_eq!(Tile::bare(Ground::CloseOcean).color(), [0x40, 0x84, 0xe2]);
        assert_eq!(
            Tile::bare(Ground::ShallowWaters).color(),
            [0x55, 0xae, 0xf0]
        );
        assert_eq!(Tile::bare(Ground::Sand).color(), [0xf7, 0xe8, 0x98]);
        assert_eq!(Tile::bare(Ground::Mountains).color(), [0x41, 0x45, 0x45]);
        assert_eq!(
            Tile::new(Ground::SoilHigh, Overlay::Grass).color(),
            [0x5f, 0x83, 0x3c]
        );
        assert_eq!(
            Tile::new(Ground::SoilLow, Overlay::Grass).color(),
            [0x7e, 0xaf, 0x46]
        );
    }

    #[test]
    fn water_and_rock_refuse_an_overlay() {
        for ground in [
            Ground::DeepOcean,
            Ground::CloseOcean,
            Ground::ShallowWaters,
            Ground::Hills,
            Ground::Mountains,
        ] {
            let tile = Tile::new(ground, Overlay::Grass);

            assert_eq!(tile.overlay, Overlay::Bare, "{}", ground.name());
            assert_eq!(tile.color(), ground.color());
        }
    }

    #[test]
    fn the_elevation_order_runs_from_the_deep_to_the_peaks() {
        let ladder = [
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

        assert!(
            ladder
                .windows(2)
                .all(|pair| pair[0].elevation() < pair[1].elevation())
        );
    }

    #[test]
    fn every_tile_has_a_distinct_colour() {
        let mut seen: Vec<[u8; 3]> = Vec::new();

        for ground in GROUNDS {
            for overlay in OVERLAYS {
                seen.push(Tile::new(ground, overlay).color());
            }
        }

        seen.sort_unstable();
        let before = seen.len();
        seen.dedup();

        assert_eq!(seen.len(), 57, "{} distinct of {before}", seen.len());
    }

    #[test]
    fn every_biome_has_its_own_name_and_its_own_pair_of_colours() {
        let mut names: Vec<&str> = OVERLAYS.iter().map(|overlay| overlay.name()).collect();
        let mut colours: Vec<[u8; 3]> = OVERLAYS
            .iter()
            .skip(1)
            .flat_map(|overlay| [overlay.color(false), overlay.color(true)])
            .map(|colour| colour.expect("a living biome has a colour"))
            .chain(GROUNDS.map(Ground::color))
            .collect();
        names.sort_unstable();
        names.dedup();
        colours.sort_unstable();
        colours.dedup();

        assert_eq!(names.len(), OVERLAYS.len());
        assert_eq!(colours.len(), (OVERLAYS.len() - 1) * 2 + GROUNDS.len());
    }

    #[test]
    fn high_ground_wears_the_darker_shade_of_every_biome() {
        for overlay in OVERLAYS.iter().skip(1) {
            let low: u32 = overlay
                .color(false)
                .expect("low")
                .map(u32::from)
                .iter()
                .sum();
            let high: u32 = overlay
                .color(true)
                .expect("high")
                .map(u32::from)
                .iter()
                .sum();

            assert!(high < low, "{}", overlay.name());
        }
    }

    #[test]
    fn every_fertility_is_a_share_and_only_bare_ground_is_barren() {
        for overlay in OVERLAYS {
            let fertility = overlay.fertility();

            assert!((0.0..=1.0).contains(&fertility), "{}", overlay.name());
            assert_eq!(
                fertility == 0.0,
                overlay == Overlay::Bare,
                "{}",
                overlay.name()
            );
        }
    }

    #[test]
    fn texture_keeps_a_tile_close_to_its_own_colour() {
        let tile = Tile::new(Ground::SoilLow, Overlay::Grass);
        let base = tile.color();

        for y in 0..64 {
            for x in 0..64 {
                let shade = tile.textured(x, y);

                for channel in 0..3 {
                    let drift = i32::from(shade[channel]) - i32::from(base[channel]);
                    assert!(drift.abs() <= 90, "{drift} at {x},{y}");
                }
            }
        }
    }

    #[test]
    fn texture_is_the_same_every_time_for_the_same_tile() {
        let tile = Tile::new(Ground::SoilHigh, Overlay::Jungle);

        assert_eq!(tile.textured(12, 34), tile.textured(12, 34));
        assert_ne!(tile.textured(12, 34), tile.textured(13, 34));
    }

    #[test]
    fn grass_is_speckled_but_the_open_sea_is_not() {
        let grass = Tile::new(Ground::SoilLow, Overlay::Grass);
        let sea = Tile::bare(Ground::DeepOcean);

        assert!(grass.speck_chance() > 0);
        assert_eq!(sea.speck_chance(), 0);
        assert!(grass.grain() > sea.grain());
    }

    #[test]
    fn a_patch_of_grass_shows_many_shades() {
        let tile = Tile::new(Ground::SoilLow, Overlay::Grass);
        let mut shades: Vec<_> = (0..32)
            .flat_map(|y| (0..32).map(move |x| (x, y)))
            .map(|(x, y)| tile.textured(x, y))
            .collect();
        shades.sort_unstable();
        shades.dedup();

        assert!(shades.len() > 12, "only {} shades", shades.len());
    }

    #[test]
    fn only_the_two_lava_grounds_burn() {
        let burning: Vec<Ground> = GROUNDS
            .into_iter()
            .filter(|ground| ground.is_lava())
            .collect();

        assert_eq!(burning, [Ground::Lava2, Ground::Lava3]);
        assert!(Tile::bare(Ground::Lava3).is_lava());
        assert!(!Tile::bare(Ground::Hills).is_lava());
    }

    #[test]
    fn grass_feeds_more_than_lava_fields() {
        assert!(Overlay::Grass.fertility() > Overlay::Jungle.fertility());
        assert!(Overlay::Jungle.fertility() > Overlay::Infernal.fertility());
        assert_eq!(Overlay::Bare.fertility(), 0.0);
    }

    #[test]
    fn trees_take_root_only_where_the_biome_can_feed_them() {
        let meadow = Tile::new(Ground::SoilLow, Overlay::Grass);
        let dunes = Tile::new(Ground::SoilLow, Overlay::Desert);
        let sea = Tile::bare(Ground::ShallowWaters);

        assert_eq!(
            meadow.with_flora(Some(Flora::Tree)).flora,
            Some(Flora::Tree)
        );
        assert_eq!(dunes.with_flora(Some(Flora::Tree)).flora, None);
        assert_eq!(sea.with_flora(Some(Flora::Bush)).flora, None);
        assert!(OVERLAYS.iter().filter(|overlay| overlay.grows()).count() >= 15);
    }

    #[test]
    fn reseeding_a_forest_keeps_its_trees_unless_the_new_biome_is_barren() {
        let forest = Tile::new(Ground::SoilHigh, Overlay::Grass).with_flora(Some(Flora::Tree));

        assert_eq!(forest.with_overlay(Overlay::Birch).flora, Some(Flora::Tree));
        assert_eq!(forest.with_overlay(Overlay::Desert).flora, None);
        assert_eq!(
            forest.with_overlay(Overlay::Desert).overlay,
            Overlay::Desert
        );
    }

    #[test]
    fn ore_lies_only_in_the_hills_and_mountains() {
        for ground in GROUNDS {
            let tile = Tile::bare(ground).with_ore(Some(Ore::Gold));

            assert_eq!(tile.ore.is_some(), ground.takes_ore(), "{}", ground.name());
        }
    }

    #[test]
    fn a_deposit_survives_a_climb_but_not_a_fall_into_soil() {
        let vein = Tile::bare(Ground::Hills).with_ore(Some(Ore::Silver));

        assert_eq!(vein.with_ground(Ground::Mountains).ore, Some(Ore::Silver));
        assert_eq!(vein.with_ground(Ground::SoilHigh).ore, None);
    }

    #[test]
    fn trees_and_deposits_show_on_the_map_and_name_themselves() {
        let meadow = Tile::new(Ground::SoilLow, Overlay::Grass);
        let tree = meadow.with_flora(Some(Flora::Tree));
        let bush = meadow.with_flora(Some(Flora::Bush));
        let vein = Tile::bare(Ground::Mountains).with_ore(Some(Ore::Mythril));
        let shade = |colour: [u8; 3]| colour.map(u32::from).iter().sum::<u32>();

        assert!(shade(tree.color()) < shade(meadow.color()));
        assert_eq!(vein.color(), Ore::Mythril.color());
        assert_eq!(tree.object(), Some("tree"));
        assert_eq!(bush.object(), Some("fruit_bush"));
        assert_eq!(vein.object(), Some("mythril"));
        assert_eq!(meadow.object(), None);
        assert_eq!(tree.name(), meadow.name());
        assert!((0..30).any(|x| bush.textured(x, 0) == BERRY));
    }

    #[test]
    fn every_deposit_has_its_own_colour_and_name() {
        let mut colours: Vec<[u8; 3]> = ORES.map(Ore::color).to_vec();
        let mut names: Vec<&str> = ORES.map(Ore::name).to_vec();
        colours.sort_unstable();
        colours.dedup();
        names.sort_unstable();
        names.dedup();

        assert_eq!(colours.len(), ORES.len());
        assert_eq!(names.len(), ORES.len());
        assert_eq!(FLORAS.map(Flora::name).len(), 4);
    }

    #[test]
    fn the_summit_is_the_highest_rock_and_keeps_the_worldbox_colour() {
        let summit = Tile::bare(Ground::Summit);

        assert!(Ground::Summit.elevation() > Ground::Mountains.elevation());
        assert!(Ground::Summit.is_high());
        assert_eq!(summit.name(), "summit");
        assert_eq!(summit.color(), [0x33, 0x33, 0x33]);
        assert_eq!(
            Tile::new(Ground::Summit, Overlay::Grass).overlay,
            Overlay::Bare
        );
    }

    #[test]
    fn a_wall_stands_on_land_and_shows_its_own_colour() {
        let soil = Tile::new(Ground::SoilLow, Overlay::Grass);
        let walled = soil.with_wall(Some(Wall::Iron));

        assert!(walled.blocks());
        assert_eq!(walled.color(), Wall::Iron.color());
        assert_eq!(walled.object(), Some("wall_iron"));
        assert_eq!(walled.name(), soil.name());
        assert!(!soil.blocks());
    }

    #[test]
    fn water_and_lava_refuse_a_wall() {
        for ground in [Ground::DeepOcean, Ground::ShallowWaters, Ground::Lava3] {
            assert_eq!(Tile::bare(ground).with_wall(Some(Wall::Order)).wall, None);
        }
    }

    #[test]
    fn a_wall_survives_a_change_of_ground_until_the_sea_takes_it() {
        let walled = Tile::bare(Ground::SoilLow).with_wall(Some(Wall::Wild));

        assert_eq!(walled.with_ground(Ground::Hills).wall, Some(Wall::Wild));
        assert_eq!(walled.with_overlay(Overlay::Jungle).wall, Some(Wall::Wild));
        assert_eq!(walled.with_ground(Ground::CloseOcean).wall, None);
        assert_eq!(Tile::bare(walled.ground).wall, None);
    }

    #[test]
    fn every_wall_has_a_name_and_colour_of_its_own() {
        let mut names: Vec<&str> = WALLS.iter().map(|wall| wall.name()).collect();
        let mut colours: Vec<[u8; 3]> = WALLS.iter().map(|wall| wall.color()).collect();
        names.sort_unstable();
        names.dedup();
        colours.sort_unstable();
        colours.dedup();

        assert_eq!(names.len(), WALLS.len());
        assert_eq!(colours.len(), WALLS.len());
    }

    #[test]
    fn the_height_bands_are_the_ones_the_worldbox_generator_uses() {
        let bands = [
            (0, Ground::DeepOcean),
            (29, Ground::DeepOcean),
            (30, Ground::CloseOcean),
            (69, Ground::CloseOcean),
            (70, Ground::ShallowWaters),
            (97, Ground::ShallowWaters),
            (98, Ground::Sand),
            (107, Ground::Sand),
            (108, Ground::SoilLow),
            (127, Ground::SoilLow),
            (128, Ground::SoilHigh),
            (198, Ground::SoilHigh),
            (199, Ground::Hills),
            (209, Ground::Hills),
            (210, Ground::Mountains),
            (229, Ground::Mountains),
            (230, Ground::Summit),
            (255, Ground::Summit),
        ];

        for (height, ground) in bands {
            assert_eq!(Ground::from_height(height), ground, "{height}");
        }
    }

    #[test]
    fn every_generator_ground_starts_where_its_height_band_starts() {
        for ground in GENERATOR_GROUNDS {
            let min = ground
                .height_min()
                .expect("a generator ground has a height");

            assert_eq!(Ground::from_height(min), ground);
        }
        assert_eq!(Ground::Lava3.height_min(), None);
    }

    #[test]
    fn only_high_soil_and_rock_lift_neighbouring_heights() {
        assert_eq!(
            Ground::SoilHigh.additional_height(),
            &[15, 16, 17, 14, 13, 12, 11, 10]
        );
        assert_eq!(Ground::Hills.additional_height(), &[2, -6]);
        assert_eq!(Ground::Mountains.additional_height(), &[2, 4]);
        assert_eq!(Ground::Summit.additional_height(), &[2, 4]);
        assert!(Ground::SoilLow.additional_height().is_empty());
    }

    #[test]
    fn a_frozen_mountain_shows_snow_until_it_is_terraformed() {
        let peak = Tile::bare(Ground::Mountains).with_frozen(true);

        assert!(peak.frozen);
        assert_eq!(peak.color(), [0xfc, 0xfd, 0xfd]);
        assert_eq!(peak.name(), "mountains:snow_block");
        assert!(!peak.with_ground(Ground::Hills).frozen);
        assert!(!Tile::bare(Ground::DeepOcean).with_frozen(true).frozen);
    }

    #[test]
    fn removing_a_mountain_steps_down_through_the_rock() {
        assert_eq!(Ground::Summit.decrease_to(), Some(Ground::Mountains));
        assert_eq!(Ground::Mountains.decrease_to(), Some(Ground::Hills));
        assert_eq!(Ground::Hills.decrease_to(), Some(Ground::SoilHigh));
        assert!(Ground::Summit.is_rock() && !Ground::SoilHigh.is_rock());
        assert_eq!(Ground::Mountains.layer(), Layer::Block);
        assert_eq!(Ground::Hills.layer(), Layer::Ground);
        assert_eq!(Ground::SoilLow.rank(), Rank::Low);
        assert_eq!(Ground::Sand.rank(), Rank::Nothing);
    }

    #[test]
    fn a_wall_is_drawn_as_one_flat_block() {
        let walled = Tile::bare(Ground::SoilLow).with_wall(Some(Wall::Light));
        let base = Wall::Light.color();

        for x in 0..32 {
            let shade = walled.textured(x, 7);

            for channel in 0..3 {
                assert!((i32::from(shade[channel]) - i32::from(base[channel])).abs() <= 4);
            }
        }
    }
}
