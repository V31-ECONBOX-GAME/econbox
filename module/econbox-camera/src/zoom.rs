use bevy::prelude::*;

pub const ZOOM_MIN: f32 = 10.0;
pub const ZOOM_MAX_DEFAULT: f32 = 130.0;
pub const ZOOM_MAX_RATIO: f32 = 1.1;
pub const ZOOM_SLACK: f32 = 5.0;
pub const WHEEL_STEP: f32 = 0.2;
pub const KEY_STEP: f32 = 0.05;
pub const FOCUS_ZOOM: f32 = 15.0;

pub fn unity_clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

pub fn zoom_max_for(map_tiles: Vec2) -> f32 {
    let side = if map_tiles.x > map_tiles.y {
        map_tiles.x
    } else {
        map_tiles.y
    };

    (side * ZOOM_MAX_RATIO).trunc()
}

pub fn opening_zoom(screen_px: UVec2) -> u32 {
    if screen_px.x >= screen_px.y {
        screen_px.y / 4
    } else {
        screen_px.x / 4
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Zoom {
    pub size: f32,
    pub target: f32,
    pub max: f32,
}

impl Default for Zoom {
    fn default() -> Self {
        Self {
            size: ZOOM_MAX_DEFAULT,
            target: ZOOM_MAX_DEFAULT,
            max: ZOOM_MAX_DEFAULT,
        }
    }
}

impl Zoom {
    pub fn reset(&mut self, screen_px: UVec2, map_tiles: Vec2) {
        self.max = zoom_max_for(map_tiles);

        let mut opening = opening_zoom(screen_px) as f32;
        if opening > self.max {
            opening = self.max.trunc();
        }

        self.target = opening;
        self.size = unity_clamp(self.target, ZOOM_MIN, self.max);
    }

    pub fn approach(&mut self, dt: f32, speed: f32) {
        self.target = unity_clamp(self.target, ZOOM_MIN, self.max);

        if self.size == self.target {
            return;
        }

        let step = dt * speed * ((self.size - self.target).abs() + ZOOM_SLACK);

        if self.target > self.size {
            self.size += step;
            if self.size > self.target {
                self.size = unity_clamp(self.target, ZOOM_MIN, self.max);
            }
        } else {
            self.size -= step;
            if self.size < self.target {
                self.size = unity_clamp(self.target, ZOOM_MIN, self.max);
            }
        }
    }

    pub fn force(&mut self, size: f32) {
        self.target = size;
        self.approach(0.0, 0.0);
        self.size = self.target;
    }

    pub fn wheel_in(&mut self) {
        self.target -= self.size * WHEEL_STEP;
    }

    pub fn wheel_out(&mut self) {
        self.target += self.size * WHEEL_STEP;
    }

    pub fn key_in(&mut self) {
        self.target -= self.size * KEY_STEP;
    }

    pub fn key_out(&mut self) {
        self.target += self.size * KEY_STEP;
    }

    pub fn settled(&self) -> bool {
        self.size == self.target
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settle(zoom: &mut Zoom, dt: f32) -> usize {
        let mut frames = 0;

        while !zoom.settled() {
            zoom.approach(dt, 5.0);
            frames += 1;
            assert!(frames < 10_000, "{zoom:?}");
        }

        frames
    }

    #[test]
    fn the_zoom_ceiling_is_a_tenth_wider_than_the_longest_side() {
        assert_eq!(zoom_max_for(Vec2::splat(256.0)), 281.0);
        assert_eq!(zoom_max_for(Vec2::new(64.0, 128.0)), 140.0);
        assert_eq!(zoom_max_for(Vec2::splat(1024.0)), 1126.0);
        assert_eq!(zoom_max_for(Vec2::new(448.0, 448.0)), 492.0);
    }

    #[test]
    fn a_new_world_opens_at_a_quarter_of_the_short_screen_side() {
        let mut zoom = Zoom::default();

        zoom.reset(UVec2::new(1024, 768), Vec2::splat(256.0));
        assert_eq!((zoom.size, zoom.target, zoom.max), (192.0, 192.0, 281.0));

        zoom.reset(UVec2::new(2480, 1438), Vec2::splat(256.0));
        assert_eq!((zoom.size, zoom.target), (281.0, 281.0));

        zoom.reset(UVec2::new(2480, 1438), Vec2::splat(1024.0));
        assert_eq!((zoom.size, zoom.target, zoom.max), (359.0, 359.0, 1126.0));

        zoom.reset(UVec2::new(600, 900), Vec2::splat(1024.0));
        assert_eq!(zoom.size, 150.0);
    }

    #[test]
    fn a_tiny_map_still_keeps_the_ten_tile_floor() {
        let mut zoom = Zoom::default();

        zoom.reset(UVec2::new(1920, 1080), Vec2::splat(8.0));
        assert_eq!(zoom.max, 8.0);
        assert_eq!(zoom.target, 8.0);
        assert_eq!(zoom.size, ZOOM_MIN);
    }

    #[test]
    fn one_wheel_notch_moves_the_target_by_a_fifth_of_the_current_size() {
        let mut zoom = Zoom {
            size: 100.0,
            target: 100.0,
            max: 281.0,
        };

        zoom.wheel_in();
        assert_eq!(zoom.target, 80.0);
        zoom.wheel_out();
        assert_eq!(zoom.target, 100.0);
        zoom.key_in();
        assert_eq!(zoom.target, 95.0);
        zoom.key_out();
        assert_eq!(zoom.target, 100.0);
    }

    #[test]
    fn the_first_eased_frame_follows_the_worldbox_formula() {
        let mut zoom = Zoom {
            size: 100.0,
            target: 80.0,
            max: 281.0,
        };

        zoom.approach(1.0 / 60.0, 5.0);
        let expected = 100.0 - (1.0 / 60.0) * 5.0 * (20.0 + 5.0);
        assert!((zoom.size - expected).abs() < 1e-4, "{}", zoom.size);

        zoom.target = 200.0;
        let before = zoom.size;
        zoom.approach(1.0 / 60.0, 5.0);
        let expected = before + (1.0 / 60.0) * 5.0 * ((200.0 - before) + 5.0);
        assert!((zoom.size - expected).abs() < 1e-4, "{}", zoom.size);
    }

    #[test]
    fn easing_lands_exactly_on_the_target_without_overshoot() {
        for dt in [1.0 / 120.0, 1.0 / 60.0, 1.0 / 30.0, 0.25] {
            let mut zoom = Zoom {
                size: 281.0,
                target: 12.0,
                max: 281.0,
            };
            let mut last = zoom.size;

            while !zoom.settled() {
                zoom.approach(dt, 5.0);
                assert!(zoom.size <= last && zoom.size >= 12.0, "{dt}: {zoom:?}");
                last = zoom.size;
            }

            assert_eq!(zoom.size, 12.0);
        }
    }

    #[test]
    fn a_wheel_notch_settles_in_about_a_third_of_a_second() {
        let mut zoom = Zoom {
            size: 100.0,
            target: 100.0,
            max: 281.0,
        };

        zoom.wheel_in();
        let frames = settle(&mut zoom, 1.0 / 60.0);

        assert!((15..=30).contains(&frames), "{frames}");
    }

    #[test]
    fn the_target_is_held_between_ten_tiles_and_the_ceiling() {
        let mut zoom = Zoom {
            size: 20.0,
            target: 2.0,
            max: 281.0,
        };

        zoom.approach(1.0 / 60.0, 5.0);
        assert_eq!(zoom.target, ZOOM_MIN);
        settle(&mut zoom, 1.0 / 60.0);
        assert_eq!(zoom.size, ZOOM_MIN);

        zoom.target = 5000.0;
        zoom.approach(1.0 / 60.0, 5.0);
        assert_eq!(zoom.target, 281.0);
        settle(&mut zoom, 1.0 / 60.0);
        assert_eq!(zoom.size, 281.0);
    }

    #[test]
    fn zooming_in_from_the_ceiling_needs_about_fifteen_notches() {
        let mut zoom = Zoom {
            size: 281.0,
            target: 281.0,
            max: 281.0,
        };
        let mut notches = 0;

        while zoom.size > ZOOM_MIN {
            zoom.wheel_in();
            settle(&mut zoom, 1.0 / 60.0);
            notches += 1;
        }

        assert_eq!(notches, 15);
    }

    #[test]
    fn forcing_a_zoom_jumps_straight_there() {
        let mut zoom = Zoom {
            size: 200.0,
            target: 200.0,
            max: 281.0,
        };

        zoom.force(30.0);
        assert_eq!((zoom.size, zoom.target), (30.0, 30.0));

        zoom.force(1.0);
        assert_eq!((zoom.size, zoom.target), (ZOOM_MIN, ZOOM_MIN));
    }

    #[test]
    fn unity_clamp_prefers_the_floor_when_the_range_is_inverted() {
        assert_eq!(unity_clamp(5.0, 10.0, 8.0), 10.0);
        assert_eq!(unity_clamp(9.0, 10.0, 8.0), 10.0);
        assert_eq!(unity_clamp(12.0, 10.0, 8.0), 8.0);
        assert_eq!(unity_clamp(50.0, 10.0, 281.0), 50.0);
    }
}
