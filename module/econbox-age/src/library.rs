use crate::Age;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Overlay {
    Chaos,
    Moon,
    Magic,
    Sun,
    RainDarkness,
    Winter,
    Ash,
    Night,
    Rain,
}

pub const OVERLAYS: [Overlay; 9] = [
    Overlay::Chaos,
    Overlay::Moon,
    Overlay::Magic,
    Overlay::Sun,
    Overlay::RainDarkness,
    Overlay::Winter,
    Overlay::Ash,
    Overlay::Night,
    Overlay::Rain,
];

impl Overlay {
    pub fn id(self) -> &'static str {
        match self {
            Self::Chaos => "chaos",
            Self::Moon => "moon",
            Self::Magic => "magic",
            Self::Sun => "sun",
            Self::RainDarkness => "rain_darkness",
            Self::Winter => "winter",
            Self::Ash => "ash",
            Self::Night => "night",
            Self::Rain => "rain",
        }
    }

    pub fn index(self) -> usize {
        OVERLAYS
            .iter()
            .position(|overlay| *overlay == self)
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Particles {
    Snow,
    Rain,
    Magic,
    Ash,
    Sun,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AgeAsset {
    pub age: Age,
    pub rate: u32,
    pub years_min: u32,
    pub years_max: u32,
    pub overlay_alpha: f32,
    pub light_alpha_game: f32,
    pub light_alpha_minimap: f32,
    pub darkness: bool,
    pub overlays: &'static [Overlay],
    pub particles: &'static [Particles],
    pub freeze_world: bool,
    pub unfreeze_world: bool,
    pub unfreeze_mountains: bool,
    pub clouds: &'static [(&'static str, u32)],
    pub biomes: &'static [&'static str],
    pub cloud_interval: f32,
    pub range_weapons_multiplier: f32,
    pub temperature_damage_bonus: i32,
    pub conditions: &'static [Age],
    pub force_next: Option<Age>,
    pub crops_grow: bool,
    pub snow_turns_babies_into_ice_ones: bool,
    pub fire_elemental_spawn_on_fire: bool,
    pub fire_elemental_spawn_chance: f32,
    pub rage_brings_demons: bool,
    pub light_age: bool,
    pub chaos: bool,
    pub winter: bool,
    pub moon: bool,
    pub night: bool,
    pub light_damage: bool,
    pub light_color: [u8; 3],
    pub title_color: [u8; 3],
    pub bonus_loyalty: i32,
    pub bonus_opinion: i32,
    pub bonus_biomes_growth: i32,
    pub grow_vegetation: bool,
    pub fire_spread_rate_bonus: f32,
    pub special_effect_interval: f32,
    pub default_slots: &'static [usize],
    pub link_default_slots: bool,
}

pub const CLOUD_INTERVAL: f32 = 15.0;
pub const LIGHT_COLOR: [u8; 3] = [0xff, 0xce, 0x61];

const BASE: AgeAsset = AgeAsset {
    age: Age::Unknown,
    rate: 1,
    years_min: 35,
    years_max: 55,
    overlay_alpha: 0.2,
    light_alpha_game: 1.0,
    light_alpha_minimap: 1.0,
    darkness: false,
    overlays: &[],
    particles: &[],
    freeze_world: false,
    unfreeze_world: false,
    unfreeze_mountains: false,
    clouds: &[],
    biomes: &[],
    cloud_interval: CLOUD_INTERVAL,
    range_weapons_multiplier: 1.0,
    temperature_damage_bonus: 1,
    conditions: &[],
    force_next: None,
    crops_grow: true,
    snow_turns_babies_into_ice_ones: false,
    fire_elemental_spawn_on_fire: false,
    fire_elemental_spawn_chance: 0.0,
    rage_brings_demons: false,
    light_age: false,
    chaos: false,
    winter: false,
    moon: false,
    night: false,
    light_damage: false,
    light_color: LIGHT_COLOR,
    title_color: [0xff, 0xff, 0xff],
    bonus_loyalty: 0,
    bonus_opinion: 0,
    bonus_biomes_growth: 0,
    grow_vegetation: true,
    fire_spread_rate_bonus: 1.0,
    special_effect_interval: 10.0,
    default_slots: &[],
    link_default_slots: false,
};

