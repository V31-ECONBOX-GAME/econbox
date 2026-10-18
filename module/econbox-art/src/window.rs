use crate::shade;
use crate::ui::{BAR_FACE, Picture, Rgba};

const fn rgb(hex: u32) -> Rgba {
    [(hex >> 16) as u8, (hex >> 8) as u8, hex as u8, 255]
}

fn tone(color: [u8; 3], by: f32) -> Rgba {
    let [r, g, b] = shade(color, by);
    [r, g, b, 255]
}

fn ring(x: u32, y: u32, width: u32, height: u32) -> u32 {
    x.min(y).min(width - 1 - x).min(height - 1 - y)
}

fn corner_cut(x: u32, y: u32, width: u32, height: u32, cut: u32) -> bool {
    let near = |v: u32, size: u32| v.min(size - 1 - v);
    near(x, width) + near(y, height) < cut
}

fn hash(x: u32, y: u32, seed: u32) -> u32 {
    let mut h =
        x.wrapping_mul(0x9e37_79b1) ^ y.wrapping_mul(0x85eb_ca77) ^ seed.wrapping_mul(0xc2b2_ae3d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h
}

fn noise(x: f32, seed: u32) -> f32 {
    let cell = x.floor();
    let t = x - cell;
    let a = (hash(cell as u32, 0, seed) % 1000) as f32 / 1000.0;
    let b = (hash(cell as u32 + 1, 0, seed) % 1000) as f32 / 1000.0;
    let s = t * t * (3.0 - 2.0 * t);
    a + (b - a) * s
}

fn mix(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);
    [0, 1, 2].map(|i| (f32::from(a[i]) + (f32::from(b[i]) - f32::from(a[i])) * t) as u8)
}

fn opaque(color: [u8; 3]) -> Rgba {
    [color[0], color[1], color[2], 255]
}

pub fn vertical_tab(selected: bool) -> Picture {
    let (width, height) = (22, 22);
    let mut out = Picture::blank(width, height);
    let face = if selected {
        [0xd0, 0x48, 0x38]
    } else {
        BAR_FACE
    };

    for y in 0..height {
        for x in 0..width {
            if x < 3 && corner_cut(x, y, width, height, 3) {
                continue;
            }

            let edge = x.min(y).min(height - 1 - y);
            let color = match edge {
                0 => tone(face, 0.42),
                1 if x == 1 || y == 1 => tone(face, 1.36),
                1 => tone(face, 0.7),
                _ if x == width - 1 => tone(face, 0.8),
                _ if (x + 2 * y) % 11 == 0 => tone(face, 1.06),
                _ => tone(face, 1.0),
            };
            out.set(x, y, color);
        }
    }

    out
}

pub const SIDEBAR_WIDTH: u32 = 49;
pub const SIDEBAR_HEIGHT: u32 = 105;

pub fn sidebar() -> Picture {
    let mut out = Picture::blank(SIDEBAR_WIDTH, SIDEBAR_HEIGHT);
    let face = BAR_FACE;

    for y in 0..SIDEBAR_HEIGHT {
        for x in 0..SIDEBAR_WIDTH {
            if corner_cut(x, y, SIDEBAR_WIDTH, SIDEBAR_HEIGHT, 3) {
                continue;
            }

            let edge = ring(x, y, SIDEBAR_WIDTH, SIDEBAR_HEIGHT);
            let color = match edge {
                0 => rgb(0x22261f),
                1 if x == 1 || y == 1 => rgb(0x8a9480),
                1 => tone(face, 0.66),
                2 => tone(face, 0.86),
                3..=5 => tone(face, 1.0),
                6 => tone(face, 0.74),
                _ => tone(face, 0.82),
            };
            out.set(x, y, color);
        }
    }

    for (cx, cy) in [
        (3, 3),
        (SIDEBAR_WIDTH - 4, 3),
        (3, SIDEBAR_HEIGHT - 4),
        (SIDEBAR_WIDTH - 4, SIDEBAR_HEIGHT - 4),
    ] {
        out.set(cx, cy, tone(face, 1.4));
    }

    out
}

pub fn header_plate() -> Picture {
    let (width, height) = (40, 22);
    let mut out = Picture::blank(width, height);
    let face = BAR_FACE;

    for y in 0..height {
        for x in 0..width {
            let side = x.min(width - 1 - x);
            if y < 4 && side < 4 - y {
                continue;
            }

            let edge = side.min(y);
            let color = match edge {
                0 => rgb(0x22261f),
                1 if y == 1 || x == 1 => rgb(0x8a9480),
                1 => tone(face, 0.7),
                _ if y == 6 || y == 16 => tone(face, 0.86),
                _ if y > 6 && y < 16 && (x + y) % 5 == 0 => tone(face, 0.94),
                _ => tone(face, 1.0),
            };
            out.set(x, y, color);
        }
    }

    out
}

