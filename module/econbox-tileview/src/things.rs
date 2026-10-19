use crate::thing_art::Shape;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    Tree,
    Plant,
    Bush,
    Mineral,
    Wall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wind {
    Still,
    Leaves,
    Celestial,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Variant {
    pub width: u16,
    pub height: u16,
    pub pivot_x: f32,
    pub pivot_y: f32,
    pub frames: u16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Kind {
    pub id: &'static str,
    pub family: Family,
    pub shape: Shape,
    pub variants: &'static [Variant],
    pub bare: &'static [Variant],
    pub mini: (u8, u8),
    pub shadow: Option<[f32; 3]>,
    pub fundament: [u8; 4],
    pub wind: Wind,
    pub flip: bool,
    pub light: Option<(f32, f32)>,
    pub sparkle: bool,
    pub speed: f32,
}

const fn v(width: u16, height: u16, pivot_x: f32, pivot_y: f32, frames: u16) -> Variant {
    Variant {
        width,
        height,
        pivot_x,
        pivot_y,
        frames,
    }
}

pub static BIRCH_PLANT: Kind = Kind {
    id: "birch_plant",
    family: Family::Plant,
    shape: Shape::Sprig,
    variants: &[
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
    ],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [0, 0, 0, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static BIRCH_TREE: Kind = Kind {
    id: "birch_tree",
    family: Family::Tree,
    shape: Shape::Round,
    variants: &[
        v(21, 26, 10.5, 1.083, 1),
        v(14, 23, 7.0, 1.0, 1),
        v(13, 20, 6.5, 1.0, 1),
        v(21, 23, 10.5, 1.0, 1),
        v(16, 23, 8.0, 1.0, 1),
        v(16, 23, 8.0, 1.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static CACTI_TREE: Kind = Kind {
    id: "cacti_tree",
    family: Family::Tree,
    shape: Shape::Cactus,
    variants: &[
        v(6, 15, 3.0, 0.0, 1),
        v(10, 14, 5.0, 0.0, 1),
        v(6, 9, 3.0, 0.0, 1),
        v(14, 20, 7.0, 0.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static CANDY_PLANT: Kind = Kind {
    id: "candy_plant",
    family: Family::Plant,
    shape: Shape::Sprig,
    variants: &[
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
    ],
    bare: &[],
    mini: (1, 1),
    shadow: Some([0.19, 0.03, 0.09]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static CANDY_TREE: Kind = Kind {
    id: "candy_tree",
    family: Family::Tree,
    shape: Shape::Round,
    variants: &[
        v(10, 10, 5.0, 0.0, 1),
        v(7, 10, 3.5, 0.0, 1),
        v(13, 14, 6.5, 1.0, 1),
        v(12, 13, 6.0, 0.0, 1),
        v(9, 18, 4.5, 1.2, 1),
        v(19, 19, 9.5, 1.188, 1),
        v(21, 22, 10.5, 1.048, 1),
        v(15, 22, 7.5, 1.048, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static CELESTIAL_PLANT: Kind = Kind {
    id: "celestial_plant",
    family: Family::Plant,
    shape: Shape::Sprig,
    variants: &[
        v(5, 5, 2.5, 0.0, 1),
        v(5, 5, 2.5, 0.0, 1),
        v(3, 4, 1.5, 0.0, 1),
        v(3, 4, 1.5, 0.0, 1),
        v(3, 4, 1.5, 0.0, 1),
        v(3, 4, 1.5, 0.0, 1),
        v(3, 4, 1.5, 0.0, 1),
    ],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [0, 0, 0, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static CELESTIAL_TREE: Kind = Kind {
    id: "celestial_tree",
    family: Family::Tree,
    shape: Shape::Glow,
    variants: &[
        v(157, 133, 78.5, 2.66, 12),
        v(164, 124, 82.0, 2.48, 11),
        v(111, 122, 55.5, 2.44, 17),
        v(110, 128, 55.0, 2.56, 18),
        v(114, 130, 57.0, 2.6, 16),
    ],
    bare: &[],
    mini: (5, 3),
    shadow: Some([0.5, 0.03, 0.05]),
    fundament: [2, 2, 2, 0],
    wind: Wind::Celestial,
    flip: true,
    light: Some((1.0, 0.0)),
    sparkle: false,
    speed: 6.0,
};

pub static CELESTIAL_TREE_SMALL: Kind = Kind {
    id: "celestial_tree_small",
    family: Family::Tree,
    shape: Shape::Glow,
    variants: &[
        v(51, 52, 25.5, 1.04, 12),
        v(51, 52, 25.5, 1.04, 12),
        v(51, 52, 25.5, 1.04, 12),
        v(51, 52, 25.5, 1.04, 12),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Still,
    flip: true,
    light: Some((0.2, 0.0)),
    sparkle: false,
    speed: 6.0,
};

pub static CLOVER_PLANT: Kind = Kind {
    id: "clover_plant",
    family: Family::Plant,
    shape: Shape::Sprig,
    variants: &[
        v(5, 5, 2.5, 0.0, 1),
        v(5, 5, 2.5, 0.0, 1),
        v(3, 4, 1.5, 0.0, 1),
    ],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [0, 0, 0, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static CLOVER_TREE: Kind = Kind {
    id: "clover_tree",
    family: Family::Tree,
    shape: Shape::Round,
    variants: &[
        v(16, 24, 8.0, 1.0, 1),
        v(14, 23, 7.0, 1.0, 1),
        v(10, 20, 5.0, 1.0, 1),
        v(16, 23, 8.0, 1.0, 1),
        v(16, 23, 8.0, 1.0, 1),
        v(16, 23, 8.0, 1.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static CORRUPTED_PLANT: Kind = Kind {
    id: "corrupted_plant",
    family: Family::Plant,
    shape: Shape::Sprig,
    variants: &[v(6, 6, 3.0, 0.0, 1), v(6, 6, 3.0, 0.0, 1)],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [0, 0, 0, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static CORRUPTED_TREE: Kind = Kind {
    id: "corrupted_tree",
    family: Family::Tree,
    shape: Shape::Thorn,
    variants: &[
        v(15, 18, 7.5, 0.75, 1),
        v(12, 14, 6.0, 0.0, 1),
        v(8, 12, 4.0, 0.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static CORRUPTED_TREE_BIG: Kind = Kind {
    id: "corrupted_tree_big",
    family: Family::Tree,
    shape: Shape::Thorn,
    variants: &[
        v(18, 23, 9.0, 0.0, 1),
        v(21, 25, 10.5, 1.042, 1),
        v(17, 21, 8.5, 0.875, 1),
    ],
    bare: &[],
    mini: (5, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [2, 2, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static CRYSTAL_PLANT: Kind = Kind {
    id: "crystal_plant",
    family: Family::Plant,
    shape: Shape::Sprig,
    variants: &[
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
    ],
    bare: &[],
    mini: (1, 1),
    shadow: Some([0.19, 0.03, 0.09]),
    fundament: [0, 0, 0, 0],
    wind: Wind::Still,
    flip: true,
    light: None,
    sparkle: true,
    speed: 6.0,
};

pub static CRYSTAL_TREE: Kind = Kind {
    id: "crystal_tree",
    family: Family::Tree,
    shape: Shape::Shard,
    variants: &[
        v(10, 10, 5.0, 0.0, 1),
        v(10, 10, 5.0, 0.0, 1),
        v(13, 14, 6.5, 1.0, 1),
        v(12, 14, 6.0, 0.0, 1),
        v(19, 19, 9.5, 1.267, 1),
        v(17, 18, 8.5, 1.125, 1),
        v(19, 21, 9.5, 1.0, 1),
        v(16, 21, 8.0, 1.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Still,
    flip: true,
    light: Some((0.1, 0.0)),
    sparkle: true,
    speed: 6.0,
};

pub static DESERT_PLANT: Kind = Kind {
    id: "desert_plant",
    family: Family::Plant,
    shape: Shape::Sprig,
    variants: &[
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
    ],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [0, 0, 0, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static DESERT_TREE: Kind = Kind {
    id: "desert_tree",
    family: Family::Tree,
    shape: Shape::Palm,
    variants: &[
        v(15, 17, 8.4, 0.0, 1),
        v(16, 16, 8.0, 0.0, 1),
        v(11, 10, 5.5, 0.0, 1),
        v(15, 22, 5.25, 1.1, 1),
        v(12, 10, 6.0, 0.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static ENCHANTED_TREE: Kind = Kind {
    id: "enchanted_tree",
    family: Family::Tree,
    shape: Shape::Round,
    variants: &[
        v(10, 10, 5.0, 0.0, 1),
        v(10, 9, 5.0, 0.0, 1),
        v(13, 14, 6.5, 1.0, 1),
        v(12, 13, 6.0, 0.0, 1),
        v(21, 20, 10.5, 1.333, 1),
        v(19, 19, 9.5, 1.188, 1),
        v(21, 22, 10.5, 1.048, 1),
        v(18, 22, 9.0, 1.048, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.03, 0.12]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: Some((0.2, 2.0)),
    sparkle: false,
    speed: 6.0,
};

pub static FLAME_FLOWER: Kind = Kind {
    id: "flame_flower",
    family: Family::Plant,
    shape: Shape::Blossom,
    variants: &[v(5, 5, 2.5, 0.0, 1), v(5, 5, 2.5, 0.0, 1)],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [0, 0, 0, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static FLOWER: Kind = Kind {
    id: "flower",
    family: Family::Plant,
    shape: Shape::Blossom,
    variants: &[
        v(5, 5, 2.5, 0.0, 1),
        v(5, 5, 2.5, 0.0, 1),
        v(3, 4, 1.5, 0.0, 1),
        v(5, 5, 2.5, 0.0, 1),
        v(5, 5, 2.5, 0.0, 1),
        v(5, 5, 2.5, 0.0, 1),
    ],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [0, 0, 0, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static FLOWER_PLANT: Kind = Kind {
    id: "flower_plant",
    family: Family::Plant,
    shape: Shape::Blossom,
    variants: &[
        v(5, 5, 2.5, 0.0, 1),
        v(5, 5, 2.5, 0.0, 1),
        v(3, 4, 1.5, 0.0, 1),
        v(5, 5, 2.5, 0.0, 1),
        v(5, 5, 2.5, 0.0, 1),
        v(5, 5, 2.5, 0.0, 1),
    ],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [0, 0, 0, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static FLOWER_TREE_1: Kind = Kind {
    id: "flower_tree_1",
    family: Family::Tree,
    shape: Shape::Round,
    variants: &[v(16, 24, 8.0, 1.0, 1), v(14, 23, 7.0, 1.0, 1)],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static FLOWER_TREE_2: Kind = Kind {
    id: "flower_tree_2",
    family: Family::Tree,
    shape: Shape::Round,
    variants: &[v(16, 20, 8.0, 1.0, 1), v(16, 23, 8.0, 1.0, 1)],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static FLOWER_TREE_3: Kind = Kind {
    id: "flower_tree_3",
    family: Family::Tree,
    shape: Shape::Round,
    variants: &[v(16, 24, 8.0, 1.0, 1), v(16, 24, 8.0, 1.0, 1)],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static FRUIT_BUSH: Kind = Kind {
    id: "fruit_bush",
    family: Family::Bush,
    shape: Shape::Bush,
    variants: &[v(10, 6, 5.0, 0.0, 1), v(10, 6, 5.0, 0.0, 1)],
    bare: &[v(10, 6, 5.0, 0.0, 1), v(10, 6, 5.0, 0.0, 1)],
    mini: (3, 2),
    shadow: Some([0.19, 0.03, 0.09]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static GARLIC_PLANT: Kind = Kind {
    id: "garlic_plant",
    family: Family::Plant,
    shape: Shape::Sprig,
    variants: &[
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
    ],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [0, 0, 0, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static GARLIC_TREE: Kind = Kind {
    id: "garlic_tree",
    family: Family::Tree,
    shape: Shape::Round,
    variants: &[
        v(16, 24, 8.0, 1.0, 1),
        v(14, 23, 7.0, 1.0, 1),
        v(10, 20, 5.0, 1.0, 1),
        v(16, 23, 8.0, 1.0, 1),
        v(16, 23, 8.0, 1.0, 1),
        v(16, 23, 8.0, 1.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static GREEN_HERB: Kind = Kind {
    id: "green_herb",
    family: Family::Plant,
    shape: Shape::Sprig,
    variants: &[
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
    ],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [0, 0, 0, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static INFERNAL_TREE: Kind = Kind {
    id: "infernal_tree",
    family: Family::Tree,
    shape: Shape::Thorn,
    variants: &[
        v(13, 17, 6.5, 0.0, 1),
        v(9, 12, 4.5, 0.0, 1),
        v(15, 17, 7.5, 0.0, 1),
        v(12, 19, 6.0, 0.0, 1),
        v(10, 13, 5.0, 0.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.1, 0.31, 0.33]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: Some((0.05, 0.0)),
    sparkle: false,
    speed: 6.0,
};

pub static INFERNAL_TREE_BIG: Kind = Kind {
    id: "infernal_tree_big",
    family: Family::Tree,
    shape: Shape::Thorn,
    variants: &[v(17, 22, 8.5, 0.0, 1)],
    bare: &[],
    mini: (2, 2),
    shadow: Some([0.37, 0.16, 0.2]),
    fundament: [2, 2, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: Some((0.1, 0.0)),
    sparkle: false,
    speed: 6.0,
};

pub static INFERNAL_TREE_SMALL: Kind = Kind {
    id: "infernal_tree_small",
    family: Family::Tree,
    shape: Shape::Thorn,
    variants: &[v(4, 6, 2.0, 0.0, 1), v(5, 7, 2.5, 0.0, 1)],
    bare: &[],
    mini: (1, 2),
    shadow: Some([0.5, 0.31, 0.33]),
    fundament: [0, 0, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: Some((0.05, 0.0)),
    sparkle: false,
    speed: 6.0,
};

pub static JUNGLE_FLOWER: Kind = Kind {
    id: "jungle_flower",
    family: Family::Plant,
    shape: Shape::Blossom,
    variants: &[v(6, 6, 3.0, 0.0, 1), v(6, 6, 3.0, 0.0, 1)],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [0, 0, 0, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static JUNGLE_PLANT: Kind = Kind {
    id: "jungle_plant",
    family: Family::Plant,
    shape: Shape::Sprig,
    variants: &[
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
    ],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [0, 0, 0, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static JUNGLE_TREE: Kind = Kind {
    id: "jungle_tree",
    family: Family::Tree,
    shape: Shape::Round,
    variants: &[
        v(8, 11, 4.0, 0.0, 1),
        v(8, 11, 4.0, 0.0, 1),
        v(11, 16, 6.0, 1.143, 1),
        v(10, 15, 5.0, 1.154, 1),
        v(14, 18, 8.0, 1.2, 1),
        v(12, 18, 6.0, 1.125, 1),
        v(16, 21, 8.0, 1.0, 1),
        v(14, 23, 7.0, 1.095, 1),
        v(16, 23, 8.0, 1.095, 1),
        v(14, 23, 7.0, 1.095, 1),
        v(12, 19, 6.0, 1.118, 1),
        v(12, 18, 6.0, 1.125, 1),
        v(9, 15, 4.0, 1.154, 1),
        v(11, 14, 5.5, 0.0, 1),
        v(6, 12, 3.0, 0.0, 1),
        v(8, 12, 4.0, 0.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static LEMON_TREE: Kind = Kind {
    id: "lemon_tree",
    family: Family::Tree,
    shape: Shape::Round,
    variants: &[
        v(8, 10, 4.0, 0.0, 1),
        v(10, 10, 5.0, 0.0, 1),
        v(8, 12, 4.0, 0.0, 1),
        v(11, 12, 5.5, 0.0, 1),
        v(13, 19, 6.5, 0.0, 1),
        v(14, 16, 7.0, 0.0, 1),
        v(15, 17, 7.5, 0.0, 1),
        v(15, 19, 7.5, 0.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static MAPLE_PLANT: Kind = Kind {
    id: "maple_plant",
    family: Family::Plant,
    shape: Shape::Sprig,
    variants: &[
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
    ],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [0, 0, 0, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static MAPLE_TREE: Kind = Kind {
    id: "maple_tree",
    family: Family::Tree,
    shape: Shape::Round,
    variants: &[
        v(20, 26, 10.0, 1.083, 1),
        v(14, 23, 7.0, 1.0, 1),
        v(13, 20, 6.5, 1.0, 1),
        v(16, 23, 8.0, 1.0, 1),
        v(16, 23, 8.0, 1.0, 1),
        v(16, 23, 8.0, 1.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static MINERAL_ADAMANTINE: Kind = Kind {
    id: "mineral_adamantine",
    family: Family::Mineral,
    shape: Shape::Rock,
    variants: &[
        v(7, 8, 3.5, 0.0, 1),
        v(9, 8, 4.5, 0.0, 1),
        v(8, 9, 4.0, 0.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.19, 0.03, 0.09]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Still,
    flip: true,
    light: Some((0.15, 0.0)),
    sparkle: true,
    speed: 6.0,
};

pub static MINERAL_BONES: Kind = Kind {
    id: "mineral_bones",
    family: Family::Mineral,
    shape: Shape::Rock,
    variants: &[v(10, 7, 5.0, 0.0, 1)],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.19, 0.03, 0.09]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Still,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static MINERAL_GEMS: Kind = Kind {
    id: "mineral_gems",
    family: Family::Mineral,
    shape: Shape::Rock,
    variants: &[v(10, 7, 5.0, 0.0, 1)],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.19, 0.03, 0.09]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Still,
    flip: true,
    light: None,
    sparkle: true,
    speed: 6.0,
};

pub static MINERAL_GOLD: Kind = Kind {
    id: "mineral_gold",
    family: Family::Mineral,
    shape: Shape::Rock,
    variants: &[
        v(10, 7, 5.0, 0.0, 1),
        v(9, 7, 4.5, 0.0, 1),
        v(10, 7, 5.0, 0.0, 1),
        v(8, 6, 4.0, 0.0, 1),
        v(8, 6, 4.0, 0.0, 1),
        v(9, 6, 4.5, 0.0, 1),
        v(6, 4, 3.0, 0.0, 1),
        v(7, 5, 3.5, 0.0, 1),
        v(5, 4, 2.5, 0.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.19, 0.03, 0.09]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Still,
    flip: true,
    light: None,
    sparkle: true,
    speed: 6.0,
};

pub static MINERAL_METALS: Kind = Kind {
    id: "mineral_metals",
    family: Family::Mineral,
    shape: Shape::Rock,
    variants: &[
        v(10, 8, 5.0, 0.0, 1),
        v(10, 7, 5.0, 0.0, 1),
        v(10, 7, 5.0, 0.0, 1),
        v(8, 6, 4.0, 0.0, 1),
        v(9, 6, 4.5, 0.0, 1),
        v(8, 6, 4.0, 0.0, 1),
        v(6, 5, 3.0, 0.0, 1),
        v(7, 5, 3.5, 0.0, 1),
        v(5, 4, 2.5, 0.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.19, 0.03, 0.09]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Still,
    flip: true,
    light: None,
    sparkle: true,
    speed: 6.0,
};

pub static MINERAL_MYTHRIL: Kind = Kind {
    id: "mineral_mythril",
    family: Family::Mineral,
    shape: Shape::Rock,
    variants: &[
        v(8, 9, 4.0, 0.0, 1),
        v(6, 8, 3.0, 0.0, 1),
        v(7, 9, 3.5, 0.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.19, 0.03, 0.09]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Still,
    flip: true,
    light: Some((0.1, 0.0)),
    sparkle: true,
    speed: 6.0,
};

pub static MINERAL_SILVER: Kind = Kind {
    id: "mineral_silver",
    family: Family::Mineral,
    shape: Shape::Rock,
    variants: &[
        v(9, 8, 4.5, 0.0, 1),
        v(8, 9, 4.0, 0.0, 1),
        v(7, 9, 3.5, 0.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.19, 0.03, 0.09]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Still,
    flip: true,
    light: None,
    sparkle: true,
    speed: 6.0,
};

pub static MINERAL_STONE: Kind = Kind {
    id: "mineral_stone",
    family: Family::Mineral,
    shape: Shape::Rock,
    variants: &[
        v(10, 8, 5.0, 0.0, 1),
        v(10, 7, 5.0, 0.0, 1),
        v(10, 8, 5.0, 0.0, 1),
        v(7, 7, 3.5, 0.0, 1),
        v(9, 6, 4.5, 0.0, 1),
        v(8, 7, 4.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 5, 3.0, 0.0, 1),
        v(5, 5, 2.5, 0.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.19, 0.03, 0.09]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Still,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static MUSHROOM_GREEN: Kind = Kind {
    id: "mushroom_green",
    family: Family::Plant,
    shape: Shape::Toadstool,
    variants: &[
        v(5, 5, 2.5, 0.0, 1),
        v(5, 5, 2.5, 0.0, 1),
        v(5, 5, 2.5, 0.0, 1),
    ],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static MUSHROOM_RED: Kind = Kind {
    id: "mushroom_red",
    family: Family::Plant,
    shape: Shape::Toadstool,
    variants: &[
        v(5, 5, 2.5, 0.0, 1),
        v(5, 5, 2.5, 0.0, 1),
        v(5, 5, 2.5, 0.0, 1),
    ],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static MUSHROOM_TEAL: Kind = Kind {
    id: "mushroom_teal",
    family: Family::Plant,
    shape: Shape::Toadstool,
    variants: &[
        v(5, 5, 2.5, 0.0, 1),
        v(5, 5, 2.5, 0.0, 1),
        v(5, 5, 2.5, 0.0, 1),
    ],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static MUSHROOM_TREE: Kind = Kind {
    id: "mushroom_tree",
    family: Family::Tree,
    shape: Shape::Cap,
    variants: &[
        v(13, 16, 6.5, 0.0, 1),
        v(14, 17, 6.452, 0.0, 1),
        v(8, 7, 4.0, 0.0, 1),
        v(13, 17, 6.5, 0.739, 1),
        v(19, 22, 9.5, 0.0, 1),
        v(16, 18, 8.0, 0.0, 1),
        v(12, 12, 6.0, 0.0, 1),
        v(7, 7, 3.5, 0.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static MUSHROOM_WHITE: Kind = Kind {
    id: "mushroom_white",
    family: Family::Plant,
    shape: Shape::Toadstool,
    variants: &[
        v(5, 5, 2.5, 0.0, 1),
        v(5, 5, 2.5, 0.0, 1),
        v(5, 5, 2.5, 0.0, 1),
    ],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static MUSHROOM_YELLOW: Kind = Kind {
    id: "mushroom_yellow",
    family: Family::Plant,
    shape: Shape::Toadstool,
    variants: &[
        v(5, 5, 2.5, 0.0, 1),
        v(5, 5, 2.5, 0.0, 1),
        v(5, 5, 2.5, 0.0, 1),
    ],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static PALM_TREE: Kind = Kind {
    id: "palm_tree",
    family: Family::Tree,
    shape: Shape::Palm,
    variants: &[
        v(14, 20, 9.0, 0.0, 1),
        v(12, 19, 3.0, 1.0, 1),
        v(12, 20, 6.0, 1.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.37, 0.16, 0.0]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static PARADOX_PLANT: Kind = Kind {
    id: "paradox_plant",
    family: Family::Plant,
    shape: Shape::Sprig,
    variants: &[
        v(5, 5, 2.5, 0.0, 1),
        v(5, 5, 2.5, 0.0, 1),
        v(3, 4, 1.5, 0.0, 1),
    ],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [0, 0, 0, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static PARADOX_TREE: Kind = Kind {
    id: "paradox_tree",
    family: Family::Tree,
    shape: Shape::Round,
    variants: &[
        v(16, 24, 8.0, 1.0, 1),
        v(14, 23, 7.0, 1.0, 1),
        v(10, 20, 5.0, 1.0, 1),
        v(16, 23, 8.0, 1.0, 1),
        v(16, 23, 8.0, 1.0, 1),
        v(16, 23, 8.0, 1.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static PINE_TREE: Kind = Kind {
    id: "pine_tree",
    family: Family::Tree,
    shape: Shape::Conifer,
    variants: &[
        v(16, 24, 8.0, 1.0, 1),
        v(14, 23, 7.0, 1.0, 1),
        v(10, 20, 5.0, 1.0, 1),
        v(16, 23, 8.0, 1.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static ROCKLANDS_PLANT: Kind = Kind {
    id: "rocklands_plant",
    family: Family::Plant,
    shape: Shape::Sprig,
    variants: &[
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
    ],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [0, 0, 0, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static ROCKLANDS_TREE: Kind = Kind {
    id: "rocklands_tree",
    family: Family::Tree,
    shape: Shape::Round,
    variants: &[
        v(22, 24, 11.0, 1.0, 1),
        v(24, 23, 12.0, 1.0, 1),
        v(21, 20, 10.5, 1.0, 1),
        v(18, 23, 9.0, 1.0, 1),
        v(16, 23, 8.0, 1.0, 1),
        v(16, 23, 8.0, 1.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Still,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static SAVANNA_PLANT: Kind = Kind {
    id: "savanna_plant",
    family: Family::Plant,
    shape: Shape::Sprig,
    variants: &[
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
    ],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [0, 0, 0, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static SAVANNA_TREE_1: Kind = Kind {
    id: "savanna_tree_1",
    family: Family::Tree,
    shape: Shape::Umbrella,
    variants: &[
        v(18, 16, 7.875, 0.0, 1),
        v(12, 9, 6.0, 0.0, 1),
        v(16, 14, 9.12, 0.0, 1),
        v(18, 16, 7.875, 0.0, 1),
        v(12, 9, 6.0, 0.0, 1),
        v(16, 14, 9.12, 0.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static SAVANNA_TREE_2: Kind = Kind {
    id: "savanna_tree_2",
    family: Family::Tree,
    shape: Shape::Umbrella,
    variants: &[
        v(16, 16, 6.88, 0.0, 1),
        v(20, 18, 11.2, 0.0, 1),
        v(18, 18, 7.92, 0.0, 1),
        v(15, 11, 8.1, 0.0, 1),
        v(16, 16, 6.88, 0.0, 1),
        v(20, 18, 11.2, 0.0, 1),
        v(18, 18, 7.92, 0.0, 1),
        v(15, 11, 8.1, 0.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static SAVANNA_TREE_BIG_1: Kind = Kind {
    id: "savanna_tree_big_1",
    family: Family::Tree,
    shape: Shape::Umbrella,
    variants: &[
        v(22, 23, 9.0, 2.0, 1),
        v(22, 23, 9.0, 2.0, 1),
        v(22, 23, 9.0, 2.0, 1),
    ],
    bare: &[],
    mini: (5, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [2, 2, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static SAVANNA_TREE_BIG_2: Kind = Kind {
    id: "savanna_tree_big_2",
    family: Family::Tree,
    shape: Shape::Umbrella,
    variants: &[
        v(22, 23, 9.0, 2.0, 1),
        v(22, 23, 9.0, 2.0, 1),
        v(22, 23, 9.0, 2.0, 1),
    ],
    bare: &[],
    mini: (5, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [2, 2, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static SINGULARITY_PLANT: Kind = Kind {
    id: "singularity_plant",
    family: Family::Plant,
    shape: Shape::Sprig,
    variants: &[
        v(5, 5, 2.5, 0.0, 1),
        v(5, 5, 2.5, 0.0, 1),
        v(3, 4, 1.5, 0.0, 1),
    ],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [0, 0, 0, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static SINGULARITY_TREE: Kind = Kind {
    id: "singularity_tree",
    family: Family::Tree,
    shape: Shape::Round,
    variants: &[
        v(16, 24, 8.0, 0.0, 20),
        v(14, 23, 7.0, 0.0, 16),
        v(12, 20, 6.0, 0.0, 14),
        v(16, 23, 8.0, 0.0, 19),
        v(16, 23, 8.0, 0.0, 15),
        v(16, 23, 8.0, 0.0, 11),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static SNOW_PLANT: Kind = Kind {
    id: "snow_plant",
    family: Family::Plant,
    shape: Shape::Sprig,
    variants: &[
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
        v(6, 6, 3.0, 0.0, 1),
    ],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [0, 0, 0, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static SWAMP_PLANT: Kind = Kind {
    id: "swamp_plant",
    family: Family::Plant,
    shape: Shape::Reed,
    variants: &[v(14, 23, 7.0, 1.0, 1)],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [0, 0, 0, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static SWAMP_PLANT_BIG: Kind = Kind {
    id: "swamp_plant_big",
    family: Family::Plant,
    shape: Shape::Reed,
    variants: &[
        v(10, 20, 5.0, 1.0, 1),
        v(18, 23, 9.0, 1.0, 1),
        v(16, 24, 8.0, 1.0, 1),
        v(18, 23, 9.0, 0.958, 1),
    ],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static SWAMP_TREE: Kind = Kind {
    id: "swamp_tree",
    family: Family::Tree,
    shape: Shape::Willow,
    variants: &[
        v(14, 18, 7.0, 0.75, 1),
        v(6, 7, 3.0, 0.304, 1),
        v(9, 12, 4.5, 0.6, 1),
        v(20, 22, 10.0, 0.957, 1),
        v(14, 19, 7.0, 0.792, 1),
        v(14, 14, 7.0, 0.609, 1),
        v(7, 9, 3.5, 0.45, 1),
        v(24, 24, 12.0, 1.043, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static TREE_GREEN_1: Kind = Kind {
    id: "tree_green_1",
    family: Family::Tree,
    shape: Shape::Round,
    variants: &[
        v(8, 9, 4.0, 0.0, 1),
        v(11, 14, 6.0, 1.0, 1),
        v(14, 15, 8.0, 1.0, 1),
        v(16, 21, 8.0, 1.0, 1),
        v(16, 21, 8.0, 1.0, 1),
        v(12, 17, 6.0, 1.0, 1),
        v(9, 13, 4.0, 1.0, 1),
        v(6, 10, 3.0, 0.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static TREE_GREEN_2: Kind = Kind {
    id: "tree_green_2",
    family: Family::Tree,
    shape: Shape::Round,
    variants: &[
        v(14, 21, 7.0, 1.0, 1),
        v(12, 16, 6.0, 1.0, 1),
        v(11, 12, 5.5, 0.0, 1),
        v(8, 10, 4.0, 0.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static TREE_GREEN_3: Kind = Kind {
    id: "tree_green_3",
    family: Family::Tree,
    shape: Shape::Round,
    variants: &[
        v(8, 9, 4.0, 0.0, 1),
        v(10, 13, 5.0, 1.0, 1),
        v(12, 16, 6.0, 1.0, 1),
        v(14, 21, 7.0, 1.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static WASTELAND_FLOWER: Kind = Kind {
    id: "wasteland_flower",
    family: Family::Plant,
    shape: Shape::Blossom,
    variants: &[
        v(5, 5, 2.5, 0.0, 1),
        v(5, 5, 2.5, 0.0, 1),
        v(3, 4, 1.5, 0.0, 1),
    ],
    bare: &[],
    mini: (1, 1),
    shadow: None,
    fundament: [0, 0, 0, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static WASTELAND_TREE: Kind = Kind {
    id: "wasteland_tree",
    family: Family::Tree,
    shape: Shape::Dead,
    variants: &[
        v(15, 14, 6.562, 0.0, 1),
        v(13, 15, 6.5, 0.0, 1),
        v(10, 8, 5.0, 0.0, 1),
        v(17, 14, 8.5, 0.0, 1),
        v(18, 20, 9.0, 0.0, 1),
        v(13, 15, 6.5, 0.0, 1),
        v(13, 13, 6.5, 0.0, 1),
        v(11, 11, 5.5, 0.0, 1),
    ],
    bare: &[],
    mini: (3, 2),
    shadow: Some([0.5, 0.14, 0.08]),
    fundament: [1, 1, 1, 0],
    wind: Wind::Leaves,
    flip: true,
    light: None,
    sparkle: false,
    speed: 6.0,
};

pub static KINDS: &[&Kind] = &[
    &BIRCH_PLANT,
    &BIRCH_TREE,
    &CACTI_TREE,
    &CANDY_PLANT,
    &CANDY_TREE,
    &CELESTIAL_PLANT,
    &CELESTIAL_TREE,
    &CELESTIAL_TREE_SMALL,
    &CLOVER_PLANT,
    &CLOVER_TREE,
    &CORRUPTED_PLANT,
    &CORRUPTED_TREE,
    &CORRUPTED_TREE_BIG,
    &CRYSTAL_PLANT,
    &CRYSTAL_TREE,
    &DESERT_PLANT,
    &DESERT_TREE,
    &ENCHANTED_TREE,
    &FLAME_FLOWER,
    &FLOWER,
    &FLOWER_PLANT,
    &FLOWER_TREE_1,
    &FLOWER_TREE_2,
    &FLOWER_TREE_3,
    &FRUIT_BUSH,
    &GARLIC_PLANT,
    &GARLIC_TREE,
    &GREEN_HERB,
    &INFERNAL_TREE,
    &INFERNAL_TREE_BIG,
    &INFERNAL_TREE_SMALL,
    &JUNGLE_FLOWER,
    &JUNGLE_PLANT,
    &JUNGLE_TREE,
    &LEMON_TREE,
    &MAPLE_PLANT,
    &MAPLE_TREE,
    &MINERAL_ADAMANTINE,
    &MINERAL_BONES,
    &MINERAL_GEMS,
    &MINERAL_GOLD,
    &MINERAL_METALS,
    &MINERAL_MYTHRIL,
    &MINERAL_SILVER,
    &MINERAL_STONE,
    &MUSHROOM_GREEN,
    &MUSHROOM_RED,
    &MUSHROOM_TEAL,
    &MUSHROOM_TREE,
    &MUSHROOM_WHITE,
    &MUSHROOM_YELLOW,
    &PALM_TREE,
    &PARADOX_PLANT,
    &PARADOX_TREE,
    &PINE_TREE,
    &ROCKLANDS_PLANT,
    &ROCKLANDS_TREE,
    &SAVANNA_PLANT,
    &SAVANNA_TREE_1,
    &SAVANNA_TREE_2,
    &SAVANNA_TREE_BIG_1,
    &SAVANNA_TREE_BIG_2,
    &SINGULARITY_PLANT,
    &SINGULARITY_TREE,
    &SNOW_PLANT,
    &SWAMP_PLANT,
    &SWAMP_PLANT_BIG,
    &SWAMP_TREE,
    &TREE_GREEN_1,
    &TREE_GREEN_2,
    &TREE_GREEN_3,
    &WASTELAND_FLOWER,
    &WASTELAND_TREE,
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pot {
    pub kind: &'static Kind,
    pub weight: u8,
}

const fn pot(kind: &'static Kind, weight: u8) -> Pot {
    Pot { kind, weight }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Biome {
    pub id: &'static str,
    pub trees: &'static [Pot],
    pub plants: &'static [Pot],
    pub bushes: &'static [Pot],
    pub minerals: &'static [Pot],
}

pub static BIOME_BIOMASS: Biome = Biome {
    id: "biome_biomass",
    trees: &[],
    plants: &[],
    bushes: &[],
    minerals: &[],
};

pub static BIOME_BIRCH: Biome = Biome {
    id: "biome_birch",
    trees: &[pot(&BIRCH_TREE, 1)],
    plants: &[
        pot(&BIRCH_PLANT, 1),
        pot(&MUSHROOM_RED, 1),
        pot(&MUSHROOM_GREEN, 1),
        pot(&MUSHROOM_TEAL, 1),
        pot(&MUSHROOM_WHITE, 1),
        pot(&MUSHROOM_YELLOW, 1),
        pot(&GREEN_HERB, 4),
        pot(&FLOWER, 4),
    ],
    bushes: &[pot(&FRUIT_BUSH, 1)],
    minerals: &[pot(&MINERAL_STONE, 4), pot(&MINERAL_METALS, 3)],
};

pub static BIOME_CANDY: Biome = Biome {
    id: "biome_candy",
    trees: &[pot(&CANDY_TREE, 1)],
    plants: &[pot(&CANDY_PLANT, 1)],
    bushes: &[],
    minerals: &[pot(&MINERAL_STONE, 4), pot(&MINERAL_METALS, 3)],
};

pub static BIOME_CELESTIAL: Biome = Biome {
    id: "biome_celestial",
    trees: &[pot(&CELESTIAL_TREE, 10), pot(&CELESTIAL_TREE_SMALL, 1)],
    plants: &[pot(&CELESTIAL_PLANT, 1)],
    bushes: &[],
    minerals: &[
        pot(&MINERAL_STONE, 4),
        pot(&MINERAL_METALS, 3),
        pot(&MINERAL_MYTHRIL, 1),
    ],
};

pub static BIOME_CLOVER: Biome = Biome {
    id: "biome_clover",
    trees: &[pot(&CLOVER_TREE, 1)],
    plants: &[pot(&CLOVER_PLANT, 1)],
    bushes: &[pot(&FRUIT_BUSH, 1)],
    minerals: &[
        pot(&MINERAL_STONE, 4),
        pot(&MINERAL_METALS, 3),
        pot(&MINERAL_GEMS, 1),
    ],
};

pub static BIOME_CORRUPTED: Biome = Biome {
    id: "biome_corrupted",
    trees: &[pot(&CORRUPTED_TREE, 1), pot(&CORRUPTED_TREE_BIG, 1)],
    plants: &[pot(&CORRUPTED_PLANT, 1)],
    bushes: &[],
    minerals: &[pot(&MINERAL_BONES, 7)],
};

pub static BIOME_CRYSTAL: Biome = Biome {
    id: "biome_crystal",
    trees: &[pot(&CRYSTAL_TREE, 1)],
    plants: &[pot(&CRYSTAL_PLANT, 1)],
    bushes: &[],
    minerals: &[],
};

pub static BIOME_CYBERTILE: Biome = Biome {
    id: "biome_cybertile",
    trees: &[],
    plants: &[],
    bushes: &[],
    minerals: &[],
};

pub static BIOME_DESERT: Biome = Biome {
    id: "biome_desert",
    trees: &[pot(&DESERT_TREE, 1)],
    plants: &[pot(&DESERT_PLANT, 1)],
    bushes: &[],
    minerals: &[
        pot(&MINERAL_STONE, 4),
        pot(&MINERAL_METALS, 3),
        pot(&MINERAL_GEMS, 1),
        pot(&MINERAL_SILVER, 1),
    ],
};

pub static BIOME_ENCHANTED: Biome = Biome {
    id: "biome_enchanted",
    trees: &[pot(&ENCHANTED_TREE, 1)],
    plants: &[pot(&FLOWER, 5)],
    bushes: &[pot(&FRUIT_BUSH, 1)],
    minerals: &[
        pot(&MINERAL_SILVER, 5),
        pot(&MINERAL_METALS, 2),
        pot(&MINERAL_GEMS, 1),
    ],
};

pub static BIOME_FLOWER: Biome = Biome {
    id: "biome_flower",
    trees: &[
        pot(&FLOWER_TREE_1, 1),
        pot(&FLOWER_TREE_2, 1),
        pot(&FLOWER_TREE_3, 1),
    ],
    plants: &[
        pot(&FLOWER_PLANT, 3),
        pot(&MUSHROOM_RED, 1),
        pot(&MUSHROOM_GREEN, 1),
        pot(&MUSHROOM_TEAL, 1),
        pot(&MUSHROOM_WHITE, 1),
        pot(&MUSHROOM_YELLOW, 1),
        pot(&GREEN_HERB, 5),
        pot(&FLOWER, 20),
    ],
    bushes: &[pot(&FRUIT_BUSH, 1)],
    minerals: &[pot(&MINERAL_STONE, 4), pot(&MINERAL_METALS, 3)],
};

pub static BIOME_GARLIC: Biome = Biome {
    id: "biome_garlic",
    trees: &[pot(&GARLIC_TREE, 1)],
    plants: &[pot(&GARLIC_PLANT, 1)],
    bushes: &[pot(&FRUIT_BUSH, 1)],
    minerals: &[pot(&MINERAL_STONE, 4), pot(&MINERAL_METALS, 3)],
};

pub static BIOME_GRASS: Biome = Biome {
    id: "biome_grass",
    trees: &[
        pot(&TREE_GREEN_1, 1),
        pot(&TREE_GREEN_2, 1),
        pot(&TREE_GREEN_3, 1),
    ],
    plants: &[
        pot(&MUSHROOM_RED, 2),
        pot(&MUSHROOM_GREEN, 2),
        pot(&MUSHROOM_TEAL, 2),
        pot(&MUSHROOM_WHITE, 2),
        pot(&MUSHROOM_YELLOW, 2),
        pot(&GREEN_HERB, 4),
        pot(&FLOWER, 4),
    ],
    bushes: &[pot(&FRUIT_BUSH, 1)],
    minerals: &[pot(&MINERAL_STONE, 4), pot(&MINERAL_METALS, 3)],
};

pub static BIOME_HILL: Biome = Biome {
    id: "biome_hill",
    trees: &[],
    plants: &[],
    bushes: &[],
    minerals: &[pot(&MINERAL_STONE, 4), pot(&MINERAL_METALS, 1)],
};

pub static BIOME_INFERNAL: Biome = Biome {
    id: "biome_infernal",
    trees: &[
        pot(&INFERNAL_TREE, 1),
        pot(&INFERNAL_TREE_BIG, 1),
        pot(&INFERNAL_TREE_SMALL, 1),
    ],
    plants: &[pot(&FLAME_FLOWER, 1)],
    bushes: &[],
    minerals: &[pot(&MINERAL_METALS, 3), pot(&MINERAL_ADAMANTINE, 3)],
};

pub static BIOME_JUNGLE: Biome = Biome {
    id: "biome_jungle",
    trees: &[pot(&JUNGLE_TREE, 1)],
    plants: &[pot(&JUNGLE_PLANT, 1), pot(&JUNGLE_FLOWER, 1)],
    bushes: &[pot(&FRUIT_BUSH, 1)],
    minerals: &[
        pot(&MINERAL_SILVER, 5),
        pot(&MINERAL_METALS, 1),
        pot(&MINERAL_GEMS, 1),
    ],
};

pub static BIOME_LEMON: Biome = Biome {
    id: "biome_lemon",
    trees: &[pot(&LEMON_TREE, 1)],
    plants: &[
        pot(&MUSHROOM_RED, 1),
        pot(&MUSHROOM_GREEN, 1),
        pot(&MUSHROOM_TEAL, 1),
        pot(&MUSHROOM_WHITE, 1),
        pot(&MUSHROOM_YELLOW, 1),
        pot(&GREEN_HERB, 5),
        pot(&FLOWER, 5),
    ],
    bushes: &[pot(&FRUIT_BUSH, 1)],
    minerals: &[pot(&MINERAL_STONE, 5), pot(&MINERAL_METALS, 3)],
};

pub static BIOME_MAPLE: Biome = Biome {
    id: "biome_maple",
    trees: &[pot(&MAPLE_TREE, 1)],
    plants: &[
        pot(&MAPLE_PLANT, 1),
        pot(&MUSHROOM_RED, 1),
        pot(&MUSHROOM_GREEN, 1),
        pot(&MUSHROOM_TEAL, 1),
        pot(&MUSHROOM_WHITE, 1),
        pot(&MUSHROOM_YELLOW, 1),
        pot(&GREEN_HERB, 4),
        pot(&FLOWER, 4),
    ],
    bushes: &[pot(&FRUIT_BUSH, 1)],
    minerals: &[pot(&MINERAL_STONE, 4), pot(&MINERAL_METALS, 3)],
};

pub static BIOME_MUSHROOM: Biome = Biome {
    id: "biome_mushroom",
    trees: &[pot(&MUSHROOM_TREE, 1)],
    plants: &[
        pot(&FLOWER, 2),
        pot(&MUSHROOM_RED, 2),
        pot(&MUSHROOM_GREEN, 2),
        pot(&MUSHROOM_TEAL, 2),
        pot(&MUSHROOM_WHITE, 2),
        pot(&MUSHROOM_YELLOW, 2),
    ],
    bushes: &[pot(&FRUIT_BUSH, 1)],
    minerals: &[
        pot(&MINERAL_SILVER, 3),
        pot(&MINERAL_STONE, 5),
        pot(&MINERAL_METALS, 3),
        pot(&MINERAL_GEMS, 1),
    ],
};

pub static BIOME_PARADOX: Biome = Biome {
    id: "biome_paradox",
    trees: &[pot(&PARADOX_TREE, 1)],
    plants: &[pot(&PARADOX_PLANT, 1)],
    bushes: &[],
    minerals: &[
        pot(&MINERAL_STONE, 4),
        pot(&MINERAL_METALS, 3),
        pot(&MINERAL_MYTHRIL, 1),
    ],
};

pub static BIOME_PERMAFROST: Biome = Biome {
    id: "biome_permafrost",
    trees: &[pot(&PINE_TREE, 1)],
    plants: &[pot(&SNOW_PLANT, 1)],
    bushes: &[],
    minerals: &[pot(&MINERAL_STONE, 4), pot(&MINERAL_METALS, 3)],
};

pub static BIOME_PUMPKIN: Biome = Biome {
    id: "biome_pumpkin",
    trees: &[],
    plants: &[],
    bushes: &[],
    minerals: &[],
};

pub static BIOME_ROCKLANDS: Biome = Biome {
    id: "biome_rocklands",
    trees: &[pot(&ROCKLANDS_TREE, 1)],
    plants: &[pot(&ROCKLANDS_PLANT, 1)],
    bushes: &[],
    minerals: &[
        pot(&MINERAL_STONE, 4),
        pot(&MINERAL_METALS, 3),
        pot(&MINERAL_GEMS, 1),
    ],
};

pub static BIOME_SAND: Biome = Biome {
    id: "biome_sand",
    trees: &[],
    plants: &[],
    bushes: &[],
    minerals: &[],
};

pub static BIOME_SAVANNA: Biome = Biome {
    id: "biome_savanna",
    trees: &[
        pot(&SAVANNA_TREE_1, 5),
        pot(&SAVANNA_TREE_2, 5),
        pot(&SAVANNA_TREE_BIG_1, 1),
        pot(&SAVANNA_TREE_BIG_2, 1),
    ],
    plants: &[pot(&SAVANNA_PLANT, 1)],
    bushes: &[pot(&FRUIT_BUSH, 1)],
    minerals: &[pot(&MINERAL_STONE, 5), pot(&MINERAL_METALS, 3)],
};

pub static BIOME_SINGULARITY: Biome = Biome {
    id: "biome_singularity",
    trees: &[pot(&SINGULARITY_TREE, 1)],
    plants: &[pot(&SINGULARITY_PLANT, 1)],
    bushes: &[],
    minerals: &[pot(&MINERAL_STONE, 4), pot(&MINERAL_METALS, 3)],
};

pub static BIOME_SWAMP: Biome = Biome {
    id: "biome_swamp",
    trees: &[pot(&SWAMP_TREE, 1)],
    plants: &[pot(&SWAMP_PLANT, 1), pot(&SWAMP_PLANT_BIG, 1)],
    bushes: &[],
    minerals: &[pot(&MINERAL_STONE, 4), pot(&MINERAL_METALS, 3)],
};

pub static BIOME_TUMOR: Biome = Biome {
    id: "biome_tumor",
    trees: &[],
    plants: &[],
    bushes: &[],
    minerals: &[],
};

pub static BIOME_WASTELAND: Biome = Biome {
    id: "biome_wasteland",
    trees: &[pot(&WASTELAND_TREE, 1)],
    plants: &[pot(&WASTELAND_FLOWER, 1)],
    bushes: &[],
    minerals: &[pot(&MINERAL_STONE, 4), pot(&MINERAL_METALS, 3)],
};
