use econbox_tile::{Flora, Overlay};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Footprint {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

impl Footprint {
    pub const SINGLE: Self = Self::new(0, 0, 0, 0);
    pub const TREE: Self = Self::new(1, 1, 1, 0);
    pub const BIG_TREE: Self = Self::new(2, 2, 1, 0);

    pub const fn new(left: u32, right: u32, top: u32, bottom: u32) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    pub fn width(self) -> u32 {
        self.left + self.right + 1
    }

    pub fn height(self) -> u32 {
        self.top + self.bottom + 1
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vegetation {
    pub id: &'static str,
    pub flora: Flora,
    pub limit_per_zone: i32,
    pub chance: f32,
    pub limit_in_radius: u32,
    pub footprint: Footprint,
}

const fn tree(id: &'static str, limit_per_zone: i32, chance: f32) -> Vegetation {
    Vegetation {
        id,
        flora: Flora::Tree,
        limit_per_zone,
        chance,
        limit_in_radius: 0,
        footprint: Footprint::TREE,
    }
}

const fn plant(id: &'static str, limit_per_zone: i32) -> Vegetation {
    Vegetation {
        id,
        flora: Flora::Plant,
        limit_per_zone,
        chance: 0.5,
        limit_in_radius: 0,
        footprint: Footprint::SINGLE,
    }
}

const fn wide(vegetation: Vegetation, footprint: Footprint) -> Vegetation {
    Vegetation {
        footprint,
        ..vegetation
    }
}

const fn spaced(vegetation: Vegetation, limit_in_radius: u32) -> Vegetation {
    Vegetation {
        limit_in_radius,
        ..vegetation
    }
}

pub const FRUIT_BUSH: Vegetation = Vegetation {
    id: "fruit_bush",
    flora: Flora::Bush,
    limit_per_zone: 1,
    chance: 0.2,
    limit_in_radius: 0,
    footprint: Footprint::TREE,
};

pub const PALM_TREE: Vegetation = tree("palm_tree", 3, 0.1);
pub const CACTI_TREE: Vegetation = tree("cacti_tree", 3, 0.2);

const TREE_GREEN_1: Vegetation = tree("tree_green_1", 3, 0.5);
const TREE_GREEN_2: Vegetation = tree("tree_green_2", 3, 0.5);
const TREE_GREEN_3: Vegetation = tree("tree_green_3", 3, 0.5);
const SAVANNA_TREE_1: Vegetation = tree("savanna_tree_1", 3, 0.5);
const SAVANNA_TREE_2: Vegetation = tree("savanna_tree_2", 3, 0.5);
const SAVANNA_TREE_BIG_1: Vegetation =
    wide(tree("savanna_tree_big_1", 3, 0.5), Footprint::BIG_TREE);
const SAVANNA_TREE_BIG_2: Vegetation =
    wide(tree("savanna_tree_big_2", 3, 0.5), Footprint::BIG_TREE);
const JUNGLE_TREE: Vegetation = tree("jungle_tree", 8, 0.5);
const DESERT_TREE: Vegetation = tree("desert_tree", 1, 0.1);
const LEMON_TREE: Vegetation = tree("lemon_tree", 1, 0.1);
const PINE_TREE: Vegetation = tree("pine_tree", 3, 0.5);
const SWAMP_TREE: Vegetation = tree("swamp_tree", 2, 0.5);
const CRYSTAL_TREE: Vegetation = tree("crystal_tree", 1, 0.1);
const ENCHANTED_TREE: Vegetation = tree("enchanted_tree", 4, 0.5);
const CORRUPTED_TREE: Vegetation = tree("corrupted_tree", 4, 0.5);
const CORRUPTED_TREE_BIG: Vegetation =
    wide(tree("corrupted_tree_big", 4, 0.5), Footprint::BIG_TREE);
const INFERNAL_TREE: Vegetation = tree("infernal_tree", 3, 0.5);
const INFERNAL_TREE_BIG: Vegetation = wide(tree("infernal_tree_big", 3, 0.5), Footprint::BIG_TREE);
const INFERNAL_TREE_SMALL: Vegetation = wide(
    tree("infernal_tree_small", 3, 0.5),
    Footprint::new(0, 0, 1, 0),
);
const CANDY_TREE: Vegetation = tree("candy_tree", 1, 0.1);
const MUSHROOM_TREE: Vegetation = tree("mushroom_tree", 2, 0.5);
const BIRCH_TREE: Vegetation = tree("birch_tree", 3, 0.5);
const MAPLE_TREE: Vegetation = tree("maple_tree", 3, 0.5);
const ROCKLANDS_TREE: Vegetation = spaced(tree("rocklands_tree", 3, 0.5), 6);
const GARLIC_TREE: Vegetation = tree("garlic_tree", 3, 0.5);
const FLOWER_TREE_1: Vegetation = tree("flower_tree_1", 3, 0.5);
const FLOWER_TREE_2: Vegetation = tree("flower_tree_2", 3, 0.5);
const FLOWER_TREE_3: Vegetation = tree("flower_tree_3", 3, 0.5);
const CELESTIAL_TREE: Vegetation = spaced(
    wide(tree("celestial_tree", 1, 0.5), Footprint::new(2, 2, 2, 0)),
    30,
);
const CELESTIAL_TREE_SMALL: Vegetation = spaced(tree("celestial_tree_small", 15, 0.5), 6);
const CLOVER_TREE: Vegetation = tree("clover_tree", 3, 0.5);

const MUSHROOM_RED: Vegetation = wide(plant("mushroom_red", 9), Footprint::TREE);
const MUSHROOM_GREEN: Vegetation = wide(plant("mushroom_green", 9), Footprint::TREE);
const MUSHROOM_TEAL: Vegetation = wide(plant("mushroom_teal", 9), Footprint::TREE);
const MUSHROOM_WHITE: Vegetation = wide(plant("mushroom_white", 9), Footprint::TREE);
const MUSHROOM_YELLOW: Vegetation = wide(plant("mushroom_yellow", 9), Footprint::TREE);
const GREEN_HERB: Vegetation = plant("green_herb", 5);
const FLOWER: Vegetation = plant("flower", 5);
const SAVANNA_PLANT: Vegetation = plant("savanna_plant", 5);
const JUNGLE_PLANT: Vegetation = plant("jungle_plant", 6);
const JUNGLE_FLOWER: Vegetation = plant("jungle_flower", 5);
const DESERT_PLANT: Vegetation = plant("desert_plant", 3);
const SNOW_PLANT: Vegetation = plant("snow_plant", 4);
const SWAMP_PLANT: Vegetation = plant("swamp_plant", 4);
const SWAMP_PLANT_BIG: Vegetation = wide(plant("swamp_plant_big", 4), Footprint::TREE);
const CRYSTAL_PLANT: Vegetation = plant("crystal_plant", 2);
const CORRUPTED_PLANT: Vegetation = plant("corrupted_plant", 5);
const FLAME_FLOWER: Vegetation = plant("flame_flower", 5);
const CANDY_PLANT: Vegetation = wide(plant("candy_plant", 5), Footprint::TREE);
const BIRCH_PLANT: Vegetation = plant("birch_plant", 5);
const MAPLE_PLANT: Vegetation = plant("maple_plant", 5);
const ROCKLANDS_PLANT: Vegetation = plant("rocklands_plant", 5);
const GARLIC_PLANT: Vegetation = plant("garlic_plant", 5);
const FLOWER_PLANT: Vegetation = plant("flower_plant", 5);
const CELESTIAL_PLANT: Vegetation = plant("celestial_plant", 5);
const CLOVER_PLANT: Vegetation = plant("clover_plant", 5);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pot {
    pub entries: &'static [(Vegetation, u32)],
}

impl Pot {
    pub const EMPTY: Self = Self { entries: &[] };

    pub fn len(self) -> usize {
        self.entries.iter().map(|(_, rate)| *rate as usize).sum()
    }

    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    pub fn get(self, mut index: usize) -> Option<Vegetation> {
        for (vegetation, rate) in self.entries {
            if index < *rate as usize {
                return Some(*vegetation);
            }
            index -= *rate as usize;
        }

        None
    }
}

const MUSHROOMS_TWICE: [(Vegetation, u32); 5] = [
    (MUSHROOM_RED, 2),
    (MUSHROOM_GREEN, 2),
    (MUSHROOM_TEAL, 2),
    (MUSHROOM_WHITE, 2),
    (MUSHROOM_YELLOW, 2),
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Biome {
    pub id: &'static str,
    pub overlay: Option<Overlay>,
    pub generator_pot_amount: u32,
    pub generator_max_size: u32,
    pub special_biome: bool,
    pub grow_vegetation_auto: bool,
    pub trees: Pot,
    pub plants: Pot,
    pub bushes: Pot,
}

const fn biome(
    id: &'static str,
    overlay: Overlay,
    generator_pot_amount: u32,
    generator_max_size: u32,
    trees: &'static [(Vegetation, u32)],
    plants: &'static [(Vegetation, u32)],
    bushes: bool,
) -> Biome {
    Biome {
        id,
        overlay: Some(overlay),
        generator_pot_amount,
        generator_max_size,
        special_biome: false,
        grow_vegetation_auto: true,
        trees: Pot { entries: trees },
        plants: Pot { entries: plants },
        bushes: if bushes {
            Pot {
                entries: &[(FRUIT_BUSH, 1)],
            }
        } else {
            Pot::EMPTY
        },
    }
}

pub const BIOME_SAND: Biome = Biome {
    id: "biome_sand",
    overlay: None,
    generator_pot_amount: 0,
    generator_max_size: 0,
    special_biome: true,
    grow_vegetation_auto: true,
    trees: Pot::EMPTY,
    plants: Pot::EMPTY,
    bushes: Pot::EMPTY,
};

pub const BIOME_HILL: Biome = Biome {
    id: "biome_hill",
    ..BIOME_SAND
};

pub const BIOMES: [Biome; 20] = [
    biome(
        "biome_grass",
        Overlay::Grass,
        8,
        0,
        &[(TREE_GREEN_1, 1), (TREE_GREEN_2, 1), (TREE_GREEN_3, 1)],
        &[
            MUSHROOMS_TWICE[0],
            MUSHROOMS_TWICE[1],
            MUSHROOMS_TWICE[2],
            MUSHROOMS_TWICE[3],
            MUSHROOMS_TWICE[4],
            (GREEN_HERB, 4),
            (FLOWER, 4),
        ],
        true,
    ),
    biome(
        "biome_savanna",
        Overlay::Savanna,
        5,
        0,
        &[
            (SAVANNA_TREE_1, 5),
            (SAVANNA_TREE_2, 5),
            (SAVANNA_TREE_BIG_1, 1),
            (SAVANNA_TREE_BIG_2, 1),
        ],
        &[(SAVANNA_PLANT, 1)],
        true,
    ),
    biome(
        "biome_jungle",
        Overlay::Jungle,
        6,
        0,
        &[(JUNGLE_TREE, 1)],
        &[(JUNGLE_PLANT, 1), (JUNGLE_FLOWER, 1)],
        true,
    ),
    biome(
        "biome_desert",
        Overlay::Desert,
        4,
        0,
        &[(DESERT_TREE, 1)],
        &[(DESERT_PLANT, 1)],
        false,
    ),
    biome(
        "biome_lemon",
        Overlay::Lemon,
        3,
        0,
        &[(LEMON_TREE, 1)],
        &[
            (MUSHROOM_RED, 1),
            (MUSHROOM_GREEN, 1),
            (MUSHROOM_TEAL, 1),
            (MUSHROOM_WHITE, 1),
            (MUSHROOM_YELLOW, 1),
            (GREEN_HERB, 5),
            (FLOWER, 5),
        ],
        true,
    ),
    biome(
        "biome_permafrost",
        Overlay::Frost,
        3,
        0,
        &[(PINE_TREE, 1)],
        &[(SNOW_PLANT, 1)],
        false,
    ),
    biome(
        "biome_swamp",
        Overlay::Swamp,
        3,
        300,
        &[(SWAMP_TREE, 1)],
        &[(SWAMP_PLANT, 1), (SWAMP_PLANT_BIG, 1)],
        false,
    ),
    biome(
        "biome_crystal",
        Overlay::Crystal,
        2,
        0,
        &[(CRYSTAL_TREE, 1)],
        &[(CRYSTAL_PLANT, 1)],
        false,
    ),
    biome(
        "biome_enchanted",
        Overlay::Enchanted,
        3,
        0,
        &[(ENCHANTED_TREE, 1)],
        &[(FLOWER, 5)],
        true,
    ),
    biome(
        "biome_corrupted",
        Overlay::Blight,
        1,
        20,
        &[(CORRUPTED_TREE, 1), (CORRUPTED_TREE_BIG, 1)],
        &[(CORRUPTED_PLANT, 1)],
        false,
    ),
    biome(
        "biome_infernal",
        Overlay::Infernal,
        1,
        20,
        &[
            (INFERNAL_TREE, 1),
            (INFERNAL_TREE_BIG, 1),
            (INFERNAL_TREE_SMALL, 1),
        ],
        &[(FLAME_FLOWER, 1)],
        false,
    ),
    biome(
        "biome_candy",
        Overlay::Candy,
        1,
        30,
        &[(CANDY_TREE, 1)],
        &[(CANDY_PLANT, 1)],
        false,
    ),
    biome(
        "biome_mushroom",
        Overlay::Mushroom,
        1,
        300,
        &[(MUSHROOM_TREE, 1)],
        &[
            (FLOWER, 2),
            MUSHROOMS_TWICE[0],
            MUSHROOMS_TWICE[1],
            MUSHROOMS_TWICE[2],
            MUSHROOMS_TWICE[3],
            MUSHROOMS_TWICE[4],
        ],
        true,
    ),
    biome(
        "biome_birch",
        Overlay::Birch,
        4,
        0,
        &[(BIRCH_TREE, 1)],
        &[
            (BIRCH_PLANT, 1),
            (MUSHROOM_RED, 1),
            (MUSHROOM_GREEN, 1),
            (MUSHROOM_TEAL, 1),
            (MUSHROOM_WHITE, 1),
            (MUSHROOM_YELLOW, 1),
            (GREEN_HERB, 4),
            (FLOWER, 4),
        ],
        true,
    ),
    biome(
        "biome_maple",
        Overlay::Maple,
        5,
        0,
        &[(MAPLE_TREE, 1)],
        &[
            (MAPLE_PLANT, 1),
            (MUSHROOM_RED, 1),
            (MUSHROOM_GREEN, 1),
            (MUSHROOM_TEAL, 1),
            (MUSHROOM_WHITE, 1),
            (MUSHROOM_YELLOW, 1),
            (GREEN_HERB, 4),
            (FLOWER, 4),
        ],
        true,
    ),
    biome(
        "biome_rocklands",
        Overlay::Rockland,
        2,
        0,
        &[(ROCKLANDS_TREE, 1)],
        &[(ROCKLANDS_PLANT, 1)],
        false,
    ),
    biome(
        "biome_garlic",
        Overlay::Garlic,
        2,
        0,
        &[(GARLIC_TREE, 1)],
        &[(GARLIC_PLANT, 1)],
        true,
    ),
    biome(
        "biome_flower",
        Overlay::Flower,
        3,
        0,
        &[(FLOWER_TREE_1, 1), (FLOWER_TREE_2, 1), (FLOWER_TREE_3, 1)],
        &[
            (FLOWER_PLANT, 3),
            (MUSHROOM_RED, 1),
            (MUSHROOM_GREEN, 1),
            (MUSHROOM_TEAL, 1),
            (MUSHROOM_WHITE, 1),
            (MUSHROOM_YELLOW, 1),
            (GREEN_HERB, 5),
            (FLOWER, 20),
        ],
        true,
    ),
    biome(
        "biome_celestial",
        Overlay::Starlight,
        1,
        0,
        &[(CELESTIAL_TREE, 10), (CELESTIAL_TREE_SMALL, 1)],
        &[(CELESTIAL_PLANT, 1)],
        false,
    ),
    biome(
        "biome_clover",
        Overlay::Clover,
        2,
        0,
        &[(CLOVER_TREE, 1)],
        &[(CLOVER_PLANT, 1)],
        true,
    ),
];

pub fn biome_of(overlay: Overlay) -> Option<&'static Biome> {
    BIOMES.iter().find(|biome| biome.overlay == Some(overlay))
}

pub fn pool() -> Vec<&'static Biome> {
    BIOMES
        .iter()
        .flat_map(|biome| std::iter::repeat_n(biome, biome.generator_pot_amount as usize))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pool_weighs_each_biome_by_its_pot_amount() {
        let pool = pool();
        let grass = pool
            .iter()
            .filter(|biome| biome.id == "biome_grass")
            .count();
        let candy = pool
            .iter()
            .filter(|biome| biome.id == "biome_candy")
            .count();

        assert_eq!(pool.len(), 60);
        assert_eq!(grass, 8);
        assert_eq!(candy, 1);
        assert_eq!(pool[0].id, "biome_grass");
    }

    #[test]
    fn the_rare_biomes_stay_small() {
        let size = |id: &str| {
            BIOMES
                .iter()
                .find(|biome| biome.id == id)
                .map(|biome| biome.generator_max_size)
        };

        assert_eq!(size("biome_corrupted"), Some(20));
        assert_eq!(size("biome_infernal"), Some(20));
        assert_eq!(size("biome_candy"), Some(30));
        assert_eq!(size("biome_swamp"), Some(300));
        assert_eq!(size("biome_mushroom"), Some(300));
        assert_eq!(size("biome_grass"), Some(0));
    }

    #[test]
    fn every_generated_biome_has_an_overlay_of_its_own() {
        let mut overlays: Vec<Overlay> = BIOMES.iter().filter_map(|biome| biome.overlay).collect();
        overlays.sort_unstable();
        overlays.dedup();

        assert_eq!(overlays.len(), BIOMES.len());
        assert_eq!(
            biome_of(Overlay::Blight).map(|b| b.id),
            Some("biome_corrupted")
        );
        assert_eq!(biome_of(Overlay::Void), None);
    }

    #[test]
    fn a_pot_draws_entries_by_their_rate() {
        let grass = biome_of(Overlay::Grass).expect("grass");

        assert_eq!(grass.plants.len(), 18);
        assert_eq!(grass.plants.get(0).map(|v| v.id), Some("mushroom_red"));
        assert_eq!(grass.plants.get(10).map(|v| v.id), Some("green_herb"));
        assert_eq!(grass.plants.get(17).map(|v| v.id), Some("flower"));
        assert_eq!(grass.plants.get(18), None);
        assert_eq!(grass.bushes.get(0), Some(FRUIT_BUSH));
        assert!(biome_of(Overlay::Desert).expect("desert").bushes.is_empty());
    }
}