pub fn tab_band() -> Picture {
    let (width, height) = (24, 22);
    let mut out = Picture::blank(width, height);
    let face = [0x2d, 0x33, 0x2b];

    for y in 0..height {
        for x in 0..width {
            if corner_cut(x, y, width, height, 2) {
                continue;
            }

            let edge = ring(x, y, width, height);
            let color = match edge {
                0 => rgb(0x171a15),
                1 if y == height - 2 => tone(face, 1.5),
                1 => tone(face, 0.7),
                _ => opaque(mix(
                    tone_rgb(face, 1.1),
                    tone_rgb(face, 0.9),
                    y as f32 / height as f32,
                )),
            };
            out.set(x, y, color);
        }
    }

    out
}

fn tone_rgb(color: [u8; 3], by: f32) -> [u8; 3] {
    shade(color, by)
}

pub fn bar_plate(face: [u8; 3], width: u32, height: u32) -> Picture {
    let mut out = Picture::blank(width, height);

    for y in 0..height {
        for x in 0..width {
            if corner_cut(x, y, width, height, 2) {
                continue;
            }

            let edge = ring(x, y, width, height);
            let color = match edge {
                0 => tone(face, 0.42),
                1 if y == 1 => tone(face, 1.34),
                1 if y == height - 2 => tone(face, 0.66),
                1 => tone(face, 0.86),
                _ if y < height / 2 => tone(face, 1.06),
                _ => tone(face, 0.96),
            };
            out.set(x, y, color);
        }
    }

    out
}

pub const RED: [u8; 3] = [0xd2, 0x46, 0x36];
pub const GRAY: [u8; 3] = [0x4a, 0x4e, 0x4a];

pub fn line() -> Picture {
    let mut out = Picture::blank(3, 2);
    for x in 0..3 {
        out.set(x, 0, rgb(0x262b23));
        out.set(x, 1, rgb(0x6e7864));
    }
    out
}

pub fn scroll_track() -> Picture {
    let (width, height) = (12, 24);
    let mut out = Picture::blank(width, height);

    for y in 0..height {
        for x in 0..width {
            if corner_cut(x, y, width, height, 2) {
                continue;
            }

            let edge = ring(x, y, width, height);
            let color = match edge {
                0 => rgb(0x1a1d18),
                1 if x == 1 => rgb(0x15181a),
                _ => rgb(0x262a25),
            };
            out.set(x, y, color);
        }
    }

    out
}

pub fn scroll_handle() -> Picture {
    let (width, height) = (6, 16);
    let mut out = Picture::blank(width, height);

    for y in 0..height {
        for x in 0..width {
            if corner_cut(x, y, width, height, 2) {
                continue;
            }

            let edge = ring(x, y, width, height);
            let color = match edge {
                0 => rgb(0x8a8a8a),
                _ if x == 1 => rgb(0xffffff),
                _ => rgb(0xe6e6e6),
            };
            out.set(x, y, color);
        }
    }

    out
}

