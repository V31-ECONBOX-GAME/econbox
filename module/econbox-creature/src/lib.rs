//! The living things: what can be put on the map, where it can stand, and how it moves.

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use econbox_art::{Mark, SPRITE, Skin, Stance, atlas, sprite};
use econbox_camera::TILE_SIZE;
use econbox_clock::WorldClock;
use econbox_tile::Tile;
use econbox_tilemap::{TileMap, TilePos};

pub const CREATURE_LAYER: f32 = 1.0;
pub const WANDER_REACH: u32 = 6;
pub const STARVING: f32 = 0.0;
pub const DROWN_RATE: f32 = 8.0;
pub const BURN_RATE: f32 = 30.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Kind {
    Folk,
    Farm,
    Beast,
    Aquatic,
    Insect,
    Mystic,
    Monster,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Folk => "folk",
            Self::Farm => "farm",
            Self::Beast => "beast",
            Self::Aquatic => "aquatic",
            Self::Insect => "insect",
            Self::Mystic => "mystic",
            Self::Monster => "monster",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Species {
    Human,
    Elf,
    Orc,
    Dwarf,
    BloomFolk,
    CrystalFolk,
    CitrusFolk,
    BulbFolk,
    SugarFolk,
    AcidFolk,
    Cat,
    Dog,
    Chicken,
    Rabbit,
    Sheep,
    Cow,
    Fox,
    Monkey,
    Armadillo,
    Raccoon,
    Rat,
    Alpaca,
    Capybara,
    Goat,
    Penguin,
    Ostrich,
    BudWalker,
    CitrusSnail,
    GarlicSprite,
    ToastyBlob,
    Wolf,
    Bear,
    Boar,
    Rhino,
    Buffalo,
    Hyena,
    Scorpion,
    Snake,
    Crocodile,
    CrystalBlade,
    SandSpider,
    Crab,
    Frog,
    Fish,
    Turtle,
    Seal,
    Bee,
    Butterfly,
    Beetle,
    Grasshopper,
    Fly,
    BlueAnt,
    GreenAnt,
    BlackAnt,
    RedAnt,
    Fairy,
    Unicorn,
    LightMage,
    Druid,
    PlagueHealer,
    FrostWalker,
    WingedOne,
    SkyFinger,
    SnowFigure,
    Zombie,
    Ogre,
    Demon,
    Bandit,
    DarkMage,
    GraveCaller,
    Skeleton,
    Ghost,
    HoppingSkull,
    BurningSkull,
    FireSpirit,
    PumpkinImp,
    GrowthBeast,
    BiomassBlob,
    AcidBlob,
    Assimilator,
    SandWorm,
    Dragon,
    Visitor,
    Saucer,
    OddOne,
}

pub const SPECIES: [Species; 85] = [
    Species::Human,
    Species::Elf,
    Species::Orc,
    Species::Dwarf,
    Species::BloomFolk,
    Species::CrystalFolk,
    Species::CitrusFolk,
    Species::BulbFolk,
    Species::SugarFolk,
    Species::AcidFolk,
    Species::Cat,
    Species::Dog,
    Species::Chicken,
    Species::Rabbit,
    Species::Sheep,
    Species::Cow,
    Species::Fox,
    Species::Monkey,
    Species::Armadillo,
    Species::Raccoon,
    Species::Rat,
    Species::Alpaca,
    Species::Capybara,
    Species::Goat,
    Species::Penguin,
    Species::Ostrich,
    Species::BudWalker,
    Species::CitrusSnail,
    Species::GarlicSprite,
    Species::ToastyBlob,
    Species::Wolf,
    Species::Bear,
    Species::Boar,
    Species::Rhino,
    Species::Buffalo,
    Species::Hyena,
    Species::Scorpion,
    Species::Snake,
    Species::Crocodile,
    Species::CrystalBlade,
    Species::SandSpider,
    Species::Crab,
    Species::Frog,
    Species::Fish,
    Species::Turtle,
    Species::Seal,
    Species::Bee,
    Species::Butterfly,
    Species::Beetle,
    Species::Grasshopper,
    Species::Fly,
    Species::BlueAnt,
    Species::GreenAnt,
    Species::BlackAnt,
    Species::RedAnt,
    Species::Fairy,
    Species::Unicorn,
    Species::LightMage,
    Species::Druid,
    Species::PlagueHealer,
    Species::FrostWalker,
    Species::WingedOne,
    Species::SkyFinger,
    Species::SnowFigure,
    Species::Zombie,
    Species::Ogre,
    Species::Demon,
    Species::Bandit,
    Species::DarkMage,
    Species::GraveCaller,
    Species::Skeleton,
    Species::Ghost,
    Species::HoppingSkull,
    Species::BurningSkull,
    Species::FireSpirit,
    Species::PumpkinImp,
    Species::GrowthBeast,
    Species::BiomassBlob,
    Species::AcidBlob,
    Species::Assimilator,
    Species::SandWorm,
    Species::Dragon,
    Species::Visitor,
    Species::Saucer,
    Species::OddOne,
];

impl Species {
    pub fn name(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Elf => "elf",
            Self::Orc => "orc",
            Self::Dwarf => "dwarf",
            Self::BloomFolk => "civ_bloom",
            Self::CrystalFolk => "civ_crystal",
            Self::CitrusFolk => "civ_citrus",
            Self::BulbFolk => "civ_bulb",
            Self::SugarFolk => "civ_sugar",
            Self::AcidFolk => "civ_acid",
            Self::Cat => "cat",
            Self::Dog => "dog",
            Self::Chicken => "chicken",
            Self::Rabbit => "rabbit",
            Self::Sheep => "sheep",
            Self::Cow => "cow",
            Self::Fox => "fox",
            Self::Monkey => "monkey",
            Self::Armadillo => "armadillo",
            Self::Raccoon => "raccoon",
            Self::Rat => "rat",
            Self::Alpaca => "alpaca",
            Self::Capybara => "capybara",
            Self::Goat => "goat",
            Self::Penguin => "penguin",
            Self::Ostrich => "ostrich",
            Self::BudWalker => "bud_walker",
            Self::CitrusSnail => "citrus_snail",
            Self::GarlicSprite => "garlic_sprite",
            Self::ToastyBlob => "toasty_blob",
            Self::Wolf => "wolf",
            Self::Bear => "bear",
            Self::Boar => "boar",
            Self::Rhino => "rhino",
            Self::Buffalo => "buffalo",
            Self::Hyena => "hyena",
            Self::Scorpion => "scorpion",
            Self::Snake => "snake",
            Self::Crocodile => "crocodile",
            Self::CrystalBlade => "crystal_blade",
            Self::SandSpider => "sand_spider",
            Self::Crab => "crab",
            Self::Frog => "frog",
            Self::Fish => "fish",
            Self::Turtle => "turtle",
            Self::Seal => "seal",
            Self::Bee => "bee",
            Self::Butterfly => "butterfly",
            Self::Beetle => "beetle",
            Self::Grasshopper => "grasshopper",
            Self::Fly => "fly",
            Self::BlueAnt => "ant_blue",
            Self::GreenAnt => "ant_green",
            Self::BlackAnt => "ant_black",
            Self::RedAnt => "ant_red",
            Self::Fairy => "fairy",
            Self::Unicorn => "unicorn",
            Self::LightMage => "light_mage",
            Self::Druid => "druid",
            Self::PlagueHealer => "plague_healer",
            Self::FrostWalker => "frost_walker",
            Self::WingedOne => "winged_one",
            Self::SkyFinger => "sky_finger",
            Self::SnowFigure => "snow_figure",
            Self::Zombie => "zombie",
            Self::Ogre => "ogre",
            Self::Demon => "demon",
            Self::Bandit => "bandit",
            Self::DarkMage => "dark_mage",
            Self::GraveCaller => "grave_caller",
            Self::Skeleton => "skeleton",
            Self::Ghost => "ghost",
            Self::HoppingSkull => "hopping_skull",
            Self::BurningSkull => "burning_skull",
            Self::FireSpirit => "fire_spirit",
            Self::PumpkinImp => "pumpkin_imp",
            Self::GrowthBeast => "growth_beast",
            Self::BiomassBlob => "biomass_blob",
            Self::AcidBlob => "acid_blob",
            Self::Assimilator => "assimilator",
            Self::SandWorm => "sand_worm",
            Self::Dragon => "dragon",
            Self::Visitor => "visitor",
            Self::Saucer => "saucer",
            Self::OddOne => "odd_one",
        }
    }

    pub fn kind(self) -> Kind {
        match self {
            Self::Human
            | Self::Elf
            | Self::Orc
            | Self::Dwarf
            | Self::BloomFolk
            | Self::CrystalFolk
            | Self::CitrusFolk
            | Self::BulbFolk
            | Self::SugarFolk
            | Self::AcidFolk => Kind::Folk,
            Self::Cat
            | Self::Dog
            | Self::Chicken
            | Self::Rabbit
            | Self::Sheep
            | Self::Cow
            | Self::Fox
            | Self::Monkey
            | Self::Armadillo
            | Self::Raccoon
            | Self::Rat
            | Self::Alpaca
            | Self::Capybara
            | Self::Goat
            | Self::Penguin
            | Self::Ostrich
            | Self::BudWalker
            | Self::CitrusSnail
            | Self::GarlicSprite
            | Self::ToastyBlob => Kind::Farm,
            Self::Wolf
            | Self::Bear
            | Self::Boar
            | Self::Rhino
            | Self::Buffalo
            | Self::Hyena
            | Self::Scorpion
            | Self::Snake
            | Self::Crocodile
            | Self::CrystalBlade
            | Self::SandSpider => Kind::Beast,
            Self::Crab | Self::Frog | Self::Fish | Self::Turtle | Self::Seal => Kind::Aquatic,
            Self::Bee
            | Self::Butterfly
            | Self::Beetle
            | Self::Grasshopper
            | Self::Fly
            | Self::BlueAnt
            | Self::GreenAnt
            | Self::BlackAnt
            | Self::RedAnt => Kind::Insect,
            Self::Fairy
            | Self::Unicorn
            | Self::LightMage
            | Self::Druid
            | Self::PlagueHealer
            | Self::FrostWalker
            | Self::WingedOne
            | Self::SkyFinger
            | Self::SnowFigure => Kind::Mystic,
            Self::Zombie
            | Self::Ogre
            | Self::Demon
            | Self::Bandit
            | Self::DarkMage
            | Self::GraveCaller
            | Self::Skeleton
            | Self::Ghost
            | Self::HoppingSkull
            | Self::BurningSkull
            | Self::FireSpirit
            | Self::PumpkinImp
            | Self::GrowthBeast
            | Self::BiomassBlob
            | Self::AcidBlob
            | Self::Assimilator
            | Self::SandWorm
            | Self::Dragon
            | Self::Visitor
            | Self::Saucer
            | Self::OddOne => Kind::Monster,
        }
    }

    pub fn swims(self) -> bool {
        matches!(
            self,
            Self::Fish
                | Self::Crab
                | Self::Frog
                | Self::Turtle
                | Self::Seal
                | Self::Crocodile
                | Self::Snake
                | Self::Penguin
        )
    }

    pub fn walks(self) -> bool {
        !matches!(self, Self::Fish)
    }

    pub fn flies(self) -> bool {
        matches!(
            self,
            Self::Bee
                | Self::Butterfly
                | Self::Fly
                | Self::Fairy
                | Self::Dragon
                | Self::WingedOne
                | Self::Saucer
                | Self::Ghost
        )
    }

    pub fn stands_on(self, tile: Tile) -> bool {
        if tile.is_lava() {
            false
        } else if tile.is_water() {
            self.swims() || self.flies()
        } else if tile.blocks() {
            self.flies()
        } else {
            self.walks()
        }
    }

    pub fn health(self) -> f32 {
        match (self, self.kind()) {
            (Self::Dragon, _) => 300.0,
            (_, Kind::Folk) => 26.0,
            (_, Kind::Farm) => 14.0,
            (_, Kind::Beast) => 40.0,
            (_, Kind::Aquatic) => 10.0,
            (_, Kind::Insect) => 4.0,
            (_, Kind::Mystic) => 30.0,
            (_, Kind::Monster) => 60.0,
        }
    }

    pub fn speed(self) -> f32 {
        match self {
            Self::Rabbit
            | Self::Fox
            | Self::Cat
            | Self::Rat
            | Self::Ostrich
            | Self::Hyena
            | Self::Bee
            | Self::Butterfly
            | Self::Fly
            | Self::Fairy
            | Self::Grasshopper => 26.0,
            Self::Wolf
            | Self::Dog
            | Self::Fish
            | Self::Seal
            | Self::Dragon
            | Self::Unicorn
            | Self::Saucer
            | Self::WingedOne
            | Self::FireSpirit
            | Self::Ghost => 22.0,
            Self::Human
            | Self::Elf
            | Self::Orc
            | Self::Dwarf
            | Self::Monkey
            | Self::BloomFolk
            | Self::CrystalFolk
            | Self::CitrusFolk
            | Self::BulbFolk
            | Self::SugarFolk
            | Self::AcidFolk
            | Self::Bandit
            | Self::DarkMage
            | Self::LightMage
            | Self::Druid
            | Self::PlagueHealer
            | Self::FrostWalker
            | Self::GraveCaller
            | Self::SkyFinger
            | Self::Skeleton
            | Self::Visitor
            | Self::OddOne
            | Self::Assimilator
            | Self::Alpaca
            | Self::Goat
            | Self::Raccoon
            | Self::Snake
            | Self::Scorpion
            | Self::CrystalBlade
            | Self::SandSpider
            | Self::BudWalker
            | Self::BlueAnt
            | Self::GreenAnt
            | Self::BlackAnt
            | Self::RedAnt => 16.0,
            Self::Bear
            | Self::Boar
            | Self::Ogre
            | Self::Demon
            | Self::Rhino
            | Self::Buffalo
            | Self::Crocodile
            | Self::GrowthBeast
            | Self::SandWorm
            | Self::HoppingSkull
            | Self::BurningSkull
            | Self::PumpkinImp
            | Self::Beetle => 13.0,
            _ => 9.0,
        }
    }

    pub fn size(self) -> f32 {
        match (self, self.kind()) {
            (Self::Dragon, _) => 9.0,
            (_, Kind::Beast | Kind::Monster) => 6.0,
            (_, Kind::Folk | Kind::Mystic) => 5.0,
            (_, Kind::Insect) => 3.0,
            _ => 4.0,
        }
    }

    pub fn stance(self) -> Stance {
        match self {
            Self::Human
            | Self::Elf
            | Self::Orc
            | Self::Dwarf
            | Self::BloomFolk
            | Self::CrystalFolk
            | Self::CitrusFolk
            | Self::BulbFolk
            | Self::SugarFolk
            | Self::AcidFolk
            | Self::BudWalker
            | Self::CrystalBlade
            | Self::LightMage
            | Self::Druid
            | Self::PlagueHealer
            | Self::FrostWalker
            | Self::WingedOne
            | Self::SkyFinger
            | Self::SnowFigure
            | Self::Zombie
            | Self::Ogre
            | Self::Demon
            | Self::Bandit
            | Self::DarkMage
            | Self::GraveCaller
            | Self::Skeleton
            | Self::Ghost
            | Self::FireSpirit
            | Self::Assimilator
            | Self::Visitor
            | Self::OddOne => Stance::Upright,
            Self::Chicken
            | Self::Penguin
            | Self::Ostrich
            | Self::Bee
            | Self::Butterfly
            | Self::Fly
            | Self::Fairy => Stance::Bird,
            Self::Armadillo
            | Self::CitrusSnail
            | Self::GarlicSprite
            | Self::ToastyBlob
            | Self::Scorpion
            | Self::SandSpider
            | Self::Crab
            | Self::Frog
            | Self::Turtle
            | Self::Beetle
            | Self::Grasshopper
            | Self::BlueAnt
            | Self::GreenAnt
            | Self::BlackAnt
            | Self::RedAnt
            | Self::HoppingSkull
            | Self::BurningSkull
            | Self::PumpkinImp
            | Self::GrowthBeast
            | Self::BiomassBlob
            | Self::AcidBlob
            | Self::Saucer => Stance::Squat,
            Self::Snake | Self::Crocodile | Self::Fish | Self::Seal | Self::SandWorm => {
                Stance::Swimmer
            }
            _ => Stance::Quadruped,
        }
    }

    pub fn mark(self) -> Mark {
        match self {
            Self::Elf | Self::Rabbit | Self::Alpaca => Mark::LongEars,
            Self::Orc
            | Self::Cow
            | Self::Goat
            | Self::Boar
            | Self::Rhino
            | Self::Buffalo
            | Self::Unicorn
            | Self::Ogre
            | Self::Demon
            | Self::Dragon => Mark::Horns,
            _ => Mark::Plain,
        }
    }

    pub fn skin(self) -> Skin {
        Skin {
            coat: self.color(),
            accent: self.accent(),
        }
    }

    pub fn accent(self) -> [u8; 3] {
        match self {
            Self::Human => [0x46, 0x6a, 0x9e],
            Self::Elf => [0x2f, 0x6b, 0x45],
            Self::Orc => [0x6b, 0x3a, 0x2c],
            Self::Dwarf => [0x8e, 0x4a, 0x2a],
            Self::BloomFolk => [0x3c, 0xb0, 0x5c],
            Self::CrystalFolk => [0x3a, 0x8c, 0xd8],
            Self::CitrusFolk => [0x5c, 0xb0, 0x48],
            Self::BulbFolk => [0x8c, 0xd8, 0x3c],
            Self::SugarFolk => [0xf0, 0xf2, 0xf6],
            Self::AcidFolk => [0x3a, 0x3c, 0x42],
            Self::Chicken => [0xe2, 0x8a, 0x22],
            Self::Armadillo => [0x7a, 0x5a, 0x3c],
            Self::Raccoon => [0x3a, 0x3c, 0x42],
            Self::Rat => [0xe8, 0x9c, 0xa8],
            Self::Alpaca => [0xc8, 0xa0, 0x6c],
            Self::Capybara => [0x6a, 0x48, 0x30],
            Self::Goat => [0x7a, 0x5a, 0x3c],
            Self::Penguin => [0xf0, 0xb4, 0x3a],
            Self::Ostrich => [0xe8, 0xb0, 0x9c],
            Self::BudWalker => [0x5c, 0xb0, 0x48],
            Self::CitrusSnail => [0x8a, 0x5c, 0x32],
            Self::GarlicSprite => [0x5c, 0xb0, 0x48],
            Self::ToastyBlob => [0x6a, 0x48, 0x30],
            Self::Rhino => [0xe8, 0xe0, 0xc8],
            Self::Buffalo => [0xe8, 0xe0, 0xc8],
            Self::Hyena => [0x6a, 0x48, 0x30],
            Self::Scorpion => [0xf0, 0xb4, 0x3a],
            Self::Snake => [0xf0, 0xb4, 0x3a],
            Self::Crocodile => [0xe8, 0xe0, 0xc8],
            Self::CrystalBlade => [0xc2, 0xc8, 0xd2],
            Self::SandSpider => [0x6a, 0x48, 0x30],
            Self::Crab => [0x8e, 0x22, 0x1c],
            Self::Frog => [0xd8, 0xe0, 0x8a],
            Self::Fish => [0x2c, 0x62, 0x90],
            Self::Turtle => [0xb0, 0x90, 0x58],
            Self::Seal => [0x3a, 0x3c, 0x42],
            Self::Bee => [0x3a, 0x3c, 0x42],
            Self::Butterfly => [0x6a, 0xe0, 0xe8],
            Self::Beetle => [0x6a, 0xe0, 0xe8],
            Self::Grasshopper => [0x4c, 0x7c, 0x3c],
            Self::Fly => [0xc8, 0xe8, 0xf4],
            Self::BlueAnt => [0x2c, 0x30, 0x3a],
            Self::GreenAnt => [0x2c, 0x30, 0x3a],
            Self::BlackAnt => [0x8a, 0x8c, 0x90],
            Self::RedAnt => [0x2c, 0x30, 0x3a],
            Self::Fairy => [0x6a, 0xe0, 0xe8],
            Self::Unicorn => [0xd0, 0x6c, 0xe8],
            Self::LightMage => [0xf0, 0xf2, 0xf6],
            Self::Druid => [0x5c, 0xb0, 0x48],
            Self::PlagueHealer => [0xe8, 0xe0, 0xc8],
            Self::FrostWalker => [0x3a, 0x8c, 0xd8],
            Self::WingedOne => [0xf2, 0xc2, 0x3a],
            Self::SkyFinger => [0xf2, 0xc2, 0x3a],
            Self::SnowFigure => [0xf4, 0x7c, 0x2c],
            Self::Zombie => [0x46, 0x52, 0x38],
            Self::Ogre => [0x53, 0x42, 0x2c],
            Self::Demon => [0x2a, 0x12, 0x1e],
            Self::Bandit => [0x3a, 0x3c, 0x42],
            Self::DarkMage => [0x6a, 0x3c, 0x58],
            Self::GraveCaller => [0x3a, 0x3c, 0x42],
            Self::Skeleton => [0x7c, 0x78, 0x74],
            Self::Ghost => [0x7c, 0x86, 0x90],
            Self::HoppingSkull => [0x3a, 0x3c, 0x42],
            Self::BurningSkull => [0xe8, 0xe8, 0xe0],
            Self::FireSpirit => [0xf2, 0xc2, 0x3a],
            Self::PumpkinImp => [0x4c, 0x7c, 0x3c],
            Self::GrowthBeast => [0x9c, 0x36, 0x26],
            Self::BiomassBlob => [0xd8, 0x6c, 0x7c],
            Self::AcidBlob => [0x4c, 0x7c, 0x3c],
            Self::Assimilator => [0x0a, 0xc0, 0xeb],
            Self::SandWorm => [0x9c, 0x5c, 0x48],
            Self::Dragon => [0xf2, 0xc2, 0x3a],
            Self::Visitor => [0x0a, 0xc0, 0xeb],
            Self::Saucer => [0x0a, 0xc0, 0xeb],
            Self::OddOne => [0xf2, 0xc2, 0x3a],
            _ => shade_up(self.color()),
        }
    }

    pub fn slot(self) -> usize {
        self as usize
    }

    pub fn color(self) -> [u8; 3] {
        match self {
            Self::Human => [0xe6, 0xb2, 0x8a],
            Self::Elf => [0x9f, 0xe0, 0xa8],
            Self::Orc => [0x5f, 0x9e, 0x4a],
            Self::Dwarf => [0xc8, 0x7d, 0x3f],
            Self::BloomFolk => [0xe8, 0x7c, 0xb0],
            Self::CrystalFolk => [0x6a, 0xe0, 0xe8],
            Self::CitrusFolk => [0xf0, 0xe0, 0x48],
            Self::BulbFolk => [0xe8, 0xe0, 0xc8],
            Self::SugarFolk => [0xf0, 0x8c, 0xc8],
            Self::AcidFolk => [0x8c, 0xd8, 0x3c],
            Self::Cat => [0xf0, 0xa6, 0x4b],
            Self::Dog => [0x9c, 0x6b, 0x42],
            Self::Chicken => [0xf2, 0xf0, 0xe4],
            Self::Rabbit => [0xe8, 0xe8, 0xf0],
            Self::Sheep => [0xd8, 0xd4, 0xc6],
            Self::Cow => [0xb0, 0xb4, 0xbc],
            Self::Fox => [0xe0, 0x72, 0x2c],
            Self::Monkey => [0x8a, 0x5a, 0x38],
            Self::Armadillo => [0xb0, 0x90, 0x70],
            Self::Raccoon => [0x8a, 0x8c, 0x90],
            Self::Rat => [0x7c, 0x78, 0x74],
            Self::Alpaca => [0xf0, 0xe8, 0xd8],
            Self::Capybara => [0xa8, 0x78, 0x48],
            Self::Goat => [0xe0, 0xdc, 0xd0],
            Self::Penguin => [0x2c, 0x30, 0x3a],
            Self::Ostrich => [0x4a, 0x40, 0x3c],
            Self::BudWalker => [0xe8, 0x7c, 0xb0],
            Self::CitrusSnail => [0xf0, 0xe0, 0x48],
            Self::GarlicSprite => [0xe8, 0xe0, 0xc8],
            Self::ToastyBlob => [0xe8, 0xd0, 0xa8],
            Self::Wolf => [0x77, 0x7f, 0x92],
            Self::Bear => [0x6b, 0x46, 0x2a],
            Self::Boar => [0x55, 0x45, 0x3c],
            Self::Rhino => [0x9a, 0x9c, 0x98],
            Self::Buffalo => [0x6a, 0x48, 0x30],
            Self::Hyena => [0xc8, 0xa0, 0x6c],
            Self::Scorpion => [0x9a, 0x4c, 0x2c],
            Self::Snake => [0x7c, 0xa8, 0x3c],
            Self::Crocodile => [0x4c, 0x7c, 0x3c],
            Self::CrystalBlade => [0x6a, 0xe0, 0xe8],
            Self::SandSpider => [0xe0, 0xc0, 0x7c],
            Self::Crab => [0xd8, 0x3c, 0x32],
            Self::Frog => [0x5c, 0xc0, 0x4a],
            Self::Fish => [0x4a, 0x9c, 0xd8],
            Self::Turtle => [0x5c, 0x8c, 0x48],
            Self::Seal => [0x8a, 0x94, 0xa0],
            Self::Bee => [0xf2, 0xc2, 0x3a],
            Self::Butterfly => [0xd0, 0x6c, 0xe8],
            Self::Beetle => [0x3c, 0x48, 0x9c],
            Self::Grasshopper => [0x9c, 0xc8, 0x48],
            Self::Fly => [0x4a, 0x4c, 0x56],
            Self::BlueAnt => [0x3a, 0x8c, 0xd8],
            Self::GreenAnt => [0x5c, 0xb0, 0x48],
            Self::BlackAnt => [0x3a, 0x3c, 0x42],
            Self::RedAnt => [0xd8, 0x3c, 0x34],
            Self::Fairy => [0xf0, 0xd0, 0xf0],
            Self::Unicorn => [0xf4, 0xf0, 0xf8],
            Self::LightMage => [0xe6, 0xb2, 0x8a],
            Self::Druid => [0xe6, 0xb2, 0x8a],
            Self::PlagueHealer => [0x3a, 0x3c, 0x42],
            Self::FrostWalker => [0xc8, 0xe8, 0xf4],
            Self::WingedOne => [0xf0, 0xf2, 0xf6],
            Self::SkyFinger => [0xf0, 0xc0, 0x96],
            Self::SnowFigure => [0xf4, 0xf8, 0xff],
            Self::Zombie => [0x7a, 0x9c, 0x5a],
            Self::Ogre => [0x86, 0x6e, 0x48],
            Self::Demon => [0xb0, 0x2c, 0x50],
            Self::Bandit => [0xe6, 0xb2, 0x8a],
            Self::DarkMage => [0xe6, 0xb2, 0x8a],
            Self::GraveCaller => [0xc8, 0xd0, 0xb8],
            Self::Skeleton => [0xe8, 0xe8, 0xe0],
            Self::Ghost => [0xd8, 0xe8, 0xf0],
            Self::HoppingSkull => [0xe8, 0xe8, 0xe0],
            Self::BurningSkull => [0xf4, 0x7c, 0x2c],
            Self::FireSpirit => [0xf4, 0x7c, 0x2c],
            Self::PumpkinImp => [0xd8, 0x8a, 0x2c],
            Self::GrowthBeast => [0xd8, 0x6c, 0x7c],
            Self::BiomassBlob => [0x9c, 0x36, 0x26],
            Self::AcidBlob => [0x8c, 0xd8, 0x3c],
            Self::Assimilator => [0x7c, 0x86, 0x90],
            Self::SandWorm => [0xe0, 0xa8, 0x8c],
            Self::Dragon => [0xc8, 0x3c, 0x30],
            Self::Visitor => [0x8c, 0xd8, 0x3c],
            Self::Saucer => [0xc2, 0xc8, 0xd2],
            Self::OddOne => [0x9a, 0xa0, 0xa8],
        }
    }
}

