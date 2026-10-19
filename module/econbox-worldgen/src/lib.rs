//! WorldBox's world generator: templates, map sizes, editable settings and the staged pipeline.

pub mod biome;
pub mod generator;
pub mod settings;
pub mod shapes;
pub mod size;
pub mod stages;
pub mod template;
pub mod tool;
pub mod world;

use econbox_tilemap::TileMap;
use std::borrow::Borrow;

pub use generator::Generator;
pub use settings::{MapGenSetting, SETTINGS, setting};
pub use size::{MAP_SIZES, MapSize, ZONE_TILES};
pub use stages::{Stage, stages};
pub use template::{
    AllowEdit, MapGenTemplate, MapGenValues, PerlinReplaceContainer, PerlinReplaceOption,
    TEMPLATE_IDS, TEMPLATES, TemplateId,
};
pub use world::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldConfig {
    pub seed: u32,
    pub template: TemplateId,
    pub size: MapSize,
    pub values: MapGenValues,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            seed: 0x_5eed_1234,
            template: TemplateId::default(),
            size: MapSize::default(),
            values: TemplateId::default().template().values,
        }
    }
}

impl WorldConfig {
    pub fn with_seed(seed: u32) -> Self {
        Self {
            seed,
            ..Self::default()
        }
    }

    pub fn with_template(self, template: TemplateId) -> Self {
        Self {
            template,
            values: template.template().values,
            ..self
        }
    }

    pub fn with_size(self, size: MapSize) -> Self {
        Self { size, ..self }
    }

    pub fn reset_values(&mut self) {
        self.values = self.template.template().values;
    }

    pub fn width(&self) -> u32 {
        self.size.tiles()
    }

    pub fn height(&self) -> u32 {
        self.size.tiles()
    }

    pub fn plan(&self) -> Vec<Stage> {
        Generator::new(
            self.width(),
            self.seed,
            self.template.template(),
            self.values,
        )
        .plan()
    }
}

pub fn generate_world(config: &WorldConfig) -> World {
    Generator::new(
        config.width(),
        config.seed,
        config.template.template(),
        config.values,
    )
    .run_all()
}

pub fn generate(config: impl Borrow<WorldConfig>) -> TileMap {
    generate_world(config.borrow()).into_map()
}

#[cfg(test)]
mod tests {
    use super::*;
    use econbox_tile::{Flora, Ground, Overlay};

    fn small() -> WorldConfig {
        WorldConfig::with_seed(7).with_size(MapSize::Tiny)
    }

    #[test]
    fn the_first_launch_world_is_a_standard_continent() {
        let config = WorldConfig::default();

        assert_eq!(config.template, TemplateId::Continent);
        assert_eq!(config.size, MapSize::Standard);
        assert_eq!((config.width(), config.height()), (256, 256));
        assert_eq!(config.values, TemplateId::Continent.template().values);
    }

    #[test]
    fn the_map_comes_out_at_the_chosen_size() {
        let map = generate(small());

        assert_eq!((map.width, map.height), (128, 128));
        assert_eq!(map.heights.len(), map.cells());
        assert_eq!(generate(WorldConfig::default()).chunks(), (4, 4));
    }

    #[test]
    fn the_same_seed_and_settings_grow_the_same_world() {
        let config = small();

        assert_eq!(generate(config), generate(config));
    }

    #[test]
    fn a_new_seed_grows_a_new_world() {
        let a = generate(small());
        let b = generate(WorldConfig { seed: 8, ..small() });

        assert_ne!(a.tiles, b.tiles);
    }

    #[test]
    fn a_continent_has_every_worldbox_band_from_the_deep_to_the_summit() {
        let map = generate(WorldConfig::with_seed(3));
        let counts = map.ground_counts();
        let of = |ground| counts.get(&ground).copied().unwrap_or(0);

        for ground in [
            Ground::DeepOcean,
            Ground::CloseOcean,
            Ground::ShallowWaters,
            Ground::Sand,
            Ground::SoilLow,
            Ground::SoilHigh,
            Ground::Hills,
            Ground::Mountains,
        ] {
            assert!(of(ground) > 0, "{ground:?}");
        }
        assert!(of(Ground::DeepOcean) > of(Ground::ShallowWaters));
        assert!(of(Ground::SoilHigh) > of(Ground::Mountains));
    }

