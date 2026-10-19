use bevy::prelude::*;

pub const DRAG_ACTIVATION: f32 = 0.1;
pub const DRAG_STILL: f32 = 0.01;
pub const DRAG_CARRY: f32 = 0.2;
pub const GLIDE_DECAY: f32 = 0.8;
pub const GLIDE_STOP: f32 = 0.01;
pub const KEY_RATE: f32 = 55.0;
pub const KEY_FAST: f32 = 2.5;
pub const REFERENCE_FRAME: f32 = 1.0 / 60.0;
pub const DRAG_HOLD_FRAMES: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Heading {
    Up,
    Down,
    Left,
    Right,
}

impl Heading {
    pub const ALL: [Heading; 4] = [Heading::Up, Heading::Down, Heading::Left, Heading::Right];

    pub fn keys(self) -> [KeyCode; 2] {
        match self {
            Heading::Up => [KeyCode::KeyW, KeyCode::ArrowUp],
            Heading::Down => [KeyCode::KeyS, KeyCode::ArrowDown],
            Heading::Left => [KeyCode::KeyA, KeyCode::ArrowLeft],
            Heading::Right => [KeyCode::KeyD, KeyCode::ArrowRight],
        }
    }

    pub fn axis(self) -> Vec2 {
        match self {
            Heading::Up => Vec2::Y,
            Heading::Down => Vec2::NEG_Y,
            Heading::Left => Vec2::NEG_X,
            Heading::Right => Vec2::X,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DragInput {
    pub pressed: bool,
    pub held: bool,
    pub over_ui: bool,
    pub pointer: Option<Vec2>,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Panning {
    pub can_drag_map: bool,
    pub running: bool,
    pub activated: bool,
    pub velocity: Vec2,
    pub mouse_last: bool,
    origin: Option<Vec2>,
    activated_frame: u32,
}

impl Default for Panning {
    fn default() -> Self {
        Self {
            can_drag_map: true,
            running: false,
            activated: false,
            velocity: Vec2::ZERO,
            mouse_last: false,
            origin: None,
            activated_frame: 0,
        }
    }
}

pub fn clamp_to_map(position: Vec2, map_tiles: Vec2) -> Vec2 {
    let half = map_tiles * 0.5;
    Vec2::new(
        position.x.clamp(-half.x, half.x),
        position.y.clamp(-half.y, half.y),
    )
}

pub fn opening_position(map_tiles: Vec2) -> Vec2 {
    (map_tiles / 2.0).floor() - map_tiles * 0.5
}

pub fn key_distance(dt: f32, fast: bool, target_zoom: f32, move_speed: f32) -> f32 {
    let mut distance = dt * KEY_RATE;
    if fast {
        distance *= KEY_FAST;
    }
    distance * target_zoom * move_speed
}

impl Panning {
    pub fn origin(&self) -> Option<Vec2> {
        self.origin
    }

    pub fn clear(&mut self, frame: u32) {
        self.origin = None;
        if self.activated && frame > self.activated_frame + DRAG_HOLD_FRAMES {
            self.activated = false;
        }
    }

    pub fn drag(&mut self, position: &mut Vec2, input: DragInput, map_tiles: Vec2, frame: u32) {
        self.running = false;

        if !input.held {
            self.clear(frame);
            return;
        }

        if input.pressed && input.over_ui {
            self.clear(frame);
            return;
        }

        let Some(offset) = input.pointer else {
            return;
        };

        if input.pressed && self.origin.is_none() {
            self.origin = Some(*position + offset);
        }

        let Some(origin) = self.origin else {
            return;
        };

        self.running = true;

        let before = *position;
        let pointer = before + offset;

        if origin.distance(pointer) > DRAG_ACTIVATION {
            self.activated = true;
            self.activated_frame = frame;
        }

        *position = origin - offset;

        let moved = *position - before;
        if moved.length() > DRAG_STILL {
            self.velocity += moved * DRAG_CARRY;
            self.mouse_last = true;
        } else {
            self.velocity = Vec2::ZERO;
        }

        *position = clamp_to_map(*position, map_tiles);
    }

    pub fn decay(&self, dt: f32) -> f32 {
        if self.mouse_last {
            GLIDE_DECAY.powf(dt / REFERENCE_FRAME)
        } else {
            GLIDE_DECAY
        }
    }

    pub fn glide(&mut self, position: &mut Vec2, right_held: bool, map_tiles: Vec2, dt: f32) {
        if self.velocity == Vec2::ZERO {
            return;
        }

        self.velocity *= self.decay(dt);

        if self.velocity.x.abs() < GLIDE_STOP {
            self.velocity.x = 0.0;
        }
        if self.velocity.y.abs() < GLIDE_STOP {
            self.velocity.y = 0.0;
        }

        if !right_held {
            *position += self.velocity;
            *position = clamp_to_map(*position, map_tiles);
        }
    }

    pub fn push(&mut self, heading: Heading, distance: f32, target_zoom: f32, move_max: f32) {
        self.velocity += heading.axis() * distance;

        let limit = target_zoom * move_max;
        self.velocity = Vec2::new(
            self.velocity.x.clamp(-limit, limit),
            self.velocity.y.clamp(-limit, limit),
        );
        self.mouse_last = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAP: Vec2 = Vec2::new(256.0, 256.0);

    fn held(pointer: Vec2) -> DragInput {
        DragInput {
            pressed: false,
            held: true,
            over_ui: false,
            pointer: Some(pointer),
        }
    }

    fn press(pointer: Vec2) -> DragInput {
        DragInput {
            pressed: true,
            ..held(pointer)
        }
    }

    #[test]
    fn the_camera_centre_is_held_inside_the_map_rectangle() {
        assert_eq!(
            clamp_to_map(Vec2::new(500.0, -20.0), MAP),
            Vec2::new(128.0, -20.0)
        );
        assert_eq!(
            clamp_to_map(Vec2::new(-500.0, 900.0), MAP),
            Vec2::new(-128.0, 128.0)
        );
        assert_eq!(
            clamp_to_map(Vec2::new(40.0, 400.0), Vec2::new(64.0, 512.0)),
            Vec2::new(32.0, 256.0)
        );
    }

    #[test]
    fn a_new_world_opens_on_the_middle_tile() {
        assert_eq!(opening_position(MAP), Vec2::ZERO);
        assert_eq!(
            opening_position(Vec2::new(65.0, 64.0)),
            Vec2::new(-0.5, 0.0)
        );
    }

    #[test]
    fn the_grabbed_spot_stays_under_the_pointer() {
        let mut panning = Panning::default();
        let mut position = Vec2::new(10.0, 10.0);

        panning.drag(&mut position, press(Vec2::new(5.0, 0.0)), MAP, 1);
        let grabbed = Vec2::new(15.0, 10.0);
        assert_eq!(panning.origin(), Some(grabbed));

        panning.drag(&mut position, held(Vec2::new(-5.0, 3.0)), MAP, 2);
        assert_eq!(position + Vec2::new(-5.0, 3.0), grabbed);
        assert!(panning.running && panning.activated);
    }

    #[test]
    fn a_press_over_the_interface_never_grabs_the_map() {
        let mut panning = Panning::default();
        let mut position = Vec2::ZERO;

        panning.drag(
            &mut position,
            DragInput {
                over_ui: true,
                ..press(Vec2::new(4.0, 4.0))
            },
            MAP,
            1,
        );
        panning.drag(&mut position, held(Vec2::new(-20.0, 0.0)), MAP, 2);

        assert_eq!(position, Vec2::ZERO);
        assert!(!panning.running);
        assert_eq!(panning.origin(), None);
    }

    #[test]
    fn dragging_carries_a_fifth_of_each_step_into_the_glide() {
        let mut panning = Panning::default();
        let mut position = Vec2::ZERO;

        panning.drag(&mut position, press(Vec2::ZERO), MAP, 1);
        assert_eq!(panning.velocity, Vec2::ZERO);

        panning.drag(&mut position, held(Vec2::new(-2.0, 0.0)), MAP, 2);
        assert_eq!(position, Vec2::new(2.0, 0.0));
        assert_eq!(panning.velocity, Vec2::new(0.4, 0.0));
        assert!(panning.mouse_last);

        panning.drag(&mut position, held(Vec2::new(-2.0, 0.0)), MAP, 3);
        assert_eq!(panning.velocity, Vec2::ZERO);
    }

    #[test]
    fn the_glide_waits_for_the_right_button_and_then_coasts_to_a_stop() {
        let mut panning = Panning::default();
        let mut position = Vec2::ZERO;
        let dt = REFERENCE_FRAME;

        panning.drag(&mut position, press(Vec2::ZERO), MAP, 1);
        for frame in 2..60 {
            let pointer = Vec2::new(-(frame as f32), 0.0);
            panning.drag(&mut position, held(pointer), MAP, frame);
            panning.glide(&mut position, true, MAP, dt);
        }
        let released = position;
        let carried = panning.velocity;
        assert!((carried.x - 0.8).abs() < 1e-3, "{carried}");

        let mut frames = 0;
        panning.drag(&mut position, DragInput::default(), MAP, 60);
        while panning.velocity != Vec2::ZERO {
            panning.glide(&mut position, false, MAP, dt);
            frames += 1;
        }

        let coasted = position.x - released.x;
        assert!((coasted - 3.2).abs() < 0.05, "{coasted}");
        assert!((18..=24).contains(&frames), "{frames}");
    }

    #[test]
    fn the_mouse_glide_decays_by_wall_time_and_the_key_glide_by_frame() {
        let mut panning = Panning {
            mouse_last: true,
            ..default()
        };
        assert!((panning.decay(REFERENCE_FRAME) - 0.8).abs() < 1e-6);
        assert!((panning.decay(REFERENCE_FRAME * 2.0) - 0.64).abs() < 1e-6);

        panning.mouse_last = false;
        assert_eq!(panning.decay(REFERENCE_FRAME * 2.0), 0.8);
    }

    #[test]
    fn holding_still_kills_the_glide() {
        let mut panning = Panning::default();
        let mut position = Vec2::ZERO;

        panning.drag(&mut position, press(Vec2::ZERO), MAP, 1);
        panning.drag(&mut position, held(Vec2::new(-3.0, 0.0)), MAP, 2);
        assert_ne!(panning.velocity, Vec2::ZERO);
        panning.drag(&mut position, held(Vec2::new(-3.0, 0.0)), MAP, 3);
        assert_eq!(panning.velocity, Vec2::ZERO);
    }

    #[test]
    fn a_drag_stays_activated_for_two_frames_after_release() {
        let mut panning = Panning::default();
        let mut position = Vec2::ZERO;

        panning.drag(&mut position, press(Vec2::ZERO), MAP, 10);
        panning.drag(&mut position, held(Vec2::new(1.0, 0.0)), MAP, 11);
        assert!(panning.activated);

        panning.drag(&mut position, DragInput::default(), MAP, 12);
        panning.drag(&mut position, DragInput::default(), MAP, 13);
        assert!(panning.activated);
        panning.drag(&mut position, DragInput::default(), MAP, 14);
        assert!(!panning.activated);
    }

    #[test]
    fn dragging_past_the_edge_stops_at_the_map_rectangle() {
        let mut panning = Panning::default();
        let mut position = Vec2::new(120.0, 0.0);

        panning.drag(&mut position, press(Vec2::ZERO), MAP, 1);
        panning.drag(&mut position, held(Vec2::new(-50.0, 0.0)), MAP, 2);

        assert_eq!(position, Vec2::new(128.0, 0.0));
    }

    #[test]
    fn key_steps_scale_with_zoom_and_shift_makes_them_faster() {
        let slow = key_distance(REFERENCE_FRAME, false, 100.0, 0.01);
        let fast = key_distance(REFERENCE_FRAME, true, 100.0, 0.01);

        assert!((slow - 55.0 / 60.0).abs() < 1e-5, "{slow}");
        assert!((fast / slow - 2.5).abs() < 1e-5);
        assert!((key_distance(REFERENCE_FRAME, false, 50.0, 0.01) * 2.0 - slow).abs() < 1e-5);
    }

    #[test]
    fn holding_a_key_cruises_at_four_steps_a_frame() {
        let mut panning = Panning::default();
        let mut position = Vec2::ZERO;
        let big = Vec2::splat(1.0e6);
        let step = key_distance(REFERENCE_FRAME, false, 100.0, 0.01);

        for _ in 0..200 {
            panning.glide(&mut position, false, big, REFERENCE_FRAME);
            panning.push(Heading::Right, step, 100.0, 0.06);
        }

        let before = position.x;
        panning.glide(&mut position, false, big, REFERENCE_FRAME);
        assert!((position.x - before - 4.0 * step).abs() < 1e-3);
    }

    #[test]
    fn the_key_glide_is_capped_by_the_zoom() {
        let mut panning = Panning::default();
        let step = key_distance(REFERENCE_FRAME, true, 100.0, 0.01);

        for _ in 0..50 {
            panning.push(Heading::Up, step, 100.0, 0.06);
            panning.push(Heading::Left, step, 100.0, 0.06);
        }

        assert_eq!(panning.velocity, Vec2::new(-6.0, 6.0));
        assert!(!panning.mouse_last);
    }

    #[test]
    fn tiny_glides_snap_to_zero_axis_by_axis() {
        let mut panning = Panning {
            velocity: Vec2::new(0.011, 3.0),
            ..default()
        };
        let mut position = Vec2::ZERO;

        panning.glide(&mut position, false, MAP, REFERENCE_FRAME);

        assert_eq!(panning.velocity.x, 0.0);
        assert!((panning.velocity.y - 2.4).abs() < 1e-5);
        assert!((position.y - 2.4).abs() < 1e-5);
        assert_eq!(position.x, 0.0);
    }
}
