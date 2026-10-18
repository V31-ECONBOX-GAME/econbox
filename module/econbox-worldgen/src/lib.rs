//! Bands the scalar fields into a WorldBox shaped tile map.

use econbox_field::{Field, HeightConfig, height, moisture, temperature};
use econbox_tile::{Ground, Overlay, Tile};
use econbox_tilemap::TileMap;

#[derive(Debug, Clone, Copy)]
pub struct WorldConfig {
    pub seed: u32,
    pub width: u32,
    pub height: u32,
    pub water_share: f32,
    pub equator_bias: f32,
    pub moisture_scale: f32,
    pub shape: HeightConfig,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            seed: 0x_5eed_1234,
            width: 448,
            height: 448,
            water_share: 0.68,
            equator_bias: 1.4,
            moisture_scale: 4.5,
            shape: HeightConfig::default(),
        }
    }
}

impl WorldConfig {
    pub fn with_seed(seed: u32) -> Self {
        Self {
            seed,
            shape: HeightConfig {
                seed,
                ..HeightConfig::default()
            },
            ..Self::default()
        }
    }
}

const WATER_BANDS: [(f32, Ground); 3] = [
    (0.37, Ground::DeepOcean),
    (0.68, Ground::CloseOcean),
    (1.00, Ground::ShallowWaters),
];

const LAND_BANDS: [(f32, Ground); 6] = [
    (0.13, Ground::Sand),
    (0.40, Ground::SoilLow),
    (0.79, Ground::SoilHigh),
    (0.93, Ground::Hills),
    (0.985, Ground::Mountains),
    (1.00, Ground::Summit),
];

pub fn generate(config: WorldConfig) -> TileMap {
    let shape = HeightConfig {
        seed: config.seed,
        ..config.shape
    };
    let relief = height(config.width, config.height, shape);
    let warmth = temperature(&relief, config.equator_bias);
    let damp = moisture(
        config.width,
        config.height,
        config.seed,
        config.moisture_scale,
    );

    let sea = relief.quantile(config.water_share);
    let water_cuts = cuts(&relief, 0.0, sea, WATER_BANDS.map(|(share, _)| share));
    let land_cuts = cuts(&relief, sea, 1.0, LAND_BANDS.map(|(share, _)| share));

    let mut map = TileMap::filled(config.width, config.height, Tile::default());

    for y in 0..config.height {
        for x in 0..config.width {
            let elevation = relief.at(x, y);
            let ground = if elevation < sea {
                band(
                    elevation,
                    &water_cuts,
                    WATER_BANDS.map(|(_, ground)| ground),
                )
            } else {
                band(elevation, &land_cuts, LAND_BANDS.map(|(_, ground)| ground))
            };

            let overlay = if matches!(ground, Ground::SoilLow | Ground::SoilHigh) {
                cover(warmth.at(x, y), damp.at(x, y))
            } else {
                Overlay::Bare
            };

            map.set(x, y, Tile::new(ground, overlay));
        }
    }

    map
}

fn cuts<const N: usize>(field: &Field, low: f32, high: f32, shares: [f32; N]) -> [f32; N] {
    let base = field.share_below(low);
    let span = field.share_below(high) - base;

    shares.map(|share| field.quantile(base + span * share))
}

fn band<const N: usize>(value: f32, cuts: &[f32; N], grounds: [Ground; N]) -> Ground {
    for (cut, ground) in cuts.iter().zip(grounds) {
        if value < *cut {
            return ground;
        }
    }

    grounds[N - 1]
}

