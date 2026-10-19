use crate::kind::CloudKind;

pub const CLOUD_INTERVAL: f32 = 15.0;

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

    pub fn clouds(self) -> &'static [(CloudKind, u32)] {
        match self {
            Self::Hope | Self::Sun | Self::Dark => &[(CloudKind::Normal, 1)],
            Self::Tears => &[
                (CloudKind::Rain, 5),
                (CloudKind::Lightning, 1),
                (CloudKind::Normal, 2),
            ],
            Self::Moon => &[(CloudKind::Rain, 5), (CloudKind::Normal, 5)],
            Self::Chaos => &[(CloudKind::Rage, 1)],
            Self::Wonders => &[(CloudKind::Magic, 5), (CloudKind::Normal, 1)],
            Self::Ice | Self::Despair => &[(CloudKind::Snow, 1)],
            Self::Ash => &[(CloudKind::Ash, 1)],
            Self::Unknown => &[],
        }
    }

    pub fn cloud_interval(self) -> f32 {
        match self {
            Self::Hope => 25.0,
            Self::Tears | Self::Ice => 20.0,
            Self::Despair => 16.0,
            _ => CLOUD_INTERVAL,
        }
    }

    pub fn pick(self, roll: u32) -> Option<CloudKind> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_world_starts_in_the_age_of_hope_with_plain_clouds_every_25_seconds() {
        let age = Age::default();

        assert_eq!(age.id(), "age_hope");
        assert_eq!(age.cloud_interval(), 25.0);
        assert_eq!(age.clouds(), &[(CloudKind::Normal, 1)]);
    }

    #[test]
    fn the_age_of_tears_rains_five_to_one_thunder_to_two_plain() {
        let mut counts = [0; 3];

        for roll in 0..8000 {
            match Age::Tears.pick(roll) {
                Some(CloudKind::Rain) => counts[0] += 1,
                Some(CloudKind::Lightning) => counts[1] += 1,
                Some(CloudKind::Normal) => counts[2] += 1,
                other => panic!("{other:?}"),
            }
        }

        assert_eq!(counts, [5000, 1000, 2000]);
    }

    #[test]
    fn the_unknown_age_brings_no_clouds() {
        assert_eq!(Age::Unknown.pick(7), None);
    }

    #[test]
    fn every_age_has_an_id_and_a_positive_interval() {
        for age in AGES {
            assert!(age.id().starts_with("age_"));
            assert!(age.cloud_interval() > 0.0);
        }
    }
}
