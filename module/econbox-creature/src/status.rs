use bevy::prelude::*;
use econbox_tilemap::TilePos;

pub const BURN_SHARE: f32 = 0.1;
pub const FEVER_SHARE: f32 = 0.01;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    Burning,
    Frozen,
    Rage,
    Angry,
    Powerup,
    SpellBoost,
    Shield,
    Caffeinated,
    Cough,
    AshFever,
    Slowness,
}

pub const STATUSES: [Status; 11] = [
    Status::Burning,
    Status::Frozen,
    Status::Rage,
    Status::Angry,
    Status::Powerup,
    Status::SpellBoost,
    Status::Shield,
    Status::Caffeinated,
    Status::Cough,
    Status::AshFever,
    Status::Slowness,
];

impl Status {
    pub fn id(self) -> &'static str {
        match self {
            Self::Burning => "burning",
            Self::Frozen => "frozen",
            Self::Rage => "rage",
            Self::Angry => "angry",
            Self::Powerup => "powerup",
            Self::SpellBoost => "spell_boost",
            Self::Shield => "shield",
            Self::Caffeinated => "caffeinated",
            Self::Cough => "cough",
            Self::AshFever => "ash_fever",
            Self::Slowness => "slowness",
        }
    }

    pub fn duration(self) -> f32 {
        match self {
            Self::Burning => 30.0,
            Self::Frozen => 15.0,
            Self::Rage | Self::Powerup => 300.0,
            Self::Angry | Self::Shield | Self::Caffeinated => 60.0,
            Self::SpellBoost => 180.0,
            Self::Cough => 200.0,
            Self::AshFever => 500.0,
            Self::Slowness => 30.0,
        }
    }

    pub fn resets_timer(self) -> bool {
        !matches!(self, Self::Burning | Self::Frozen)
    }

    pub fn interval(self) -> Option<f32> {
        match self {
            Self::Burning => Some(2.0),
            Self::AshFever => Some(10.0),
            _ => None,
        }
    }

    pub fn removes(self) -> &'static [Status] {
        match self {
            Self::Burning => &[Self::Frozen],
            Self::Frozen => &[Self::Burning],
            Self::Shield => &[Self::Burning],
            Self::Caffeinated => &[Self::Frozen],
            Self::Slowness => &[Self::Caffeinated],
            _ => &[],
        }
    }

    pub fn opposites(self) -> &'static [Status] {
        match self {
            Self::Burning | Self::Frozen => &[Self::Shield],
            _ => &[],
        }
    }

    pub fn speed_multiplier(self) -> f32 {
        match self {
            Self::Cough | Self::AshFever => -0.1,
            _ => 0.0,
        }
    }

    pub fn health_multiplier(self) -> f32 {
        match self {
            Self::Cough => -0.1,
            Self::AshFever => -0.6,
            _ => 0.0,
        }
    }

    pub fn immovable(self) -> bool {
        self == Self::Frozen
    }

    pub fn share(self) -> Option<f32> {
        match self {
            Self::Burning => Some(BURN_SHARE),
            Self::AshFever => Some(FEVER_SHARE),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Active {
    pub status: Status,
    pub left: f32,
    pub action_in: f32,
}

#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct Statuses(pub Vec<Active>);

impl Statuses {
    pub fn has(&self, status: Status) -> bool {
        self.0.iter().any(|active| active.status == status)
    }

    pub fn add(&mut self, status: Status) -> bool {
        if let Some(active) = self.0.iter_mut().find(|active| active.status == status) {
            if !status.resets_timer() {
                return false;
            }
            active.left = active.left.max(status.duration());
            return true;
        }

        if status
            .opposites()
            .iter()
            .any(|opposite| self.has(*opposite))
        {
            return false;
        }

        self.0.push(Active {
            status,
            left: status.duration(),
            action_in: status.interval().unwrap_or(0.0),
        });

        for removed in status.removes() {
            self.finish(*removed);
        }

        true
    }

    pub fn finish(&mut self, status: Status) {
        self.0.retain(|active| active.status != status);
    }

    pub fn speed(&self) -> f32 {
        if self.0.iter().any(|active| active.status.immovable()) {
            return 0.0;
        }

        (1.0 + self
            .0
            .iter()
            .map(|active| active.status.speed_multiplier())
            .sum::<f32>())
        .max(0.0)
    }

    pub fn health(&self) -> f32 {
        (1.0 + self
            .0
            .iter()
            .map(|active| active.status.health_multiplier())
            .sum::<f32>())
        .max(0.0)
    }

    pub fn tick(&mut self, seconds: f32, max_health: f32) -> f32 {
        let mut damage = 0.0;

        for active in &mut self.0 {
            if let (Some(interval), Some(share)) = (active.status.interval(), active.status.share())
            {
                if active.action_in > 0.0 {
                    active.action_in -= seconds;
                } else {
                    active.action_in = interval;
                    damage += ((max_health * share) as i32).max(1) as f32;
                }
            }

            active.left -= seconds;
        }

        self.0.retain(|active| active.left > 0.0);
        damage
    }
}

#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct Afflict {
    pub centre: TilePos,
    pub reach: f32,
    pub status: Status,
    pub chance: f32,
}

#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct Harm {
    pub centre: TilePos,
    pub reach: f32,
    pub amount: f32,
    pub chance: f32,
}

