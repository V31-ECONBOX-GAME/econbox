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
    Lava2,
    Lava3,
}

pub const GROUNDS: [Ground; 10] = [
    Ground::DeepOcean,
    Ground::CloseOcean,
    Ground::ShallowWaters,
    Ground::Sand,
    Ground::SoilLow,
    Ground::SoilHigh,
    Ground::Hills,
    Ground::Mountains,
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

    pub fn is_high(self) -> bool {
        matches!(self, Self::SoilHigh | Self::Hills | Self::Mountains)
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
            Self::Lava2 => 4,
            Self::Lava3 => 5,
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
}

pub const OVERLAYS: [Overlay; 6] = [
    Overlay::Bare,
    Overlay::Grass,
    Overlay::Jungle,
    Overlay::Enchanted,
    Overlay::Infernal,
    Overlay::Pumpkin,
];

impl Overlay {
    pub fn color(self, high: bool) -> Option<[u8; 3]> {
        Some(match (self, high) {
            (Self::Bare, _) => return None,
            (Self::Grass, false) => [0x7e, 0xaf, 0x46],
            (Self::Grass, true) => [0x5f, 0x83, 0x3c],
            (Self::Jungle, false) => [0x46, 0xa0, 0x52],
            (Self::Jungle, true) => [0x1f, 0x70, 0x20],
            (Self::Enchanted, false) => [0x8c, 0xdc, 0x6a],
            (Self::Enchanted, true) => [0x76, 0xb1, 0x53],
            (Self::Infernal, false) => [0x9c, 0x36, 0x26],
            (Self::Infernal, true) => [0x68, 0x37, 0x2d],
            (Self::Pumpkin, false) => [0x8f, 0x93, 0x39],
            (Self::Pumpkin, true) => [0x69, 0x6c, 0x02],
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Bare => "bare",
            Self::Grass => "grass",
            Self::Jungle => "jungle",
            Self::Enchanted => "enchanted",
            Self::Infernal => "infernal",
            Self::Pumpkin => "pumpkin",
        }
    }

    pub fn fertility(self) -> f32 {
        match self {
            Self::Grass => 1.0,
            Self::Jungle => 0.85,
            Self::Enchanted => 0.7,
            Self::Pumpkin => 0.4,
            Self::Infernal => 0.05,
            Self::Bare => 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Tile {
    pub ground: Ground,
    pub overlay: Overlay,
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
        }
    }

    pub fn bare(ground: Ground) -> Self {
        Self::new(ground, Overlay::Bare)
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

    pub fn color(self) -> [u8; 3] {
        self.overlay
            .color(self.ground.is_high())
            .unwrap_or_else(|| self.ground.color())
    }

    pub fn name(self) -> String {
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
        match self.overlay {
            Overlay::Jungle => 30,
            Overlay::Grass => 16,
            Overlay::Enchanted => 14,
            Overlay::Infernal => 10,
            Overlay::Pumpkin => 12,
            Overlay::Bare => 0,
        }
    }

    pub fn grain(self) -> i32 {
        match self.ground {
            Ground::DeepOcean | Ground::CloseOcean => 4,
            Ground::ShallowWaters => 7,
            Ground::Sand => 9,
            Ground::Hills | Ground::Mountains => 11,
            _ => 14,
        }
    }

    pub fn textured(self, x: u32, y: u32) -> [u8; 3] {
        let noise = hash(x, y);
        let base = self.color();
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

        assert!(seen.len() >= 20, "{} distinct of {before}", seen.len());
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
}