pub static HOPE: AgeAsset = AgeAsset {
    age: Age::Hope,
    rate: 10,
    light_age: true,
    unfreeze_world: true,
    cloud_interval: 25.0,
    temperature_damage_bonus: 7,
    bonus_loyalty: 15,
    bonus_opinion: 15,
    title_color: [0x86, 0xbc, 0x4e],
    light_damage: true,
    biomes: &["biome_grass"],
    clouds: &[("cloud_normal", 1)],
    default_slots: &[1, 3, 5, 6],
    ..BASE
};

pub static SUN: AgeAsset = AgeAsset {
    age: Age::Sun,
    rate: 5,
    overlays: &[Overlay::Sun],
    fire_elemental_spawn_on_fire: true,
    fire_elemental_spawn_chance: 0.01,
    overlay_alpha: 0.1,
    temperature_damage_bonus: 15,
    unfreeze_world: true,
    unfreeze_mountains: true,
    particles: &[Particles::Sun],
    title_color: [0xf7, 0xa4, 0x2a],
    bonus_loyalty: 5,
    bonus_opinion: 5,
    bonus_biomes_growth: 3,
    grow_vegetation: false,
    fire_spread_rate_bonus: 2.0,
    light_damage: true,
    special_effect_interval: 7.0,
    biomes: &["biome_desert", "biome_savanna"],
    clouds: &[("cloud_normal", 1)],
    ..BASE
};

pub static DARK: AgeAsset = AgeAsset {
    age: Age::Dark,
    darkness: true,
    overlay_alpha: 0.3,
    overlays: &[Overlay::Night],
    conditions: &[Age::Hope, Age::Sun, Age::Wonders, Age::Tears],
    rate: 3,
    unfreeze_world: true,
    temperature_damage_bonus: -4,
    crops_grow: false,
    night: true,
    title_color: [0xa4, 0xa9, 0xb2],
    range_weapons_multiplier: 0.5,
    bonus_loyalty: -5,
    bonus_opinion: -5,
    bonus_biomes_growth: -2,
    biomes: &["biome_corrupted"],
    clouds: &[("cloud_normal", 1)],
    default_slots: &[2, 7],
    link_default_slots: true,
    ..BASE
};

pub static TEARS: AgeAsset = AgeAsset {
    age: Age::Tears,
    particles: &[Particles::Rain],
    conditions: &[Age::Hope, Age::Sun, Age::Wonders],
    rate: 3,
    unfreeze_world: true,
    overlays: &[Overlay::Rain],
    overlay_alpha: 0.3,
    cloud_interval: 20.0,
    special_effect_interval: 5.0,
    title_color: [0x6c, 0x97, 0xd8],
    bonus_biomes_growth: 5,
    biomes: &["biome_swamp", "biome_mushroom", "biome_jungle"],
    clouds: &[
        ("cloud_rain", 5),
        ("cloud_lightning", 1),
        ("cloud_normal", 2),
    ],
    default_slots: &[2, 7],
    link_default_slots: true,
    ..BASE
};

pub static MOON: AgeAsset = AgeAsset {
    age: Age::Moon,
    darkness: true,
    moon: true,
    overlays: &[Overlay::Moon],
    unfreeze_world: true,
    temperature_damage_bonus: -3,
    conditions: &[Age::Hope, Age::Sun],
    rate: 3,
    light_color: [0x8d, 0xff, 0xf3],
    title_color: [0xb5, 0xfa, 0xff],
    default_slots: &[2, 7],
    biomes: &["biome_crystal"],
    clouds: &[("cloud_rain", 5), ("cloud_normal", 5)],
    ..BASE
};