#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct Relieve {
    pub centre: TilePos,
    pub reach: f32,
    pub status: Status,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_its_worldbox_id_and_a_positive_duration() {
        for status in STATUSES {
            assert!(!status.id().is_empty());
            assert!(status.duration() > 0.0);
        }
    }

    #[test]
    fn freezing_puts_out_fire_and_fire_thaws_ice() {
        let mut statuses = Statuses::default();
        statuses.add(Status::Burning);
        statuses.add(Status::Frozen);

        assert!(statuses.has(Status::Frozen) && !statuses.has(Status::Burning));

        statuses.add(Status::Burning);
        assert!(statuses.has(Status::Burning) && !statuses.has(Status::Frozen));
    }

    #[test]
    fn a_shield_keeps_fire_and_ice_off() {
        let mut statuses = Statuses::default();
        statuses.add(Status::Shield);

        assert!(!statuses.add(Status::Burning));
        assert!(!statuses.add(Status::Frozen));
    }

    #[test]
    fn burning_does_not_restart_but_rage_does() {
        let mut statuses = Statuses::default();
        statuses.add(Status::Burning);
        statuses.add(Status::Rage);
        statuses.tick(10.0, 100.0);

        assert!(!statuses.add(Status::Burning));
        assert!(statuses.add(Status::Rage));
        let left = |status| statuses.0.iter().find(|a| a.status == status).unwrap().left;
        assert_eq!(left(Status::Rage), 300.0);
        assert_eq!(left(Status::Burning), 20.0);
    }

    #[test]
    fn burning_takes_a_tenth_of_full_health_every_two_seconds() {
        let mut statuses = Statuses::default();
        statuses.add(Status::Burning);
        let mut taken = 0.0;

        for _ in 0..100 {
            taken += statuses.tick(0.1, 100.0);
        }

        assert!((30.0..=40.0).contains(&taken), "{taken}");
    }

    #[test]
    fn the_frozen_cannot_move_and_the_coughing_slow_down() {
        let mut statuses = Statuses::default();
        statuses.add(Status::Cough);
        assert!((statuses.speed() - 0.9).abs() < 1e-6);
        assert!((statuses.health() - 0.9).abs() < 1e-6);

        statuses.add(Status::Frozen);
        assert_eq!(statuses.speed(), 0.0);
    }

    #[test]
    fn statuses_run_out() {
        let mut statuses = Statuses::default();
        statuses.add(Status::Frozen);
        statuses.tick(14.0, 10.0);
        assert!(statuses.has(Status::Frozen));
        statuses.tick(1.5, 10.0);
        assert!(!statuses.has(Status::Frozen));
    }
}
