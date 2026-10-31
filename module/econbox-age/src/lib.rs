//! WorldBox's world ages: which one rules, how long it lasts, and what it brings to the map.

pub mod library;

use bevy::prelude::*;
use econbox_clock::{Frames, STEP, WorldClock};

pub use library::{AgeAsset, CLOUD_INTERVAL, LIGHT_COLOR, OVERLAYS, Overlay, Particles};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Age {
    #[default]
    Hope,
    Sun,
    Dark,
    Tears,
    Moon,
    Chaos,
    Wonders,
    Ice,
    Ash,
    Despair,
    Unknown,
}

pub const AGES: [Age; 11] = [
    Age::Hope,
    Age::Sun,
    Age::Dark,
    Age::Tears,
    Age::Moon,
    Age::Chaos,
    Age::Wonders,
    Age::Ice,
    Age::Ash,
    Age::Despair,
    Age::Unknown,
];

impl Age {
    pub fn asset(self) -> &'static AgeAsset {
        match self {
            Self::Hope => &library::HOPE,
            Self::Sun => &library::SUN,
            Self::Dark => &library::DARK,
            Self::Tears => &library::TEARS,
            Self::Moon => &library::MOON,
            Self::Chaos => &library::CHAOS,
            Self::Wonders => &library::WONDERS,
            Self::Ice => &library::ICE,
            Self::Ash => &library::ASH,
            Self::Despair => &library::DESPAIR,
            Self::Unknown => &library::UNKNOWN,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::Hope => "age_hope",
            Self::Sun => "age_sun",
            Self::Dark => "age_dark",
            Self::Tears => "age_tears",
            Self::Moon => "age_moon",
            Self::Chaos => "age_chaos",
            Self::Wonders => "age_wonders",
            Self::Ice => "age_ice",
            Self::Ash => "age_ash",
            Self::Despair => "age_despair",
            Self::Unknown => "age_unknown",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        AGES.into_iter().find(|age| age.id() == id)
    }