pub static CHAOS: AgeAsset = AgeAsset {
    age: Age::Chaos,
    overlays: &[Overlay::Chaos],
    chaos: true,
    rage_brings_demons: true,
    conditions: &[Age::Hope, Age::Sun, Age::Wonders],
    overlay_alpha: 0.35,
    rate: 2,
    unfreeze_world: true,
    unfreeze_mountains: true,
    title_color: [0xe6, 0x50, 0x3a],
    bonus_loyalty: -55,
    bonus_opinion: -35,
    light_damage: true,
    biomes: &["biome_infernal"],
    clouds: &[("cloud_rage", 1)],
    ..BASE
};

pub static WONDERS: AgeAsset = AgeAsset {
    age: Age::Wonders,
    rate: 2,
    light_age: true,
    particles: &[Particles::Magic],
    overlays: &[Overlay::Magic],
    overlay_alpha: 0.15,
    unfreeze_world: true,
    unfreeze_mountains: true,
    title_color: [0xd6, 0x55, 0x9c],
    biomes: &["biome_enchanted", "biome_candy"],
    clouds: &[("cloud_magic", 5), ("cloud_normal", 1)],
    default_slots: &[2, 7],
    link_default_slots: true,
    ..BASE
};

pub static ICE: AgeAsset = AgeAsset {
    age: Age::Ice,
    particles: &[Particles::Snow],
    conditions: &[Age::Hope, Age::Sun, Age::Wonders],
    rate: 2,
    freeze_world: true,
    overlays: &[Overlay::Winter],
    winter: true,
    overlay_alpha: 0.2,
    temperature_damage_bonus: 5,
    crops_grow: false,
    cloud_interval: 20.0,
    title_color: [0xc1, 0xfa, 0xff],
    bonus_biomes_growth: -20,
    years_min: 30,
    years_max: 40,
    biomes: &["biome_permafrost"],
    clouds: &[("cloud_snow", 1)],
    ..BASE
};

pub static ASH: AgeAsset = AgeAsset {
    age: Age::Ash,
    conditions: &[Age::Hope, Age::Sun, Age::Wonders],
    rate: 2,
    overlays: &[Overlay::Ash],
    particles: &[Particles::Ash],
    overlay_alpha: 0.4,
    unfreeze_world: true,
    cloud_interval: 15.0,
    title_color: [0xdd, 0xc4, 0x9d],
    bonus_loyalty: -25,
    bonus_opinion: -10,
    bonus_biomes_growth: -4,
    clouds: &[("cloud_ash", 1)],
    ..BASE
};

pub static DESPAIR: AgeAsset = AgeAsset {
    age: Age::Despair,
    darkness: true,
    particles: &[Particles::Snow],
    conditions: &[Age::Dark, Age::Ice],
    force_next: Some(Age::Hope),
    rate: 4,
    overlays: &[Overlay::Winter, Overlay::Night],
    winter: true,
    overlay_alpha: 0.25,
    freeze_world: true,
    temperature_damage_bonus: 3,
    crops_grow: false,
    night: true,
    cloud_interval: 16.0,
    title_color: [0x72, 0x85, 0x99],
    snow_turns_babies_into_ice_ones: true,
    range_weapons_multiplier: 0.5,
    bonus_loyalty: -10,
    bonus_opinion: -10,
    bonus_biomes_growth: -2,
    years_min: 30,
    years_max: 40,
    biomes: &["biome_permafrost", "biome_corrupted"],
    clouds: &[("cloud_snow", 1)],
    ..BASE
};

pub static UNKNOWN: AgeAsset = AgeAsset {
    age: Age::Unknown,
    title_color: [0xaa, 0xaa, 0xaa],
    default_slots: &[4, 8],
    ..BASE
};
