use crate::art::Puff;
use econbox_drops::DropKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CloudKind {
    Rain,
    Rage,
    Lightning,
    Acid,
    Lava,
    Fire,
    Snow,
    Ash,
    Magic,
    Normal,
}

pub const KINDS: [CloudKind; 10] = [
    CloudKind::Rain,
    CloudKind::Rage,
    CloudKind::Lightning,
    CloudKind::Acid,
    CloudKind::Lava,
    CloudKind::Fire,
    CloudKind::Snow,
    CloudKind::Ash,
    CloudKind::Magic,
    CloudKind::Normal,
];

const SMALL: [Puff; 2] = [Puff::Small, Puff::Wisp];
const BIG: [Puff; 3] = [Puff::Broad, Puff::Tall, Puff::Grand];
const ALL: [Puff; 5] = [
    Puff::Small,
    Puff::Wisp,
    Puff::Broad,
    Puff::Tall,
    Puff::Grand,
];

impl CloudKind {
    pub fn id(self) -> &'static str {
        match self {
            Self::Rain => "cloud_rain",
            Self::Rage => "cloud_rage",
            Self::Lightning => "cloud_lightning",
            Self::Acid => "cloud_acid",
            Self::Lava => "cloud_lava",
            Self::Fire => "cloud_fire",
            Self::Snow => "cloud_snow",
            Self::Ash => "cloud_ash",
            Self::Magic => "cloud_magic",
            Self::Normal => "cloud_normal",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        KINDS.into_iter().find(|kind| kind.id() == id)
    }

    pub fn color(self) -> [u8; 3] {
        match self {
            Self::Rain => [0x5d, 0x72, 0x8c],
            Self::Rage => [0xff, 0x30, 0x30],
            Self::Lightning => [0x44, 0x53, 0x66],
            Self::Acid => [0x17, 0xd4, 0x1c],
            Self::Lava => [0xd1, 0x71, 0x19],
            Self::Fire => [0xd1, 0x42, 0x19],
            Self::Snow => [0xb8, 0xff, 0xfa],
            Self::Ash => [0xc6, 0xb0, 0x77],
            Self::Magic => [0xc9, 0x76, 0xcc],
            Self::Normal => [0xff, 0xff, 0xff],
        }
    }

    pub fn max_alpha(self) -> f32 {
        match self {
            Self::Normal => 0.5,
            _ => 0.8,
        }
    }

    pub fn drop_every(self) -> f32 {
        match self {
            Self::Normal => 2.0,
            _ => 0.05,
        }
    }

    pub fn bolt_every(self) -> Option<f32> {
        match self {
            Self::Lightning => Some(0.05),
            _ => None,
        }
    }

    pub fn speed(self) -> (f32, f32) {
        match self {
            Self::Lava | Self::Fire => (1.0, 3.0),
            Self::Magic => (1.0, 7.0),
            Self::Normal => (2.0, 6.0),
            _ => (1.0, 4.0),
        }
    }

    pub fn drop(self) -> DropKind {
        match self {
            Self::Rain | Self::Lightning => DropKind::Rain,
            Self::Rage => DropKind::Rage,
            Self::Acid => DropKind::Acid,
            Self::Lava => DropKind::Lava,
            Self::Fire => DropKind::Fire,
            Self::Snow => DropKind::Snow,
            Self::Ash => DropKind::Ash,
            Self::Magic => DropKind::MagicRain,
            Self::Normal => DropKind::LifeSeed,
        }
    }

    pub fn puffs(self) -> &'static [Puff] {
        match self {
            Self::Magic => &SMALL,
            Self::Normal => &ALL,
            _ => &BIG,
        }
    }

    pub fn glows(self) -> bool {
        matches!(self, Self::Rage | Self::Lava | Self::Fire | Self::Magic)
    }

    pub fn disaster(self) -> bool {
        !matches!(self, Self::Rain | Self::Normal)
    }

    pub fn normal(self) -> bool {
        self == Self::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_is_found_by_the_id_worldbox_gives_it() {
        for kind in KINDS {
            assert_eq!(CloudKind::from_id(kind.id()), Some(kind));
        }

        assert_eq!(CloudKind::from_id("cloud_blood"), None);
    }

    #[test]
    fn the_plain_cloud_is_fainter_slower_to_drop_and_never_a_disaster() {
        let plain = CloudKind::Normal;

        assert_eq!(plain.max_alpha(), 0.5);
        assert_eq!(plain.drop_every(), 2.0);
        assert_eq!(plain.speed(), (2.0, 6.0));
        assert_eq!(plain.puffs().len(), 5);
        assert!(plain.normal() && !plain.disaster());
        assert_eq!(plain.drop(), DropKind::LifeSeed);
    }

    #[test]
    fn weather_clouds_are_big_and_magic_ones_small() {
        for kind in KINDS {
            let big = kind.puffs().iter().all(|puff| puff.big());

            match kind {
                CloudKind::Normal => assert!(!big),
                CloudKind::Magic => assert!(kind.puffs().iter().all(|puff| !puff.big())),
                _ => assert!(big, "{kind:?}"),
            }
        }
    }

    #[test]
    fn only_the_rain_cloud_is_weather_that_is_not_a_disaster() {
        let calm: Vec<CloudKind> = KINDS.into_iter().filter(|kind| !kind.disaster()).collect();

        assert_eq!(calm, [CloudKind::Rain, CloudKind::Normal]);
    }

    #[test]
    fn only_the_thunder_cloud_throws_lightning() {
        for kind in KINDS {
            assert_eq!(kind.bolt_every().is_some(), kind == CloudKind::Lightning);
        }
    }
}
