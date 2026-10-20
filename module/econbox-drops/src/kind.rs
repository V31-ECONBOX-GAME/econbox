pub const FALLING_SPEED: f32 = 3.2;
pub const FALLING_SPEED_RANDOM: f32 = 0.5;
pub const FALLING_HEIGHT: (f32, f32) = (30.0, 45.0);
pub const DROP_SCALE: f32 = 0.2;
pub const ANIMATION_SPEED: f32 = 0.1;
pub const ANIMATION_SPEED_RANDOM: f32 = 0.1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DropKind {
    Rain,
    Acid,
    Fire,
    Lava,
    Snow,
    Ash,
    MagicRain,
    Rage,
    LifeSeed,
}

pub const DROPS: [DropKind; 9] = [
    DropKind::Rain,
    DropKind::Acid,
    DropKind::Fire,
    DropKind::Lava,
    DropKind::Snow,
    DropKind::Ash,
    DropKind::MagicRain,
    DropKind::Rage,
    DropKind::LifeSeed,
];

impl DropKind {
    pub fn id(self) -> &'static str {
        match self {
            Self::Rain => "rain",
            Self::Acid => "acid",
            Self::Fire => "fire",
            Self::Lava => "lava",
            Self::Snow => "snow",
            Self::Ash => "ash",
            Self::MagicRain => "magic_rain",
            Self::Rage => "rage",
            Self::LifeSeed => "life_seed",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        DROPS.into_iter().find(|kind| kind.id() == id)
    }

    pub fn falling_speed(self) -> f32 {
        match self {
            Self::Snow | Self::Ash | Self::MagicRain | Self::Rage | Self::LifeSeed => 0.3,
            Self::Rain | Self::Acid | Self::Fire | Self::Lava => FALLING_SPEED,
        }
    }

    pub fn drifts(self) -> bool {
        !matches!(self, Self::Rain | Self::Acid | Self::Lava)
    }

    pub fn random_frame(self) -> bool {
        self != Self::Lava
    }

    pub fn random_flip(self) -> bool {
        self == Self::Fire
    }

    pub fn animation(self) -> Option<(f32, f32)> {
        match self {
            Self::Fire => Some((0.03, 0.08)),
            Self::Lava => Some((0.03, ANIMATION_SPEED_RANDOM)),
            _ => None,
        }
    }

    pub fn lit(self) -> bool {
        matches!(self, Self::Fire | Self::Acid | Self::Lava)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_drop_is_found_by_its_worldbox_id() {
        for kind in DROPS {
            assert_eq!(DropKind::from_id(kind.id()), Some(kind));
        }
    }

    #[test]
    fn rain_acid_fire_and_lava_fall_fast_and_the_rest_float_down() {
        for kind in DROPS {
            let fast = matches!(
                kind,
                DropKind::Rain | DropKind::Acid | DropKind::Fire | DropKind::Lava
            );
            assert_eq!(kind.falling_speed() > 1.0, fast, "{kind:?}");
        }
    }

    #[test]
    fn only_fire_and_lava_animate_and_only_fire_flips() {
        for kind in DROPS {
            assert_eq!(
                kind.animation().is_some(),
                matches!(kind, DropKind::Fire | DropKind::Lava)
            );
            assert_eq!(kind.random_flip(), kind == DropKind::Fire);
        }
    }

    #[test]
    fn the_light_ones_drift_sideways_as_they_fall() {
        assert!(DropKind::Snow.drifts());
        assert!(DropKind::Fire.drifts());
        assert!(!DropKind::Rain.drifts());
        assert!(!DropKind::Lava.drifts());
    }
}