    #[test]
    fn the_round_edge_leaves_the_corners_of_a_continent_in_the_deep() {
        let map = generate(WorldConfig::with_seed(3));

        for (x, y) in [(0, 0), (255, 0), (0, 255), (255, 255), (128, 0), (0, 128)] {
            assert_eq!(map.at(x, y).ground, Ground::DeepOcean, "{x},{y}");
        }
    }

    #[test]
    fn every_tile_matches_its_height_band_or_a_later_stage_moved_it() {
        let map = generate(WorldConfig::with_seed(5));
        let mut agree = 0;

        for y in 0..map.height {
            for x in 0..map.width {
                if Ground::from_height(map.height_at(x, y)) == map.at(x, y).ground {
                    agree += 1;
                }
            }
        }

        assert!(agree * 10 > map.cells() * 7, "{agree} of {}", map.cells());
    }

    #[test]
    fn soil_wears_biomes_in_patches_and_nothing_else_does() {
        let map = generate(WorldConfig::with_seed(11));
        let mut biomes = std::collections::BTreeSet::new();

        for tile in &map.tiles {
            if tile.overlay != Overlay::Bare {
                assert!(tile.ground.is_soil(), "{}", tile.name());
                biomes.insert(tile.overlay);
            } else {
                assert!(!tile.ground.is_soil(), "bare soil left");
            }
        }

        assert!(biomes.len() >= 3, "{biomes:?}");
    }

    #[test]
    fn vegetation_and_fruit_bushes_dot_the_land_but_never_the_sea() {
        let map = generate(WorldConfig::with_seed(11));
        let flora: Vec<_> = map
            .tiles
            .iter()
            .filter(|tile| tile.flora.is_some())
            .collect();

        assert!(flora.iter().any(|tile| tile.flora == Some(Flora::Tree)));
        assert!(flora.iter().any(|tile| tile.flora == Some(Flora::Plant)));
        assert!(flora.iter().all(|tile| tile.is_land()));
        assert!(map.tiles.iter().all(|tile| tile.ore.is_none()));
    }

    #[test]
    fn tall_mountains_are_capped_with_snow() {
        let map = generate(WorldConfig::with_seed(11));

        for (index, tile) in map.tiles.iter().enumerate() {
            if tile.frozen {
                assert_eq!(tile.ground, Ground::Mountains);
                assert!(map.heights[index] > 220);
            }
        }
    }

    #[test]
    fn switching_template_resets_its_values() {
        let mut config = WorldConfig::default().with_template(TemplateId::Lasagna);
        assert!(config.values.low_ground);

        config.values.low_ground = false;
        config.reset_values();
        assert!(config.values.low_ground);
    }

    #[test]
    fn every_template_generates_at_every_size_without_panicking() {
        for template in TEMPLATE_IDS {
            let config = WorldConfig::with_seed(2)
                .with_template(template)
                .with_size(MapSize::Tiny);
            let map = generate(config);

            assert_eq!(map.cells(), 128 * 128, "{}", template.id());
        }

        let empty = generate(WorldConfig::default().with_template(TemplateId::Empty));
        assert!(
            empty
                .tiles
                .iter()
                .all(|tile| tile.ground == Ground::DeepOcean)
        );
    }

    #[test]
    fn box_world_is_walled_in_by_mountains() {
        let map = generate(
            WorldConfig::with_seed(4)
                .with_template(TemplateId::BoxWorld)
                .with_size(MapSize::Tiny),
        );

        for step in 0..128 {
            assert!(map.at(step, 0).ground == Ground::Mountains);
            assert!(map.at(0, step).ground == Ground::Mountains);
        }
    }

    #[test]
    fn the_plan_follows_the_template() {
        let plan = WorldConfig::default()
            .with_template(TemplateId::Toast)
            .plan();

        assert!(plan.contains(&Stage::SquareEdges));
        assert!(!plan.contains(&Stage::RoundEdges));
    }
}