pub fn cover(warmth: f32, damp: f32) -> Overlay {
    if warmth > 0.66 && damp > 0.60 {
        return Overlay::Jungle;
    }
    if warmth > 0.70 && damp < 0.17 {
        return Overlay::Bare;
    }

    Overlay::Grass
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small() -> WorldConfig {
        WorldConfig {
            width: 224,
            height: 224,
            ..WorldConfig::default()
        }
    }

    #[test]
    fn the_map_is_the_size_worldbox_uses() {
        let config = WorldConfig::default();

        assert_eq!(config.width, 448);
        assert_eq!(config.height, 448);
        assert_eq!(generate(config).chunks(), (7, 7));
    }

    #[test]
    fn the_sea_covers_the_share_the_config_asks_for() {
        for share in [0.55_f32, 0.68, 0.80] {
            let map = generate(WorldConfig {
                water_share: share,
                ..small()
            });

            assert!(
                (map.water_share() - share).abs() < 0.06,
                "{share} -> {}",
                map.water_share()
            );
        }
    }

    #[test]
    fn the_border_of_the_map_is_open_sea() {
        let map = generate(small());
        let mut deep = 0;
        let mut edge = 0;

        for (x, y) in (0..map.width)
            .flat_map(|x| [(x, 0), (x, map.height - 1)])
            .chain((0..map.height).flat_map(|y| [(0, y), (map.width - 1, y)]))
        {
            assert!(map.at(x, y).is_water(), "land at the edge {x},{y}");
            deep += usize::from(map.at(x, y).ground == Ground::DeepOcean);
            edge += 1;
        }

        assert!(deep * 100 / edge > 80, "only {deep}/{edge} deep");
    }

    #[test]
    fn every_island_is_ringed_by_shallows_then_sand() {
        let map = generate(small());
        let counts = map.ground_counts();

        for ground in [
            Ground::DeepOcean,
            Ground::CloseOcean,
            Ground::ShallowWaters,
            Ground::Sand,
            Ground::SoilLow,
            Ground::SoilHigh,
            Ground::Hills,
            Ground::Mountains,
            Ground::Summit,
        ] {
            assert!(counts.get(&ground).copied().unwrap_or(0) > 0, "{ground:?}");
        }
    }

    #[test]
    fn the_deep_is_bigger_than_the_shallows_and_peaks_are_rarest() {
        let map = generate(small());
        let counts = map.ground_counts();
        let of = |ground| counts.get(&ground).copied().unwrap_or(0);

        assert!(of(Ground::DeepOcean) > of(Ground::ShallowWaters));
        assert!(of(Ground::SoilHigh) > of(Ground::Mountains));
        assert!(of(Ground::Hills) > of(Ground::Mountains));
        assert!(of(Ground::Mountains) > of(Ground::Summit));
    }

    #[test]
    fn the_same_seed_grows_the_same_world() {
        assert_eq!(generate(small()), generate(small()));
    }

    #[test]
    fn a_new_seed_grows_a_new_world() {
        let a = generate(WorldConfig::with_seed(1));
        let b = generate(WorldConfig::with_seed(2));

        assert_ne!(a.tiles, b.tiles);
        assert!((a.water_share() - b.water_share()).abs() < 0.12);
    }

    #[test]
    fn only_soil_and_sand_carry_a_biome() {
        let map = generate(small());

        for tile in &map.tiles {
            if tile.overlay != Overlay::Bare {
                assert!(tile.ground.takes_overlay(), "{}", tile.name());
            }
        }
    }

    #[test]
    fn jungle_wants_it_hot_and_wet() {
        assert_eq!(cover(0.9, 0.9), Overlay::Jungle);
        assert_eq!(cover(0.9, 0.1), Overlay::Bare);
        assert_eq!(cover(0.5, 0.5), Overlay::Grass);
        assert_eq!(cover(0.05, 0.9), Overlay::Grass);
    }

    #[test]
    fn a_beach_is_plain_sand_the_way_worldbox_leaves_it() {
        let map = generate(small());

        for tile in &map.tiles {
            if tile.ground == Ground::Sand {
                assert_eq!(tile.overlay, Overlay::Bare, "{}", tile.name());
            }
        }
    }

    #[test]
    fn bare_soil_is_as_rare_as_it_is_in_a_real_save() {
        let map = generate(small());
        let bare = map
            .tiles
            .iter()
            .filter(|tile| {
                matches!(tile.ground, Ground::SoilLow | Ground::SoilHigh)
                    && tile.overlay == Overlay::Bare
            })
            .count();
        let share = bare as f32 / map.cells() as f32;

        assert!(share < 0.03, "bare soil {:.2}%", share * 100.0);
    }
}
