use crate::zoom::{FOCUS_ZOOM, Zoom};
use bevy::prelude::*;

pub const FOLLOW_EASE_TIME: f32 = 1.0;

#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub enum FocusOn {
    Spot(Vec2),
    Follow(Entity),
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusEnded {
    Reached,
    Cancelled,
}

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq)]
pub struct Focus {
    pub zoom: Option<f32>,
    pub unit: Option<Entity>,
    pub timer: f32,
}

pub fn ease_out_cubic(start: f32, end: f32, t: f32) -> f32 {
    let t = t - 1.0;
    (end - start) * (t * t * t + 1.0) + start
}

impl Focus {
    pub fn following(&self) -> bool {
        self.unit.is_some()
    }

    pub fn stop_following(&mut self) {
        self.unit = None;
        self.timer = 0.0;
    }

    pub fn spot(&mut self, zoom: &mut Zoom, position: &mut Vec2, at: Vec2) {
        self.stop_following();
        zoom.target = FOCUS_ZOOM;
        self.zoom = Some(FOCUS_ZOOM);
        *position = at;
    }

    pub fn follow(&mut self, zoom: &mut Zoom, unit: Entity) {
        self.stop_following();
        zoom.target = FOCUS_ZOOM;
        self.zoom = Some(FOCUS_ZOOM);
        self.unit = Some(unit);
        self.timer = 0.0;
    }

    pub fn check(&mut self, zoom: &Zoom) -> Option<FocusEnded> {
        let wanted = self.zoom?;

        if zoom.size == wanted {
            self.zoom = None;
            return Some(FocusEnded::Reached);
        }

        if zoom.target != wanted {
            self.zoom = None;
            return Some(FocusEnded::Cancelled);
        }

        None
    }

    pub fn track(&mut self, position: &mut Vec2, unit_at: Option<Vec2>, dragging: bool, dt: f32) {
        if self.unit.is_none() {
            return;
        }

        let Some(mut next) = unit_at else {
            self.stop_following();
            return;
        };

        if dragging {
            self.timer = 0.0;
            return;
        }

        if self.timer <= FOLLOW_EASE_TIME {
            self.timer = (self.timer + dt).clamp(0.0, FOLLOW_EASE_TIME);
            next.x = ease_out_cubic(position.x, next.x, self.timer);
            next.y = ease_out_cubic(position.y, next.y, self.timer);
        }

        *position = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zoom() -> Zoom {
        Zoom {
            size: 200.0,
            target: 200.0,
            max: 281.0,
        }
    }

    #[test]
    fn focusing_on_a_spot_jumps_there_and_aims_for_fifteen_tiles() {
        let mut focus = Focus::default();
        let mut zoom = zoom();
        let mut position = Vec2::ZERO;

        focus.spot(&mut zoom, &mut position, Vec2::new(30.0, -12.0));

        assert_eq!(position, Vec2::new(30.0, -12.0));
        assert_eq!(zoom.target, FOCUS_ZOOM);
        assert_eq!(focus.check(&zoom), None);

        zoom.size = FOCUS_ZOOM;
        assert_eq!(focus.check(&zoom), Some(FocusEnded::Reached));
        assert_eq!(focus.check(&zoom), None);
    }

    #[test]
    fn touching_the_zoom_on_the_way_cancels_the_focus() {
        let mut focus = Focus::default();
        let mut zoom = zoom();
        let mut position = Vec2::ZERO;

        focus.spot(&mut zoom, &mut position, Vec2::ZERO);
        zoom.wheel_out();

        assert_eq!(focus.check(&zoom), Some(FocusEnded::Cancelled));
        assert_eq!(focus.zoom, None);
    }

    #[test]
    fn following_eases_in_over_a_second_then_sticks_to_the_unit() {
        let mut focus = Focus::default();
        let mut zoom = zoom();
        let unit = Entity::from_raw_u32(7).expect("entity");
        let mut position = Vec2::ZERO;

        focus.follow(&mut zoom, unit);
        assert!(focus.following());

        focus.track(&mut position, Some(Vec2::new(100.0, 0.0)), false, 0.5);
        let expected = ease_out_cubic(0.0, 100.0, 0.5);
        assert!((position.x - expected).abs() < 1e-4, "{position}");

        focus.track(&mut position, Some(Vec2::new(100.0, 0.0)), false, 0.6);
        assert_eq!(position, Vec2::new(100.0, 0.0));

        focus.track(&mut position, Some(Vec2::new(104.0, 3.0)), false, 0.016);
        assert_eq!(position, Vec2::new(104.0, 3.0));
    }

    #[test]
    fn dragging_holds_the_follow_and_restarts_its_ease() {
        let mut focus = Focus {
            unit: Some(Entity::from_raw_u32(3).expect("entity")),
            timer: 2.0,
            ..default()
        };
        let mut position = Vec2::new(5.0, 5.0);

        focus.track(&mut position, Some(Vec2::new(50.0, 50.0)), true, 0.016);

        assert_eq!(position, Vec2::new(5.0, 5.0));
        assert_eq!(focus.timer, 0.0);
    }

    #[test]
    fn a_vanished_unit_ends_the_follow() {
        let mut focus = Focus {
            unit: Some(Entity::from_raw_u32(3).expect("entity")),
            ..default()
        };
        let mut position = Vec2::ZERO;

        focus.track(&mut position, None, false, 0.016);

        assert!(!focus.following());
        assert_eq!(position, Vec2::ZERO);
    }

    #[test]
    fn the_cubic_ease_starts_fast_and_lands_on_the_end() {
        assert_eq!(ease_out_cubic(0.0, 10.0, 0.0), 0.0);
        assert_eq!(ease_out_cubic(0.0, 10.0, 1.0), 10.0);
        assert!((ease_out_cubic(0.0, 10.0, 0.5) - 8.75).abs() < 1e-5);
    }
}
