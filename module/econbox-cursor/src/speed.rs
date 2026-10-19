use bevy::prelude::*;

pub const SPEED_DECAY: f32 = 0.95;
pub const SPEED_DROP: f32 = 1.0;
pub const SPEED_CAP: f32 = 100.0;

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq)]
pub struct CursorSpeed {
    pub speed: f32,
    pub fmod_speed: f32,
    pub difference: f32,
    last: Vec2,
    current: Vec2,
}

impl CursorSpeed {
    pub fn step(&mut self, left_held: bool, pointer: Option<Vec2>) {
        if left_held {
            self.last = self.current;
            if let Some(pointer) = pointer {
                self.current = pointer;
            }
            self.difference = self.current.distance(self.last) / 2.0;
            if self.difference > self.speed {
                self.speed = self.difference;
            }
        }

        self.speed = self.speed * SPEED_DECAY - SPEED_DROP;
        if self.speed < 0.0 {
            self.speed = 0.0;
        }

        self.fmod_speed = self.speed.trunc().min(SPEED_CAP);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fast_stroke_spikes_the_speed_and_then_it_bleeds_away() {
        let mut speed = CursorSpeed::default();

        speed.step(true, Some(Vec2::ZERO));
        speed.step(true, Some(Vec2::new(80.0, 0.0)));
        assert_eq!(speed.difference, 40.0);
        assert!((speed.speed - (40.0 * 0.95 - 1.0)).abs() < 1e-4);
        assert_eq!(speed.fmod_speed, 37.0);

        for _ in 0..200 {
            speed.step(false, Some(Vec2::new(80.0, 0.0)));
        }
        assert_eq!(speed.speed, 0.0);
        assert_eq!(speed.fmod_speed, 0.0);
    }

    #[test]
    fn the_sound_parameter_is_capped_at_a_hundred() {
        let mut speed = CursorSpeed::default();

        speed.step(true, Some(Vec2::ZERO));
        speed.step(true, Some(Vec2::new(1000.0, 0.0)));

        assert_eq!(speed.fmod_speed, SPEED_CAP);
        assert!(speed.speed > SPEED_CAP);
    }

    #[test]
    fn moving_without_the_button_adds_nothing() {
        let mut speed = CursorSpeed::default();

        speed.step(false, Some(Vec2::new(500.0, 500.0)));

        assert_eq!(speed.speed, 0.0);
    }
}
