use econbox_tile::{Ground, Overlay, Tile, Wall};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Ocean,
    Ground,
    Lava,
    Block,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Paint {
    Ground(Ground),
    Overlay(Overlay, bool),
    Wall(Wall),
    Fixed([u8; 3]),
}

impl Paint {
    pub fn color(self) -> [u8; 3] {
        match self {
            Self::Ground(ground) => ground.color(),
            Self::Overlay(overlay, high) => overlay
                .color(high)
                .unwrap_or_else(|| Ground::SoilLow.color()),
            Self::Wall(wall) => wall.color(),
            Self::Fixed(color) => color,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Water,
    Sand,
    Soil,
    Grass,
    Speckle([u8; 3]),
    Lava([u8; 3]),
    Hills,
    Mountains,
    Summit,
    Rocks,
    Raised,
    Void,
    Plinth,
    Outline,
    Rim,
    Pit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Look {
    pub id: &'static str,
    pub paint: Paint,
    pub z: u8,
    pub layer: Layer,
    pub height_min: i16,
    pub edge: Option<[u8; 3]>,
    pub liquid: bool,
    pub ocean: bool,
    pub ground: bool,
    pub block: bool,
    pub rocks: bool,
    pub edge_hills: bool,
    pub edge_mountains: bool,
    pub wall: bool,
    pub fills_with_ocean: bool,
    pub explodable: bool,
    pub check_edge: bool,
    pub edge_frame: Option<u8>,
    pub variants: u8,
    pub style: Style,
}

impl Look {
    pub fn color(&self) -> [u8; 3] {
        self.paint.color()
    }

    pub fn is(&self, other: &Look) -> bool {
        self.id == other.id
    }
}

const fn look(id: &'static str, paint: Paint, z: u8, variants: u8, style: Style) -> Look {
    Look {
        id,
        paint,
        z,
        layer: Layer::Ground,
        height_min: -1,
        edge: None,
        liquid: false,
        ocean: false,
        ground: false,
        block: false,
        rocks: false,
        edge_hills: false,
        edge_mountains: false,
        wall: false,
        fills_with_ocean: false,
        explodable: false,
        check_edge: true,
        edge_frame: None,
        variants,
        style,
    }
}

const fn hex(rgb: u32) -> [u8; 3] {
    [(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8]
}

pub static DEEP_OCEAN: Look = Look {
    layer: Layer::Ocean,
    height_min: 0,
    liquid: true,
    ocean: true,
    ..look(
        "deep_ocean",
        Paint::Ground(Ground::DeepOcean),
        0,
        1,
        Style::Water,
    )
};

pub static CLOSE_OCEAN: Look = Look {
    layer: Layer::Ocean,
    height_min: 30,
    liquid: true,
    ocean: true,
    ..look(
        "close_ocean",
        Paint::Ground(Ground::CloseOcean),
        5,
        1,
        Style::Water,
    )
};

pub static SHALLOW_WATERS: Look = Look {
    layer: Layer::Ocean,
    height_min: 70,
    edge: Some(hex(0x3f90ea)),
    liquid: true,
    ocean: true,
    ..look(
        "shallow_waters",
        Paint::Ground(Ground::ShallowWaters),
        6,
        1,
        Style::Water,
    )
};

pub static SAND: Look = Look {
    height_min: 98,
    edge: Some(hex(0xd8c08c)),
    ground: true,
    ..look("sand", Paint::Ground(Ground::Sand), 18, 3, Style::Sand)
};

pub static SOIL_LOW: Look = Look {
    height_min: 108,
    ground: true,
    ..look(
        "soil_low",
        Paint::Ground(Ground::SoilLow),
        20,
        3,
        Style::Soil,
    )
};

pub static SOIL_HIGH: Look = Look {
    height_min: 128,
    ground: true,
    ..look(
        "soil_high",
        Paint::Ground(Ground::SoilHigh),
        21,
        3,
        Style::Soil,
    )
};

pub static LAVA2: Look = Look {
    layer: Layer::Lava,
    liquid: true,
    ..look(
        "lava2",
        Paint::Ground(Ground::Lava2),
        26,
        1,
        Style::Lava(hex(0xffde00)),
    )
};

pub static LAVA3: Look = Look {
    layer: Layer::Lava,
    liquid: true,
    ..look(
        "lava3",
        Paint::Ground(Ground::Lava3),
        27,
        1,
        Style::Lava(hex(0xfff3b0)),
    )
};

pub static HILLS: Look = Look {
    height_min: 199,
    ground: true,
    rocks: true,
    edge_hills: true,
    ..look("hills", Paint::Ground(Ground::Hills), 92, 3, Style::Hills)
};

pub static MOUNTAINS: Look = Look {
    layer: Layer::Block,
    height_min: 210,
    block: true,
    rocks: true,
    edge_mountains: true,
    edge_frame: Some(2),
    ..look(
        "mountains",
        Paint::Ground(Ground::Mountains),
        94,
        3,
        Style::Mountains,
    )
};

pub static SUMMIT: Look = Look {
    layer: Layer::Block,
    height_min: 230,
    block: true,
    rocks: true,
    edge_mountains: true,
    edge_frame: Some(2),
    ..look(
        "summit",
        Paint::Ground(Ground::Summit),
        96,
        6,
        Style::Summit,
    )
};

pub static BORDER_PIT: Look = look("border_pit", Paint::Fixed(hex(0x4a4a4a)), 7, 1, Style::Pit);

pub static BORDER_WATER: Look = look(
    "border_water",
    Paint::Fixed(hex(0x88c6f4)),
    8,
    1,
    Style::Outline,
);

pub static BORDER_WATER_RUNUP: Look = look(
    "border_water_runup",
    Paint::Fixed(hex(0x88c6f4)),
    9,
    1,
    Style::Rim,
);

pub const RIM_TINT: [u8; 3] = [0xdd, 0xfc, 0xff];

pub const RIM_ALPHA: (f32, f32, f32) = (0.02, 0.7, 0.4);

const fn barrier(id: &'static str, wall: Wall, z: u8, edge: u32) -> Look {
    Look {
        layer: Layer::Block,
        edge: Some(hex(edge)),
        block: true,
        wall: true,
        ..look(id, Paint::Wall(wall), z, 1, Style::Plinth)
    }
}

pub static WALL_EVIL: Look = barrier("wall_evil", Wall::Evil, 10, 0x5a1c32);
pub static WALL_ORDER: Look = barrier("wall_order", Wall::Order, 11, 0x787f87);
pub static WALL_ANCIENT: Look = barrier("wall_ancient", Wall::Ancient, 12, 0x3a485e);
pub static WALL_WILD: Look = barrier("wall_wild", Wall::Wild, 13, 0x4b2e1b);
pub static WALL_GREEN: Look = barrier("wall_green", Wall::Green, 14, 0x3f6b3f);
pub static WALL_IRON: Look = barrier("wall_iron", Wall::Iron, 15, 0x2e363e);
pub static WALL_LIGHT: Look = barrier("wall_light", Wall::Light, 16, 0xe7dc42);

const fn biome(
    overlay: Overlay,
    ids: [&'static str; 2],
    z: [u8; 2],
    variants: [u8; 2],
    style: Style,
) -> [Look; 2] {
    [
        Look {
            height_min: 108,
            ground: true,
            ..look(
                ids[0],
                Paint::Overlay(overlay, false),
                z[0],
                variants[0],
                style,
            )
        },
        Look {
            height_min: 128,
            ground: true,
            ..look(
                ids[1],
                Paint::Overlay(overlay, true),
                z[1],
                variants[1],
                style,
            )
        },
    ]
}

const fn low_height(pair: [Look; 2]) -> [Look; 2] {
    let [low, high] = pair;
    [
        low,
        Look {
            height_min: 108,
            ..high
        },
    ]
}

const fn high_edge(pair: [Look; 2], edge: u32) -> [Look; 2] {
    let [low, high] = pair;
    [
        low,
        Look {
            edge: Some(hex(edge)),
            ..high
        },
    ]
}

static GRASS: [Look; 2] = biome(
    Overlay::Grass,
    ["grass_low", "grass_high"],
    [40, 41],
    [3, 3],
    Style::Grass,
);
static JUNGLE: [Look; 2] = biome(
    Overlay::Jungle,
    ["jungle_low", "jungle_high"],
    [70, 71],
    [3, 3],
    Style::Grass,
);
static ENCHANTED: [Look; 2] = low_height(biome(
    Overlay::Enchanted,
    ["enchanted_low", "enchanted_high"],
    [44, 45],
    [3, 3],
    Style::Speckle(hex(0xd6f7a4)),
));
static INFERNAL: [Look; 2] = biome(
    Overlay::Infernal,
    ["infernal_low", "infernal_high"],
    [72, 73],
    [3, 3],
    Style::Speckle(hex(0xe2622e)),
);
static PUMPKIN: [Look; 2] = biome(
    Overlay::Pumpkin,
    ["pumpkin_low", "pumpkin_high"],
    [36, 37],
    [3, 3],
    Style::Speckle(hex(0xd07a22)),
);
static SAVANNA: [Look; 2] = biome(
    Overlay::Savanna,
    ["savanna_low", "savanna_high"],
    [68, 69],
    [3, 3],
    Style::Grass,
);
static BIRCH: [Look; 2] = biome(
    Overlay::Birch,
    ["birch_low", "birch_high"],
    [48, 49],
    [5, 3],
    Style::Grass,
);
static MAPLE: [Look; 2] = biome(
    Overlay::Maple,
    ["maple_low", "maple_high"],
    [50, 51],
    [5, 3],
    Style::Speckle(hex(0xd8662a)),
);
static SWAMP: [Look; 2] = biome(
    Overlay::Swamp,
    ["swamp_low", "swamp_high"],
    [28, 31],
    [5, 3],
    Style::Speckle(hex(0x4c675e)),
);
static CORRUPTED: [Look; 2] = biome(
    Overlay::Blight,
    ["corrupted_low", "corrupted_high"],
    [42, 43],
    [3, 3],
    Style::Speckle(hex(0x8a5a86)),
);
static MUSHROOM: [Look; 2] = biome(
    Overlay::Mushroom,
    ["mushroom_low", "mushroom_high"],
    [46, 47],
    [6, 3],
    Style::Speckle(hex(0x8a6466)),
);
static CANDY: [Look; 2] = biome(
    Overlay::Candy,
    ["candy_low", "candy_high"],
    [77, 78],
    [3, 3],
    Style::Speckle(hex(0xfff0f6)),
);
static GARLIC: [Look; 2] = biome(
    Overlay::Garlic,
    ["garlic_low", "garlic_high"],
    [54, 55],
    [5, 5],
    Style::Speckle(hex(0xe4e2c8)),
);
static LEMON: [Look; 2] = biome(
    Overlay::Lemon,
    ["lemon_low", "lemon_high"],
    [79, 80],
    [3, 3],
    Style::Speckle(hex(0xf4ea4c)),
);
static DESERT: [Look; 2] = biome(
    Overlay::Desert,
    ["desert_low", "desert_high"],
    [29, 74],
    [3, 3],
    Style::Sand,
);
static ROCKLANDS: [Look; 2] = biome(
    Overlay::Rockland,
    ["rocklands_low", "rocklands_high"],
    [52, 53],
    [5, 5],
    Style::Rocks,
);
static CRYSTAL: [Look; 2] = biome(
    Overlay::Crystal,
    ["crystal_low", "crystal_high"],
    [75, 76],
    [3, 3],
    Style::Speckle(hex(0xc8fffa)),
);
static PERMAFROST: [Look; 2] = high_edge(
    biome(
        Overlay::Frost,
        ["permafrost_low", "permafrost_high"],
        [83, 84],
        [4, 3],
        Style::Speckle(hex(0xe6f2fa)),
    ),
    0xf5f7f6,
);
static FLOWER: [Look; 2] = biome(
    Overlay::Flower,
    ["flower_low", "flower_high"],
    [56, 57],
    [5, 3],
    Style::Speckle(hex(0x3f9a3c)),
);
static CELESTIAL: [Look; 2] = biome(
    Overlay::Starlight,
    ["celestial_low", "celestial_high"],
    [58, 59],
    [5, 5],
    Style::Speckle(hex(0xe8c8ff)),
);
static SINGULARITY: [Look; 2] = biome(
    Overlay::Void,
    ["singularity_low", "singularity_high"],
    [60, 61],
    [2, 2],
    Style::Void,
);
static PARADOX: [Look; 2] = biome(
    Overlay::Paradox,
    ["paradox_low", "paradox_high"],
    [64, 65],
    [3, 3],
    Style::Raised,
);
static CLOVER: [Look; 2] = biome(
    Overlay::Clover,
    ["clover_low", "clover_high"],
    [62, 63],
    [3, 3],
    Style::Speckle(hex(0x2f7a64)),
);

pub fn ground(ground: Ground) -> &'static Look {
    match ground {
        Ground::DeepOcean => &DEEP_OCEAN,
        Ground::CloseOcean => &CLOSE_OCEAN,
        Ground::ShallowWaters => &SHALLOW_WATERS,
        Ground::Sand => &SAND,
        Ground::SoilLow => &SOIL_LOW,
        Ground::SoilHigh => &SOIL_HIGH,
        Ground::Hills => &HILLS,
        Ground::Mountains => &MOUNTAINS,
        Ground::Summit => &SUMMIT,
        Ground::Lava2 => &LAVA2,
        Ground::Lava3 => &LAVA3,
    }
}

pub fn biome_pair(overlay: Overlay) -> Option<&'static [Look; 2]> {
    Some(match overlay {
        Overlay::Bare => return None,
        Overlay::Grass => &GRASS,
        Overlay::Jungle => &JUNGLE,
        Overlay::Enchanted => &ENCHANTED,
        Overlay::Infernal => &INFERNAL,
        Overlay::Pumpkin => &PUMPKIN,
        Overlay::Savanna => &SAVANNA,
        Overlay::Birch => &BIRCH,
        Overlay::Maple => &MAPLE,
        Overlay::Swamp => &SWAMP,
        Overlay::Blight => &CORRUPTED,
        Overlay::Mushroom => &MUSHROOM,
        Overlay::Candy => &CANDY,
        Overlay::Garlic => &GARLIC,
        Overlay::Lemon => &LEMON,
        Overlay::Desert => &DESERT,
        Overlay::Rockland => &ROCKLANDS,
        Overlay::Crystal => &CRYSTAL,
        Overlay::Frost => &PERMAFROST,
        Overlay::Flower => &FLOWER,
        Overlay::Starlight => &CELESTIAL,
        Overlay::Void => &SINGULARITY,
        Overlay::Paradox => &PARADOX,
        Overlay::Clover => &CLOVER,
    })
}

pub fn wall(wall: Wall) -> &'static Look {
    match wall {
        Wall::Evil => &WALL_EVIL,
        Wall::Order => &WALL_ORDER,
        Wall::Ancient => &WALL_ANCIENT,
        Wall::Wild => &WALL_WILD,
        Wall::Green => &WALL_GREEN,
        Wall::Iron => &WALL_IRON,
        Wall::Light => &WALL_LIGHT,
    }
}

pub fn main(tile: Tile) -> &'static Look {
    ground(tile.ground)
}

pub fn of(tile: Tile) -> &'static Look {
    if let Some(kind) = tile.wall {
        return wall(kind);
    }

    match biome_pair(tile.overlay) {
        Some(pair) => &pair[usize::from(tile.ground.is_high())],
        None => ground(tile.ground),
    }
}

pub fn all() -> Vec<&'static Look> {
    let mut looks: Vec<&'static Look> = econbox_tile::GROUNDS.into_iter().map(ground).collect();
    looks.extend(
        econbox_tile::OVERLAYS
            .into_iter()
            .filter_map(biome_pair)
            .flat_map(|pair| pair.iter()),
    );
    looks.extend(econbox_tile::WALLS.into_iter().map(wall));
    looks.extend([&BORDER_PIT, &BORDER_WATER, &BORDER_WATER_RUNUP]);
    looks
}