    pub fn clouds(self) -> &'static [(&'static str, u32)] {
        self.asset().clouds
    }

    pub fn cloud_interval(self) -> f32 {
        self.asset().cloud_interval
    }

    pub fn pick_cloud(self, roll: u32) -> Option<&'static str> {
        let pool = self.clouds();
        let total: u32 = pool.iter().map(|(_, weight)| weight).sum();

        if total == 0 {
            return None;
        }

        let mut left = roll % total;
        pool.iter().find_map(|&(kind, weight)| {
            if left < weight {
                Some(kind)
            } else {
                left -= weight;
                None
            }
        })
    }

    pub fn normal() -> impl Iterator<Item = Age> {
        AGES.into_iter().filter(|age| *age != Age::Unknown)
    }

    pub fn for_slot(slot: usize) -> impl Iterator<Item = Age> {
        AGES.into_iter()
            .filter(move |age| age.asset().default_slots.contains(&slot))
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WorldAge(pub Age);

pub const SLOTS: usize = 8;
pub const NIGHT_MAX: f32 = 0.6;
pub const NIGHT_SPEED: f32 = 0.1;
pub const OVERLAY_RISE: f32 = 0.2;
pub const OVERLAY_FALL: f32 = 0.7;
pub const OVERLAY_FADE: f32 = 0.2;
pub const YEAR: f32 = 12.0 * 5.0;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgeSettings {
    pub overlay_effect: i32,
    pub night_effect: i32,
    pub night_lights: bool,
}

impl Default for AgeSettings {
    fn default() -> Self {
        Self {
            overlay_effect: 100,
            night_effect: 100,
            night_lights: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layer {
    pub alpha: f32,
    pub enabled: bool,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            alpha: 0.0,
            enabled: true,
        }
    }
}

impl Layer {
    pub fn shown(&self) -> f32 {
        if self.enabled {
            self.alpha.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dice(u64);

impl Dice {
    pub fn new(seed: u32) -> Self {
        let state = u64::from(seed) ^ 0x9e37_79b9_7f4a_7c15;
        Self(if state == 0 { 1 } else { state })
    }

    pub fn roll(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    pub fn below(&mut self, count: usize) -> usize {
        (self.roll() % count.max(1) as u64) as usize
    }

    pub fn range(&mut self, min: u32, max_exclusive: u32) -> u32 {
        if max_exclusive <= min {
            return min;
        }
        min + (self.roll() % u64::from(max_exclusive - min)) as u32
    }

    pub fn pick<T: Copy>(&mut self, pool: &[T]) -> Option<T> {
        (!pool.is_empty()).then(|| pool[self.below(pool.len())])
    }
}

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct Ages {
    pub slots: [Age; SLOTS],
    pub slot: usize,
    pub current: Age,
    pub progress: f32,
    pub duration: f32,
    pub paused: bool,
    pub speed: f32,
    pub night: f32,
    pub layers: [Layer; 9],
    pub dice: Dice,
}

impl Default for Ages {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Ages {
    pub fn new(seed: u32) -> Self {
        let mut ages = Self {
            slots: [Age::Unknown; SLOTS],
            slot: 0,
            current: Age::Hope,
            progress: 0.0,
            duration: 0.0,
            paused: false,
            speed: 1.0,
            night: 0.0,
            layers: [Layer::default(); 9],
            dice: Dice::new(seed),
        };
        ages.set_current(Age::Hope, true);
        ages.set_default_ages();
        ages
    }

    pub fn restart(&mut self, seed: u32) {
        self.dice = Dice::new(seed);
        self.set_default_ages();
    }

    pub fn asset(&self) -> &'static AgeAsset {
        self.current.asset()
    }

    pub fn next_slot(&self) -> usize {
        (self.slot + 1) % SLOTS
    }

    pub fn next_age(&self) -> Age {
        self.slots[self.next_slot()]
    }

    pub fn night_mod(&self) -> f32 {
        self.night
    }

    pub fn show_lights(&self) -> bool {
        self.night_mod() != 0.0
    }

    pub fn time_left(&self) -> f32 {
        self.duration * (1.0 - self.progress)
    }

    pub fn moons_left(&self) -> i32 {
        ((self.duration * (1.0 - self.progress) / 5.0 + 1.0) / self.speed) as i32
    }

    pub fn set_default_ages(&mut self) {
        for slot in 1..=SLOTS {
            let mut pool: Vec<Age> = Age::for_slot(slot).collect();
            let Some(mut chosen) = self.dice.pick(&pool) else {
                continue;
            };
            if chosen.asset().link_default_slots {
                let taken = chosen
                    .asset()
                    .default_slots
                    .iter()
                    .any(|&other| other != slot && self.slots[other - 1] == chosen);
                if taken {
                    pool.retain(|age| *age != chosen);
                    if let Some(other) = self.dice.pick(&pool) {
                        chosen = other;
                    }
                }
            }
            self.set_age_to_slot(chosen, slot - 1);
        }
        self.set_current_slot(0, 0.0);
    }

    pub fn set_age_to_slot(&mut self, age: Age, slot: usize) {
        if self.slots[slot] == age {
            return;
        }
        self.slots[slot] = age;
        if slot == self.slot {
            self.set_current(age, false);
        }
    }

    pub fn start_next(&mut self, progress: f32) {
        self.set_current_slot(self.next_slot(), progress);
    }

    pub fn set_current_slot(&mut self, slot: usize, progress: f32) {
        self.slot = slot;
        self.progress = progress;
        let age = self.check(self.slots[slot]);
        self.set_current(age, true);
    }

    pub fn set_current(&mut self, age: Age, override_time: bool) {
        let age = self.check(age);
        if override_time {
            let asset = age.asset();
            self.duration =
                (self.dice.range(asset.years_min, asset.years_max) as f32 * YEAR) as i32 as f32;
        }
        self.current = age;
    }

    fn check(&mut self, age: Age) -> Age {
        if age == Age::Unknown {
            let normal: Vec<Age> = Age::normal().collect();
            return self.dice.pick(&normal).unwrap_or(Age::Hope);
        }
        age
    }

    pub fn update(&mut self, elapsed: f32, running: bool, settings: &AgeSettings) {
        if running && !self.paused && self.duration > 0.0 {
            self.progress += elapsed / (self.duration / self.speed);
            if self.progress >= 1.0 {
                self.start_next(0.0);
            }
        }

        self.update_layers(STEP, settings.overlay_effect);

        if !running {
            return;
        }

        if self.asset().darkness {
            let cap = NIGHT_MAX * (settings.night_effect as f32 / 100.0);
            self.night = (self.night + elapsed * NIGHT_SPEED).min(cap);
        } else {
            self.night = (self.night - elapsed * NIGHT_SPEED).max(0.0);
        }
    }

    pub fn update_layers(&mut self, elapsed: f32, overlay_effect: i32) {
        let asset = self.asset();
        let option = overlay_effect as f32;
        let target = asset.overlay_alpha * (option / 100.0);

        for (layer, overlay) in self.layers.iter_mut().zip(OVERLAYS) {
            let wanted = asset.overlays.contains(&overlay);
            if wanted {
                layer.enabled = option > 0.0;
                if layer.alpha < target {
                    layer.alpha = (layer.alpha + elapsed * OVERLAY_RISE).min(target);
                }
                if layer.alpha > target {
                    layer.alpha = (layer.alpha - elapsed * OVERLAY_FALL).max(target);
                }
            }
            if !wanted && layer.enabled && layer.alpha > 0.0 {
                layer.alpha -= elapsed * OVERLAY_FADE;
                if layer.alpha <= 0.0 {
                    layer.enabled = false;
                }
            }
        }
    }

    pub fn layer(&self, overlay: Overlay) -> Layer {
        self.layers[overlay.index()]
    }
}

pub struct AgePlugin;

impl Plugin for AgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldAge>()
            .init_resource::<Ages>()
            .init_resource::<AgeSettings>()
            .init_resource::<Frames>()
            .add_systems(Update, advance);
    }
}

fn advance(
    frames: Res<Frames>,
    clock: Option<Res<WorldClock>>,
    settings: Res<AgeSettings>,
    mut ages: ResMut<Ages>,
    mut world: ResMut<WorldAge>,
) {
    let clock = clock.map(|clock| *clock).unwrap_or_default();
    for _ in 0..frames.due {
        ages.update(clock.step(), clock.running(), &settings);
    }
    world.set_if_neq(WorldAge(ages.current));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_world_starts_in_the_age_of_hope_with_plain_clouds_every_25_seconds() {
        let age = Age::default();

        assert_eq!(age.id(), "age_hope");
        assert_eq!(age.cloud_interval(), 25.0);
        assert_eq!(age.clouds(), &[("cloud_normal", 1)]);
        assert_eq!(Ages::new(7).current, Age::Hope);
    }

    #[test]
    fn the_age_of_tears_rains_five_to_one_thunder_to_two_plain() {
        let mut counts = [0; 3];

        for roll in 0..8000 {
            match Age::Tears.pick_cloud(roll) {
                Some("cloud_rain") => counts[0] += 1,
                Some("cloud_lightning") => counts[1] += 1,
                Some("cloud_normal") => counts[2] += 1,
                other => panic!("{other:?}"),
            }
        }

        assert_eq!(counts, [5000, 1000, 2000]);
    }

    #[test]
    fn the_unknown_age_brings_no_clouds() {
        assert_eq!(Age::Unknown.pick_cloud(7), None);
    }

    #[test]
    fn every_age_has_an_id_and_a_positive_interval() {
        for age in AGES {
            assert!(age.id().starts_with("age_"));
            assert!(age.cloud_interval() > 0.0);
            assert_eq!(Age::from_id(age.id()), Some(age));
            assert_eq!(age.asset().age, age);
        }
    }

    #[test]
    fn the_default_wheel_alternates_hope_with_dark_ages_and_unknowns() {
        for seed in 0..200 {
            let ages = Ages::new(seed);
            let s = ages.slots;

            for slot in [0, 2, 4, 5] {
                assert_eq!(s[slot], Age::Hope, "{seed} {s:?}");
            }
            for slot in [3, 7] {
                assert_eq!(s[slot], Age::Unknown, "{seed} {s:?}");
            }
            for slot in [1, 6] {
                assert!(
                    [Age::Dark, Age::Tears, Age::Moon, Age::Wonders].contains(&s[slot]),
                    "{seed} {s:?}"
                );
            }
            assert!(s[1] != s[6] || s[1] == Age::Moon, "{seed} {s:?}");
        }
    }

    #[test]
    fn an_age_lasts_its_years_in_months_of_five_seconds() {
        for seed in 0..100 {
            let ages = Ages::new(seed);
            assert!(ages.duration >= 35.0 * YEAR && ages.duration < 55.0 * YEAR);
            assert_eq!(ages.duration % YEAR, 0.0);
        }
    }

    #[test]
    fn an_unknown_slot_becomes_some_real_age_when_it_comes() {
        let mut ages = Ages::new(3);
        ages.set_current_slot(3, 0.0);

        assert_ne!(ages.current, Age::Unknown);
        assert_eq!(ages.slots[3], Age::Unknown);
    }

    #[test]
    fn when_the_time_runs_out_the_next_slot_takes_over() {
        let mut ages = Ages::new(1);
        let settings = AgeSettings::default();
        let frames = (ages.duration / STEP).ceil() as usize;

        for _ in 0..frames * 99 / 100 {
            ages.update(STEP, true, &settings);
        }
        assert_eq!(ages.slot, 0);

        for _ in 0..frames / 20 {
            if ages.slot != 0 {
                break;
            }
            ages.update(STEP, true, &settings);
        }

        assert_eq!(ages.slot, 1);
        assert_eq!(ages.current, ages.slots[1]);
        assert!(ages.progress < 0.01);
    }

    #[test]
    fn a_paused_world_or_wheel_keeps_the_age() {
        let mut ages = Ages::new(1);
        let settings = AgeSettings::default();
        ages.update(1.0e6, false, &settings);
        assert_eq!(ages.slot, 0);

        ages.paused = true;
        ages.update(1.0e6, true, &settings);
        assert_eq!(ages.slot, 0);
    }

    #[test]
    fn dark_ages_bring_the_night_at_a_tenth_per_second_up_to_its_cap() {
        let mut ages = Ages::new(1);
        let settings = AgeSettings {
            night_effect: 50,
            ..AgeSettings::default()
        };
        ages.set_current(Age::Dark, false);

        ages.update(1.0, true, &settings);
        assert!((ages.night - 0.1).abs() < 1e-6);
        assert!(ages.show_lights());

        for _ in 0..20 {
            ages.update(1.0, true, &settings);
        }
        assert!((ages.night - 0.3).abs() < 1e-6);

        ages.set_current(Age::Hope, false);
        ages.update(1.0, false, &settings);
        assert!((ages.night - 0.3).abs() < 1e-6);
        for _ in 0..10 {
            ages.update(1.0, true, &settings);
        }
        assert_eq!(ages.night, 0.0);
        assert!(!ages.show_lights());
    }

    #[test]
    fn an_overlay_rises_slowly_falls_fast_and_fades_out_when_the_age_ends() {
        let mut ages = Ages::new(1);
        ages.set_current(Age::Ash, false);
        let ash = Overlay::Ash;

        ages.update_layers(1.0, 100);
        assert!((ages.layer(ash).alpha - 0.2).abs() < 1e-6);
        ages.update_layers(5.0, 100);
        assert!((ages.layer(ash).alpha - 0.4).abs() < 1e-6);

        ages.update_layers(0.1, 50);
        assert!((ages.layer(ash).alpha - 0.33).abs() < 1e-5);
        ages.update_layers(1.0, 50);
        assert!((ages.layer(ash).alpha - 0.2).abs() < 1e-6);

        ages.set_current(Age::Hope, false);
        ages.update_layers(0.5, 100);
        assert!((ages.layer(ash).alpha - 0.1).abs() < 1e-6);
        ages.update_layers(0.6, 100);
        assert!(!ages.layer(ash).enabled);
        assert_eq!(ages.layer(ash).shown(), 0.0);
    }

    #[test]
    fn a_zero_overlay_option_switches_the_layer_off() {
        let mut ages = Ages::new(1);
        ages.set_current(Age::Chaos, false);
        ages.update_layers(1.0, 0);

        assert!(!ages.layer(Overlay::Chaos).enabled);
    }

    #[test]
    fn the_despair_age_shows_both_winter_and_night() {
        let mut ages = Ages::new(1);
        ages.set_current(Age::Despair, false);
        ages.update_layers(10.0, 100);

        assert!((ages.layer(Overlay::Winter).shown() - 0.25).abs() < 1e-6);
        assert!((ages.layer(Overlay::Night).shown() - 0.25).abs() < 1e-6);
        assert_eq!(ages.layer(Overlay::Sun).shown(), 0.0);
    }
}
