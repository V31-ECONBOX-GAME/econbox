use bevy::prelude::Resource;

pub const MAIN_ZOOM: f32 = 240.0;
pub const TRANSITION_SPEED: f32 = 3.5;
pub const LOW_RES_FLOOR: f32 = 0.9;
pub const SWITCH_FLASH: f32 = 0.1;
pub const SWITCH_FADE: f32 = 0.1;
pub const SWITCH_GREY: f32 = 0.741_176_5;
pub const SHADOW_ZOOM: f32 = 0.85;
pub const BUILDING_HIDE: f32 = 0.3;
pub const SHADOW_ALPHA: f32 = 103.0 / 255.0;

pub fn camera_multiplier(value: f32, pixel_width: f32, pixel_height: f32) -> f32 {
    let aspect = pixel_width / pixel_height.max(1.0);
    let mut longest = if pixel_width > pixel_height {
        pixel_width
    } else {
        pixel_height
    };

    if aspect > 2.0 {
        longest /= aspect * 0.5;
    }

    value * (pixel_height / longest) * 0.5
}

pub fn bound_low(pixel_width: f32, pixel_height: f32) -> f32 {
    camera_multiplier(MAIN_ZOOM, pixel_width, pixel_height)
}

pub fn ease_in_circ(value: f32) -> f32 {
    1.0 - (1.0 - value * value).max(0.0).sqrt()
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct MapMode {
    pub low_res: bool,
    pub factor: f32,
    pub zoom: f32,
    pub flash: f32,
    pub switched: bool,
    pub shadow_bound: f32,
}

impl Default for MapMode {
    fn default() -> Self {
        Self {
            low_res: true,
            factor: 1.0,
            zoom: MAIN_ZOOM,
            flash: 0.0,
            switched: false,
            shadow_bound: MAIN_ZOOM * SHADOW_ZOOM,
        }
    }
}

impl MapMode {
    pub fn set_zoom(&mut self, orthographic: f32, pixel_width: f32, pixel_height: f32) {
        self.zoom = orthographic;
        self.shadow_bound = camera_multiplier(MAIN_ZOOM * SHADOW_ZOOM, pixel_width, pixel_height);
        let low = orthographic > bound_low(pixel_width, pixel_height);

        self.switched = low != self.low_res;
        if self.switched {
            if low && self.tiles_on() {
                self.flash = SWITCH_FLASH;
            }
            self.low_res = low;
        }
    }

    pub fn advance(&mut self, seconds: f32) {
        if self.low_res && self.factor < 1.0 {
            self.factor = (self.factor + seconds * TRANSITION_SPEED).min(1.0);
        } else if !self.low_res && self.factor > 0.0 {
            self.factor = (self.factor - seconds * TRANSITION_SPEED).max(0.0);
        }

        self.flash = (self.flash - seconds * SWITCH_FADE).max(0.0);
    }

    pub fn snap(&mut self) {
        self.factor = if self.low_res { 1.0 } else { 0.0 };
    }

    pub fn tiles_on(&self) -> bool {
        !self.low_res
    }

    pub fn map_on(&self) -> bool {
        self.low_res || self.factor > 0.0
    }

    pub fn map_alpha(&self) -> f32 {
        if self.low_res {
            self.factor.max(LOW_RES_FLOOR)
        } else {
            self.factor
        }
    }

    pub fn edges_alpha(&self) -> f32 {
        self.factor
    }

    pub fn tween_buildings(&self) -> f32 {
        1.0 - ease_in_circ(self.factor)
    }

    pub fn full_low_res(&self) -> bool {
        self.low_res && self.factor >= 1.0
    }

    pub fn render_buildings(&self) -> bool {
        let tween = self.tween_buildings();
        if self.low_res && tween < BUILDING_HIDE {
            return false;
        }
        !self.full_low_res() && tween != 0.0
    }

    pub fn shadows_on(&self) -> bool {
        self.zoom <= self.shadow_bound && self.render_buildings()
    }

    pub fn shadow_alpha(&self) -> f32 {
        SHADOW_ALPHA * (4.0 - self.tween_buildings() * 3.0)
    }
}

pub const RUNUP_INTERVAL: f32 = 0.01;
pub const RUNUP_SPEED: f32 = 0.6;
pub const RUNUP_MIN: f32 = 0.02;
pub const RUNUP_MAX: f32 = 0.7;

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct Runup {
    pub alpha: f32,
    pub timer: f32,
    pub rising: bool,
}

impl Default for Runup {
    fn default() -> Self {
        Self {
            alpha: 0.4,
            timer: 0.0,
            rising: true,
        }
    }
}

impl Runup {
    pub fn update(&mut self, elapsed: f32) {
        if self.timer > 0.0 {
            self.timer -= elapsed;
            return;
        }

        self.timer = RUNUP_INTERVAL;
        if self.rising {
            self.alpha += elapsed * RUNUP_SPEED;
            if self.alpha >= RUNUP_MAX {
                self.alpha = RUNUP_MAX;
                self.rising = false;
            }
        } else {
            self.alpha -= elapsed * RUNUP_SPEED;
            if self.alpha <= RUNUP_MIN {
                self.alpha = RUNUP_MIN;
                self.rising = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wide_window_switches_to_the_map_once_two_hundred_and_forty_tiles_fit_across() {
        let bound = bound_low(1600.0, 1000.0);

        assert!((bound - 75.0).abs() < 1e-4, "{bound}");
        assert!((bound * 2.0 * 1.6 - 240.0).abs() < 1e-3);
    }

    #[test]
    fn a_very_wide_window_and_a_tall_one_use_their_own_bounds() {
        assert!((bound_low(3000.0, 1000.0) - 60.0).abs() < 1e-4);
        assert!((bound_low(800.0, 1200.0) - 120.0).abs() < 1e-4);
        assert!((bound_low(1000.0, 1000.0) - 120.0).abs() < 1e-4);
    }

    #[test]
    fn zooming_out_past_the_bound_turns_the_map_on_and_back() {
        let mut mode = MapMode {
            low_res: false,
            factor: 0.0,
            ..MapMode::default()
        };

        mode.set_zoom(70.0, 1600.0, 1000.0);
        assert!(!mode.low_res && !mode.switched);

        mode.set_zoom(80.0, 1600.0, 1000.0);
        assert!(mode.low_res && mode.switched);
        assert_eq!(mode.flash, SWITCH_FLASH);
        assert_eq!(mode.map_alpha(), LOW_RES_FLOOR);
        assert_eq!(mode.edges_alpha(), 0.0);

        mode.advance(1.0 / 7.0);
        assert!((mode.factor - 0.5).abs() < 1e-5);
        assert!((mode.map_alpha() - LOW_RES_FLOOR).abs() < 1e-5);

        mode.advance(1.0);
        assert_eq!(mode.factor, 1.0);
        assert_eq!(mode.map_alpha(), 1.0);
        assert!(!mode.tiles_on());
        assert_eq!(mode.tween_buildings(), 0.0);

        mode.set_zoom(20.0, 1600.0, 1000.0);
        assert!(mode.tiles_on() && mode.map_on());
        mode.advance(1.0);
        assert!(!mode.map_on());
        assert_eq!(mode.tween_buildings(), 1.0);
    }

    #[test]
    fn the_switch_flash_fades_in_one_second() {
        let mut mode = MapMode {
            low_res: false,
            factor: 0.0,
            ..MapMode::default()
        };
        mode.set_zoom(500.0, 1600.0, 1000.0);
        mode.advance(0.5);
        assert!((mode.flash - 0.05).abs() < 1e-6);
        mode.advance(0.6);
        assert_eq!(mode.flash, 0.0);
    }

    #[test]
    fn the_coast_rim_breathes_between_its_bounds() {
        let mut runup = Runup::default();
        let mut seen_max = false;
        let mut seen_min = false;

        for _ in 0..2000 {
            runup.update(1.0 / 60.0);
            assert!((RUNUP_MIN..=RUNUP_MAX).contains(&runup.alpha));
            seen_max |= runup.alpha == RUNUP_MAX;
            seen_min |= runup.alpha == RUNUP_MIN;
        }

        assert!(seen_max && seen_min);
    }
}