pub fn check(on: bool) -> Picture {
    let size = 16;
    let mut out = Picture::blank(size, size);
    let (ink, dark) = if on {
        (rgb(0x6cf05a), rgb(0x1f7a1c))
    } else {
        (rgb(0xf05a4a), rgb(0x7a1f1c))
    };

    let strokes: &[(i32, i32)] = if on {
        &[
            (3, 8),
            (4, 9),
            (5, 10),
            (6, 11),
            (7, 10),
            (8, 9),
            (9, 8),
            (10, 7),
            (11, 6),
            (12, 5),
            (13, 4),
        ]
    } else {
        &[
            (4, 4),
            (5, 5),
            (6, 6),
            (7, 7),
            (8, 8),
            (9, 9),
            (10, 10),
            (11, 11),
            (11, 4),
            (10, 5),
            (9, 6),
            (8, 7),
            (7, 8),
            (6, 9),
            (5, 10),
            (4, 11),
        ]
    };

    for &(x, y) in strokes {
        for (dx, dy) in [(0, 1), (1, 1)] {
            out.set((x + dx) as u32, (y + dy) as u32, dark);
        }
    }
    for &(x, y) in strokes {
        out.set(x as u32, y as u32, ink);
        out.set(x as u32 + 1, y as u32, ink);
    }

    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scene {
    Ember,
    Dusk,
    Grove,
    Tide,
    Void,
}

impl Scene {
    fn palette(self) -> ([u8; 3], [u8; 3], [u8; 3], [u8; 3], [u8; 3]) {
        match self {
            Self::Ember => (
                [0x3a, 0x0f, 0x0c],
                [0xc8, 0x4a, 0x1c],
                [0x24, 0x14, 0x14],
                [0x10, 0x09, 0x0a],
                [0xff, 0xb0, 0x40],
            ),
            Self::Dusk => (
                [0x1a, 0x1c, 0x4a],
                [0xd0, 0x6a, 0x8a],
                [0x2a, 0x24, 0x48],
                [0x14, 0x12, 0x26],
                [0xff, 0xe0, 0xa0],
            ),
            Self::Grove => (
                [0x1c, 0x3c, 0x5a],
                [0x8a, 0xc8, 0xd0],
                [0x2c, 0x5a, 0x34],
                [0x14, 0x2c, 0x1a],
                [0xff, 0xf4, 0xc0],
            ),
            Self::Tide => (
                [0x0c, 0x22, 0x44],
                [0x3a, 0x8a, 0xb8],
                [0x1a, 0x3e, 0x5c],
                [0x0a, 0x1a, 0x2c],
                [0xe0, 0xf4, 0xff],
            ),
            Self::Void => (
                [0x0a, 0x08, 0x1a],
                [0x3a, 0x1c, 0x5a],
                [0x5a, 0x2a, 0x2a],
                [0x1c, 0x10, 0x24],
                [0xff, 0xe0, 0x6a],
            ),
        }
    }

    fn seed(self) -> u32 {
        self as u32 * 7919 + 13
    }
}

pub fn illustration(scene: Scene, width: u32, height: u32) -> Picture {
    let (top, horizon, far, near, light) = scene.palette();
    let seed = scene.seed();
    let mut out = Picture::blank(width, height);
    let (w, h) = (width as f32, height as f32);
    let sun = (w * 0.68, h * 0.3, h * 0.11);

    for y in 0..height {
        for x in 0..width {
            let (fx, fy) = (x as f32, y as f32);
            let mut color = mix(top, horizon, fy / (h * 0.7));

            let star = hash(x, y, seed).is_multiple_of(997) && fy < h * 0.5;
            if star {
                color = mix(color, light, 0.8);
            }

            let reach = ((fx - sun.0).powi(2) + (fy - sun.1).powi(2)).sqrt();
            if reach < sun.2 {
                color = mix(light, horizon, (reach / sun.2).powi(3) * 0.5);
            } else if reach < sun.2 * 2.6 {
                color = mix(color, light, 0.22 * (1.0 - reach / (sun.2 * 2.6)));
            }

            let ridge_far = h * 0.52
                + (noise(fx / (w * 0.09), seed) - 0.5) * h * 0.3
                + (noise(fx / (w * 0.03), seed + 1) - 0.5) * h * 0.06;
            if fy > ridge_far {
                color = mix(far, near, (fy - ridge_far) / (h * 0.5));
            }

            let ridge_near = h * 0.72
                + (noise(fx / (w * 0.14), seed + 2) - 0.5) * h * 0.26
                + (noise(fx / (w * 0.04), seed + 3) - 0.5) * h * 0.05;
            if fy > ridge_near {
                color = mix(near, [0, 0, 0], (fy - ridge_near) / (h * 0.6));
                if fy - ridge_near < 2.0 {
                    color = mix(color, light, 0.25);
                }
            }

            if (x + y).is_multiple_of(2) && hash(x / 3, y / 3, seed + 5).is_multiple_of(5) {
                color = shade(color, 0.94);
            }

            out.set(x, y, opaque(color));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_scene_paints_every_pixel() {
        for scene in [
            Scene::Ember,
            Scene::Dusk,
            Scene::Grove,
            Scene::Tide,
            Scene::Void,
        ] {
            let picture = illustration(scene, 64, 48);
            assert!(picture.pixels.iter().all(|px| px[3] == 255), "{scene:?}");
        }
    }

    #[test]
    fn a_selected_tab_is_red_and_an_idle_one_is_not() {
        let red = vertical_tab(true).get(10, 10);
        let idle = vertical_tab(false).get(10, 10);
        assert!(red[0] > red[1] && red[0] > red[2]);
        assert!(idle[1] >= idle[0]);
    }

    #[test]
    fn the_check_and_the_cross_both_have_ink() {
        assert!(check(true).pixels.iter().any(|px| px[3] > 0));
        assert!(check(false).pixels.iter().any(|px| px[3] > 0));
    }
}