fn shade_up(color: [u8; 3]) -> [u8; 3] {
    color.map(|channel| (f32::from(channel) * 1.35).clamp(0.0, 255.0) as u8)
}

#[derive(Resource, Debug, Clone)]
pub struct CreatureArt {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Creature {
    pub species: Species,
    pub health: f32,
    pub age: f32,
    pub food: f32,
}

impl Creature {
    pub fn new(species: Species) -> Self {
        Self {
            species,
            health: species.health(),
            age: 0.0,
            food: 1.0,
        }
    }

    pub fn alive(&self) -> bool {
        self.health > 0.0
    }

    pub fn starving(&self) -> bool {
        self.food <= STARVING
    }

    pub fn suffer(&mut self, effect: Effect) {
        match effect {
            Effect::Kill => self.health = 0.0,
            Effect::Hurt(amount) => self.health -= amount,
            Effect::Heal => {
                self.health = self.species.health();
                self.food = 1.0;
            }
            Effect::Weaken => self.health *= 0.5,
            Effect::Purge if self.species.kind() == Kind::Monster => self.health = 0.0,
            Effect::Purge => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Effect {
    Kill,
    Hurt(f32),
    Heal,
    Weaken,
    Purge,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct Strike {
    pub centre: TilePos,
    pub reach: f32,
    pub effect: Effect,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Wander {
    pub target: Vec2,
    pub rest: f32,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct Spawn {
    pub species: Species,
    pub pos: TilePos,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct Census {
    pub alive: usize,
    pub deaths: u64,
    pub per_species: [usize; SPECIES.len()],
}

impl Default for Census {
    fn default() -> Self {
        Self {
            alive: 0,
            deaths: 0,
            per_species: [0; SPECIES.len()],
        }
    }
}

impl Census {
    pub fn of(&self, species: Species) -> usize {
        self.per_species[species.slot()]
    }

    pub fn of_kind(&self, kind: Kind) -> usize {
        SPECIES
            .iter()
            .filter(|species| species.kind() == kind)
            .map(|species| self.of(*species))
            .sum()
    }
}

pub fn centre_of(map: &TileMap, pos: TilePos) -> Vec2 {
    let half = Vec2::new(map.width as f32, map.height as f32) * TILE_SIZE * 0.5;

    Vec2::new(
        -half.x + (pos.x as f32 + 0.5) * TILE_SIZE,
        half.y - (pos.y as f32 + 0.5) * TILE_SIZE,
    )
}

pub fn tile_of(map: &TileMap, world: Vec2) -> Option<TilePos> {
    let half = Vec2::new(map.width as f32, map.height as f32) * TILE_SIZE * 0.5;
    let local = world + half;

    if local.x < 0.0 || local.y < 0.0 {
        return None;
    }

    let x = (local.x / TILE_SIZE) as u32;
    let y = map
        .height
        .checked_sub(1 + (local.y / TILE_SIZE) as u32)
        .unwrap_or(u32::MAX);

    (x < map.width && y < map.height).then_some(TilePos::new(x, y))
}

fn drift(seed: u32) -> Vec2 {
    let mut h = seed.wrapping_mul(0x9e37_79b9);
    h ^= h >> 15;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;

    let angle = (h % 3600) as f32 * std::f32::consts::TAU / 3600.0;

    Vec2::new(angle.cos(), angle.sin())
}

pub fn sheet() -> Image {
    let frames: Vec<_> = SPECIES
        .iter()
        .map(|species| sprite(species.stance(), species.mark(), species.skin()))
        .collect();
    let (data, width, height) = atlas(&frames);

    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        mipmap_filter: ImageFilterMode::Nearest,
        ..default()
    });

    image
}

pub struct CreaturePlugin;

impl Plugin for CreaturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Census>()
            .add_message::<Spawn>()
            .add_message::<Strike>()
            .add_systems(Startup, draw)
            .add_systems(
                Update,
                (place, roam, live, smite, strand, reap, count)
                    .chain()
                    .run_if(resource_exists::<TileMap>),
            );
    }
}

fn draw(
    mut commands: Commands,
    images: Option<ResMut<Assets<Image>>>,
    layouts: Option<ResMut<Assets<TextureAtlasLayout>>>,
) {
    let (Some(mut images), Some(mut layouts)) = (images, layouts) else {
        return;
    };

    commands.insert_resource(CreatureArt {
        image: images.add(sheet()),
        layout: layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(SPRITE as u32),
            SPECIES.len() as u32,
            1,
            None,
            None,
        )),
    });
}

fn place(
    mut commands: Commands,
    mut asked: MessageReader<Spawn>,
    map: Res<TileMap>,
    art: Option<Res<CreatureArt>>,
) {
    for request in asked.read() {
        let Some(tile) = map.get(request.pos) else {
            continue;
        };

        if !request.species.stands_on(tile) {
            continue;
        }

        let at = centre_of(&map, request.pos);
        let [r, g, b] = request.species.color();
        let size = request.species.size();

        let look = match &art {
            Some(art) => Sprite {
                image: art.image.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: art.layout.clone(),
                    index: request.species.slot(),
                }),
                custom_size: Some(Vec2::splat(size * 2.0)),
                ..default()
            },
            None => Sprite {
                color: Color::srgb_u8(r, g, b),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
        };

        commands.spawn((
            Creature::new(request.species),
            Wander {
                target: at,
                rest: 0.0,
            },
            look,
            Transform::from_xyz(at.x, at.y, CREATURE_LAYER),
        ));
    }
}

fn roam(
    time: Res<Time>,
    clock: Res<WorldClock>,
    map: Res<TileMap>,
    mut wanderers: Query<(&Creature, &mut Wander, &mut Transform)>,
) {
    let rate = clock.rate();

    if rate == 0.0 {
        return;
    }

    let step = time.delta_secs() * rate;

    for (creature, mut wander, mut transform) in &mut wanderers {
        let here = transform.translation.truncate();
        wander.rest -= step;

        if wander.rest <= 0.0 || here.distance(wander.target) < TILE_SIZE * 0.5 {
            let seed = (here.x.to_bits() ^ here.y.to_bits()).wrapping_add(clock.ticks as u32);
            let reach = WANDER_REACH as f32 * TILE_SIZE;
            let wish = here + drift(seed) * reach;

            if let Some(pos) = tile_of(&map, wish)
                && map
                    .get(pos)
                    .is_some_and(|tile| creature.species.stands_on(tile))
            {
                wander.target = centre_of(&map, pos);
            }

            wander.rest = 1.0 + (seed % 200) as f32 / 100.0;
        }

        let toward = wander.target - here;

        if toward.length() > f32::EPSILON {
            let move_by = toward.normalize() * creature.species.speed() * step;
            let next = if move_by.length() > toward.length() {
                wander.target
            } else {
                here + move_by
            };

            let walled_in = tile_of(&map, here)
                .and_then(|pos| map.get(pos))
                .is_some_and(Tile::blocks);

            if walled_in
                || tile_of(&map, next)
                    .and_then(|pos| map.get(pos))
                    .is_some_and(|tile| creature.species.stands_on(tile))
            {
                transform.translation.x = next.x;
                transform.translation.y = next.y;
            } else {
                wander.rest = 0.0;
            }
        }
    }
}

fn live(time: Res<Time>, clock: Res<WorldClock>, mut creatures: Query<&mut Creature>) {
    let rate = clock.rate();

    if rate == 0.0 {
        return;
    }

    let step = time.delta_secs() * rate;

    for mut creature in &mut creatures {
        creature.age += step;
        creature.food = (creature.food - step * 0.01).max(0.0);

        if creature.starving() {
            creature.health -= step;
        }
    }
}

fn smite(
    mut asked: MessageReader<Strike>,
    map: Res<TileMap>,
    mut creatures: Query<(&mut Creature, &Transform)>,
) {
    for strike in asked.read() {
        let centre = centre_of(&map, strike.centre);
        let reach = (strike.reach + 0.5) * TILE_SIZE;

        for (mut creature, transform) in &mut creatures {
            if transform.translation.truncate().distance(centre) <= reach {
                creature.suffer(strike.effect);
            }
        }
    }
}

fn strand(
    time: Res<Time>,
    clock: Res<WorldClock>,
    map: Res<TileMap>,
    mut creatures: Query<(&mut Creature, &Transform)>,
) {
    let rate = clock.rate();

    if rate == 0.0 {
        return;
    }

    let step = time.delta_secs() * rate;

    for (mut creature, transform) in &mut creatures {
        let Some(tile) =
            tile_of(&map, transform.translation.truncate()).and_then(|pos| map.get(pos))
        else {
            continue;
        };

        if tile.is_lava() {
            creature.health -= BURN_RATE * step;
        } else if !tile.blocks() && !creature.species.stands_on(tile) {
            creature.health -= DROWN_RATE * step;
        }
    }
}

fn reap(mut commands: Commands, creatures: Query<(Entity, &Creature)>, mut census: ResMut<Census>) {
    for (entity, creature) in &creatures {
        if !creature.alive() {
            commands.entity(entity).despawn();
            census.deaths += 1;
        }
    }
}

fn count(creatures: Query<&Creature>, mut census: ResMut<Census>) {
    let mut per_species = [0; SPECIES.len()];

    for creature in &creatures {
        per_species[creature.species.slot()] += 1;
    }

    let alive = per_species.iter().sum();

    if census.alive != alive || census.per_species != per_species {
        census.alive = alive;
        census.per_species = per_species;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use econbox_tile::{Ground, Overlay, Wall};

    fn map() -> TileMap {
        TileMap::filled(64, 64, Tile::new(Ground::SoilLow, Overlay::Grass))
    }

    #[test]
    fn every_species_has_a_name_of_its_own() {
        let mut names: Vec<&str> = SPECIES.iter().map(|species| species.name()).collect();
        names.sort_unstable();
        let before = names.len();
        names.dedup();

        assert_eq!(names.len(), before);
        assert_eq!(before, SPECIES.len());
    }

    #[test]
    fn every_species_has_a_look_of_its_own() {
        let mut frames: Vec<_> = SPECIES
            .iter()
            .map(|species| sprite(species.stance(), species.mark(), species.skin()))
            .collect();
        frames.sort_unstable();
        frames.dedup();

        assert_eq!(frames.len(), SPECIES.len());
    }

    #[test]
    fn every_species_sits_in_the_sheet_at_its_own_slot() {
        for (at, species) in SPECIES.iter().enumerate() {
            assert_eq!(species.slot(), at, "{}", species.name());
        }

        let sheet = sheet();

        assert_eq!(sheet.width(), (SPECIES.len() * SPRITE) as u32);
        assert_eq!(sheet.height(), SPRITE as u32);
    }

    #[test]
    fn the_folk_are_the_ones_that_could_found_a_kingdom() {
        let folk: Vec<Species> = SPECIES
            .iter()
            .copied()
            .filter(|species| species.kind() == Kind::Folk)
            .collect();

        assert_eq!(
            folk,
            [
                Species::Human,
                Species::Elf,
                Species::Orc,
                Species::Dwarf,
                Species::BloomFolk,
                Species::CrystalFolk,
                Species::CitrusFolk,
                Species::BulbFolk,
                Species::SugarFolk,
                Species::AcidFolk,
            ]
        );
    }

    #[test]
    fn land_creatures_refuse_the_sea_and_fish_refuse_the_shore() {
        let grass = Tile::new(Ground::SoilLow, Overlay::Grass);
        let sea = Tile::bare(Ground::DeepOcean);

        assert!(Species::Human.stands_on(grass));
        assert!(!Species::Human.stands_on(sea));
        assert!(Species::Fish.stands_on(sea));
        assert!(!Species::Fish.stands_on(grass));
    }

    #[test]
    fn a_frog_is_at_home_on_both_sides_of_the_shore() {
        assert!(Species::Frog.stands_on(Tile::bare(Ground::ShallowWaters)));
        assert!(Species::Frog.stands_on(Tile::new(Ground::SoilLow, Overlay::Grass)));
    }

    #[test]
    fn shore_dwellers_are_at_home_in_the_sea_and_on_the_land() {
        let grass = Tile::new(Ground::SoilLow, Overlay::Grass);
        let sea = Tile::bare(Ground::DeepOcean);

        for species in [
            Species::Turtle,
            Species::Seal,
            Species::Crocodile,
            Species::Snake,
            Species::Penguin,
        ] {
            assert!(species.stands_on(grass), "{}", species.name());
            assert!(species.stands_on(sea), "{}", species.name());
        }
    }

    #[test]
    fn flyers_cross_the_open_sea() {
        let sea = Tile::bare(Ground::DeepOcean);
        let flyers: Vec<Species> = SPECIES
            .iter()
            .copied()
            .filter(|species| species.flies())
            .collect();

        assert!(flyers.contains(&Species::Dragon));
        assert!(flyers.contains(&Species::Bee));
        assert!(flyers.contains(&Species::Fairy));

        for species in flyers {
            assert!(species.stands_on(sea), "{}", species.name());
            assert!(species.walks(), "{}", species.name());
        }
    }

    #[test]
    fn walkers_that_neither_swim_nor_fly_keep_out_of_the_sea() {
        let sea = Tile::bare(Ground::ShallowWaters);

        for species in [
            Species::Rhino,
            Species::SandWorm,
            Species::BlueAnt,
            Species::Bandit,
            Species::Unicorn,
            Species::BloomFolk,
        ] {
            assert!(!species.stands_on(sea), "{}", species.name());
        }
    }

    #[test]
    fn every_species_has_somewhere_to_stand() {
        let grass = Tile::new(Ground::SoilLow, Overlay::Grass);
        let sea = Tile::bare(Ground::DeepOcean);

        for species in SPECIES {
            assert!(
                species.stands_on(grass) || species.stands_on(sea),
                "{}",
                species.name()
            );
        }
    }

    #[test]
    fn insects_are_the_smallest_and_frailest_of_all() {
        for species in SPECIES {
            if species.kind() != Kind::Insect {
                assert!(
                    species.health() > Species::Bee.health(),
                    "{}",
                    species.name()
                );
                assert!(
                    species.size() > Species::BlueAnt.size(),
                    "{}",
                    species.name()
                );
            }
        }
    }

    #[test]
    fn nothing_outlasts_or_outgrows_a_dragon() {
        for species in SPECIES {
            if species != Species::Dragon {
                assert!(
                    Species::Dragon.health() > species.health(),
                    "{}",
                    species.name()
                );
                assert!(
                    Species::Dragon.size() > species.size(),
                    "{}",
                    species.name()
                );
            }
        }
    }

    #[test]
    fn a_census_reads_back_each_species_and_kind_on_its_own() {
        let mut census = Census::default();
        census.per_species[Species::Dragon.slot()] = 2;
        census.per_species[Species::Skeleton.slot()] = 3;
        census.per_species[Species::RedAnt.slot()] = 5;
        census.per_species[Species::Human.slot()] = 1;

        assert_eq!(census.of(Species::Dragon), 2);
        assert_eq!(census.of(Species::RedAnt), 5);
        assert_eq!(census.of(Species::BlackAnt), 0);
        assert_eq!(census.of_kind(Kind::Monster), 5);
        assert_eq!(census.of_kind(Kind::Insect), 5);
        assert_eq!(census.of_kind(Kind::Folk), 1);
        assert_eq!(census.of_kind(Kind::Mystic), 0);
    }

    #[test]
    fn the_census_counts_one_of_every_species_dropped_on_its_ground() {
        let mut ground = map();
        ground.set(0, 0, Tile::bare(Ground::DeepOcean));

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(CreaturePlugin)
            .init_resource::<WorldClock>()
            .insert_resource(ground);
        app.update();

        for (at, species) in SPECIES.iter().enumerate() {
            let pos = if species.walks() {
                TilePos::new(1 + at as u32 % 60, 1 + at as u32 / 60)
            } else {
                TilePos::new(0, 0)
            };

            app.world_mut().write_message(Spawn {
                species: *species,
                pos,
            });
        }

        app.update();
        let census = app.world().resource::<Census>();

        assert_eq!(census.alive, SPECIES.len());

        for species in SPECIES {
            assert_eq!(census.of(species), 1, "{}", species.name());
        }

        for kind in [
            Kind::Folk,
            Kind::Farm,
            Kind::Beast,
            Kind::Aquatic,
            Kind::Insect,
            Kind::Mystic,
            Kind::Monster,
        ] {
            let members = SPECIES
                .iter()
                .filter(|species| species.kind() == kind)
                .count();

            assert_eq!(census.of_kind(kind), members, "{}", kind.name());
        }
    }

    #[test]
    fn a_wall_stops_walkers_and_swimmers_but_not_flyers() {
        let wall = Tile::new(Ground::SoilLow, Overlay::Grass).with_wall(Some(Wall::Order));

        for species in SPECIES {
            assert_eq!(
                species.stands_on(wall),
                species.flies(),
                "{}",
                species.name()
            );
        }
    }

    #[test]
    fn nothing_will_set_foot_on_lava() {
        let lava = Tile::bare(Ground::Lava3);

        for species in SPECIES {
            assert!(!species.stands_on(lava), "{}", species.name());
        }
    }

    #[test]
    fn a_kill_ends_any_creature_however_tough() {
        let mut demon = Creature::new(Species::Demon);
        demon.suffer(Effect::Kill);

        assert!(!demon.alive());
    }

    #[test]
    fn a_hurt_takes_exactly_what_it_says() {
        let mut ogre = Creature::new(Species::Ogre);
        ogre.suffer(Effect::Hurt(10.0));

        assert_eq!(ogre.health, Species::Ogre.health() - 10.0);
    }

    #[test]
    fn a_blessing_makes_a_creature_whole_and_fed_again() {
        let mut human = Creature::new(Species::Human);
        human.health = 1.0;
        human.food = 0.0;
        human.suffer(Effect::Heal);

        assert_eq!(human.health, Species::Human.health());
        assert!(!human.starving());
    }

    #[test]
    fn a_curse_halves_health_but_never_kills_on_its_own() {
        let mut cow = Creature::new(Species::Cow);

        for _ in 0..40 {
            cow.suffer(Effect::Weaken);
        }

        assert!(cow.alive());
        assert!(cow.health < 1e-3);
    }

    #[test]
    fn a_purge_takes_monsters_and_spares_everyone_else() {
        for species in SPECIES {
            let mut creature = Creature::new(species);
            creature.suffer(Effect::Purge);

            assert_eq!(
                creature.alive(),
                species.kind() != Kind::Monster,
                "{}",
                species.name()
            );
        }
    }

    #[test]
    fn a_fresh_creature_starts_whole_and_fed() {
        let creature = Creature::new(Species::Human);

        assert_eq!(creature.health, Species::Human.health());
        assert_eq!(creature.age, 0.0);
        assert!(creature.alive());
        assert!(!creature.starving());
    }

    #[test]
    fn a_monster_outlasts_a_rabbit() {
        assert!(Species::Demon.health() > Species::Rabbit.health());
        assert!(Species::Rabbit.speed() > Species::Demon.speed());
    }

    #[test]
    fn the_centre_of_a_tile_maps_back_to_that_tile() {
        let map = map();

        for (x, y) in [(0, 0), (31, 17), (63, 63)] {
            let pos = TilePos::new(x, y);

            assert_eq!(tile_of(&map, centre_of(&map, pos)), Some(pos), "{x},{y}");
        }
    }

    #[test]
    fn a_point_off_the_map_belongs_to_no_tile() {
        let map = map();
        let half = 64.0 * TILE_SIZE * 0.5;

        assert!(tile_of(&map, Vec2::new(-half - 1.0, 0.0)).is_none());
        assert!(tile_of(&map, Vec2::new(0.0, half + 1.0)).is_none());
    }

    #[test]
    fn a_wander_step_always_points_somewhere() {
        for seed in 0..64_u32 {
            let step = drift(seed);

            assert!((step.length() - 1.0).abs() < 1e-4, "{seed}");
        }
    }

    #[test]
    fn wandering_does_not_always_head_the_same_way() {
        let mut seen: Vec<(i32, i32)> = (0..32_u32)
            .map(|seed| {
                let step = drift(seed) * 100.0;
                (step.x as i32, step.y as i32)
            })
            .collect();
        seen.sort_unstable();
        seen.dedup();

        assert!(seen.len() > 8, "only {} directions", seen.len());
    }

    #[test]
    fn a_starving_creature_loses_health_until_it_dies() {
        let mut creature = Creature::new(Species::Rabbit);
        creature.food = 0.0;

        assert!(creature.starving());

        creature.health = 0.0;
        assert!(!creature.alive());
    }
}
