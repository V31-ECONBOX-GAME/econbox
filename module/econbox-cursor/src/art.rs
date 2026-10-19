use crate::look::LOOKS;
use bevy::prelude::*;

pub const SIDE: usize = 25;
pub const HOTSPOT: (u16, u16) = (0, 0);

type Rgb = [u8; 3];

const OUTLINE: Rgb = [0x3a, 0x1e, 0x16];
const SHADOW: [u8; 4] = [0x10, 0x08, 0x20, 0x5a];
const GLOW: Rgb = [0xff, 0xd8, 0x4a];
const SPARK_CORE: Rgb = [0xff, 0xff, 0xff];
const SPARK_ARM: Rgb = [0xff, 0xe8, 0x7a];
const GRAIN: Rgb = [0xff, 0xf0, 0xa8];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stuff {
    Skin,
    Cuff,
    Gold,
    Steel,
    Brass,
    Wood,
    Gem,
    Glass,
    Rim,
}

impl Stuff {
    fn tones(self) -> [Rgb; 4] {
        match self {
            Stuff::Skin => [
                [0xee, 0xb0, 0x84],
                [0xfb, 0xd2, 0xa8],
                [0xcc, 0x86, 0x5c],
                [0xa8, 0x62, 0x42],
            ],
            Stuff::Cuff => [
                [0xee, 0xec, 0xe2],
                [0xff, 0xff, 0xff],
                [0xc0, 0xbc, 0xb0],
                [0xa0, 0x9c, 0x92],
            ],
            Stuff::Gold => [
                [0xf0, 0xc0, 0x48],
                [0xff, 0xe4, 0x88],
                [0xc0, 0x8a, 0x28],
                [0x9a, 0x68, 0x1c],
            ],
            Stuff::Steel => [
                [0xc8, 0xd0, 0xd8],
                [0xf4, 0xf8, 0xfc],
                [0x8a, 0x94, 0xa0],
                [0x6a, 0x72, 0x7e],
            ],
            Stuff::Brass => [
                [0xd8, 0xa0, 0x40],
                [0xf6, 0xd0, 0x78],
                [0x9c, 0x6a, 0x22],
                [0x7a, 0x50, 0x18],
            ],
            Stuff::Wood => [
                [0x8a, 0x58, 0x34],
                [0xb0, 0x7a, 0x4c],
                [0x60, 0x3a, 0x20],
                [0x4a, 0x2c, 0x18],
            ],
            Stuff::Gem => [
                [0xd8, 0x30, 0x48],
                [0xff, 0x8a, 0x9a],
                [0x98, 0x18, 0x2c],
                [0x70, 0x10, 0x20],
            ],
            Stuff::Glass => [
                [0xa8, 0xdc, 0xf0],
                [0xf0, 0xfc, 0xff],
                [0x78, 0xb0, 0xcc],
                [0x60, 0x94, 0xb0],
            ],
            Stuff::Rim => [
                [0x50, 0x5a, 0x6a],
                [0x8a, 0x96, 0xa8],
                [0x34, 0x3a, 0x46],
                [0x28, 0x2c, 0x36],
            ],
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Shape {
    Capsule(Vec2, Vec2, f32),
    Ellipse(Vec2, Vec2, f32),
    Circle(Vec2, f32),
}

impl Shape {
    fn contains(self, p: Vec2) -> bool {
        match self {
            Shape::Capsule(a, b, r) => {
                let ab = b - a;
                let t = ((p - a).dot(ab) / ab.length_squared().max(f32::EPSILON)).clamp(0.0, 1.0);
                p.distance(a + ab * t) <= r
            }
            Shape::Ellipse(c, radii, turn) => {
                let d = p - c;
                let (sin, cos) = turn.sin_cos();
                let local = Vec2::new(d.x * cos + d.y * sin, -d.x * sin + d.y * cos);
                (local / radii).length_squared() <= 1.0
            }
            Shape::Circle(c, r) => p.distance(c) <= r,
        }
    }

    fn shifted(self, by: Vec2) -> Self {
        match self {
            Shape::Capsule(a, b, r) => Shape::Capsule(a + by, b + by, r),
            Shape::Ellipse(c, radii, turn) => Shape::Ellipse(c + by, radii, turn),
            Shape::Circle(c, r) => Shape::Circle(c + by, r),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Part {
    shape: Shape,
    stuff: Stuff,
    crease: bool,
}

fn part(shape: Shape, stuff: Stuff) -> Part {
    Part {
        shape,
        stuff,
        crease: false,
    }
}

fn finger(shape: Shape) -> Part {
    Part {
        shape,
        stuff: Stuff::Skin,
        crease: true,
    }
}

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

const DIAGONAL: f32 = -std::f32::consts::FRAC_PI_4;

fn pointing(cuff: Stuff) -> Vec<Part> {
    vec![
        part(Shape::Capsule(v(15.2, 22.6), v(22.6, 15.2), 2.3), cuff),
        part(
            Shape::Ellipse(v(13.6, 14.6), v(7.0, 5.2), DIAGONAL),
            Stuff::Skin,
        ),
        finger(Shape::Capsule(v(11.6, 7.9), v(14.3, 10.6), 1.9)),
        finger(Shape::Capsule(v(14.7, 9.8), v(16.9, 12.3), 1.9)),
        finger(Shape::Capsule(v(17.4, 12.1), v(19.0, 14.0), 1.7)),
        finger(Shape::Capsule(v(2.2, 2.2), v(9.8, 9.8), 2.0)),
        finger(Shape::Capsule(v(7.6, 12.8), v(11.2, 15.8), 1.9)),
    ]
}

fn open_hand() -> Vec<Part> {
    vec![
        part(
            Shape::Capsule(v(15.6, 23.0), v(23.0, 15.6), 2.3),
            Stuff::Cuff,
        ),
        part(
            Shape::Ellipse(v(14.2, 15.2), v(6.6, 5.6), DIAGONAL),
            Stuff::Skin,
        ),
        finger(Shape::Capsule(v(8.6, 12.6), v(3.4, 8.9), 1.7)),
        finger(Shape::Capsule(v(10.4, 10.7), v(5.2, 5.4), 1.7)),
        finger(Shape::Capsule(v(12.2, 9.1), v(8.3, 3.8), 1.7)),
        finger(Shape::Capsule(v(13.9, 7.9), v(12.0, 3.4), 1.5)),
        finger(Shape::Capsule(v(8.4, 15.6), v(3.8, 16.8), 1.8)),
    ]
}

fn fist(extra: &[Part]) -> Vec<Part> {
    let mut parts = vec![
        part(
            Shape::Capsule(v(15.4, 23.0), v(23.0, 15.4), 2.3),
            Stuff::Cuff,
        ),
        part(
            Shape::Ellipse(v(13.8, 15.2), v(6.6, 5.6), DIAGONAL),
            Stuff::Skin,
        ),
        finger(Shape::Circle(v(8.6, 12.2), 2.0)),
        finger(Shape::Circle(v(10.3, 10.2), 2.0)),
        finger(Shape::Circle(v(12.2, 8.6), 2.0)),
        finger(Shape::Circle(v(14.2, 7.4), 1.8)),
    ];
    parts.extend_from_slice(extra);
    parts
}

fn grab() -> Vec<Part> {
    fist(&[finger(Shape::Capsule(v(8.8, 16.4), v(12.6, 13.2), 1.6))])
}

fn little_finger() -> Vec<Part> {
    fist(&[
        finger(Shape::Capsule(v(15.4, 7.0), v(19.6, 2.4), 1.3)),
        finger(Shape::Capsule(v(8.8, 16.4), v(12.6, 13.2), 1.6)),
    ])
}

fn pinch() -> Vec<Part> {
    vec![
        part(
            Shape::Capsule(v(15.4, 23.0), v(23.0, 15.4), 2.3),
            Stuff::Cuff,
        ),
        part(
            Shape::Ellipse(v(13.8, 15.2), v(6.6, 5.6), DIAGONAL),
            Stuff::Skin,
        ),
        finger(Shape::Circle(v(12.2, 8.6), 2.0)),
        finger(Shape::Circle(v(14.2, 7.4), 1.8)),
        finger(Shape::Capsule(v(10.4, 10.0), v(5.6, 5.4), 1.7)),
        finger(Shape::Capsule(v(9.0, 14.6), v(5.0, 7.6), 1.7)),
    ]
}

fn sword(by: Vec2) -> Vec<Part> {
    [
        part(
            Shape::Capsule(v(2.0, 2.0), v(13.4, 13.4), 1.3),
            Stuff::Steel,
        ),
        part(
            Shape::Capsule(v(11.4, 17.4), v(17.4, 11.4), 1.2),
            Stuff::Brass,
        ),
        part(
            Shape::Capsule(v(15.6, 15.6), v(19.4, 19.4), 1.1),
            Stuff::Wood,
        ),
        part(Shape::Circle(v(20.6, 20.6), 1.7), Stuff::Gem),
    ]
    .into_iter()
    .map(|piece| Part {
        shape: piece.shape.shifted(by),
        ..piece
    })
    .collect()
}

fn magnifier() -> Vec<Part> {
    vec![
        part(
            Shape::Capsule(v(10.6, 10.6), v(15.2, 15.2), 1.5),
            Stuff::Wood,
        ),
        part(
            Shape::Capsule(v(17.4, 24.4), v(24.4, 17.4), 2.0),
            Stuff::Cuff,
        ),
        part(
            Shape::Ellipse(v(18.0, 18.6), v(5.0, 4.0), DIAGONAL),
            Stuff::Skin,
        ),
        finger(Shape::Circle(v(14.4, 16.6), 1.7)),
        finger(Shape::Circle(v(16.2, 14.6), 1.7)),
        part(Shape::Circle(v(6.5, 6.5), 5.6), Stuff::Rim),
        part(Shape::Circle(v(6.5, 6.5), 4.1), Stuff::Glass),
    ]
}

#[derive(Debug, Clone, Copy)]
enum Extra {
    None,
    Glow,
    Spark(Vec2, i32),
    Grains(i32),
}

fn sheet(index: usize) -> (Vec<Part>, Extra) {
    let tip = v(2.0, 2.0);
    let pinkie_tip = v(20.0, 2.0);

    match index {
        0 => (pointing(Stuff::Cuff), Extra::None),
        1 => (pointing(Stuff::Cuff), Extra::Glow),
        2..=5 => (pointing(Stuff::Cuff), Extra::Spark(tip, 5 - index as i32)),
        6 => (open_hand(), Extra::None),
        7 => (grab(), Extra::None),
        8 => (little_finger(), Extra::None),
        9 | 10 => (little_finger(), Extra::Glow),
        11 => (little_finger(), Extra::Spark(pinkie_tip, 2)),
        12 => (little_finger(), Extra::Spark(pinkie_tip, 1)),
        13..=16 => (pinch(), Extra::Grains(index as i32 - 13)),
        17..=21 => {
            let step = index as i32 - 17;
            let around = [
                v(2.0, 2.0),
                v(4.5, 1.5),
                v(1.5, 4.5),
                v(3.5, 3.5),
                v(1.0, 1.0),
            ];
            let size = [2, 1, 1, 2, 0][step as usize];
            (
                pointing(Stuff::Cuff),
                Extra::Spark(around[step as usize], size),
            )
        }
        22..=25 => {
            let by = [v(2.0, 2.0), v(1.0, 1.0), v(0.0, 0.0), v(2.0, 1.0)][index - 22];
            (sword(by), Extra::None)
        }
        26 => (magnifier(), Extra::None),
        _ => (pointing(Stuff::Gold), Extra::None),
    }
}

fn fill(parts: &[Part]) -> Vec<Option<usize>> {
    let mut grid = vec![None; SIDE * SIDE];

    for y in 0..SIDE {
        for x in 0..SIDE {
            let p = v(x as f32 + 0.5, y as f32 + 0.5);
            for (at, piece) in parts.iter().enumerate() {
                if piece.shape.contains(p) {
                    grid[y * SIDE + x] = Some(at);
                }
            }
        }
    }

    grid
}

fn at(grid: &[Option<usize>], x: i32, y: i32) -> Option<usize> {
    if x < 0 || y < 0 || x >= SIDE as i32 || y >= SIDE as i32 {
        return None;
    }
    grid[y as usize * SIDE + x as usize]
}

fn put(pixels: &mut [[u8; 4]], x: i32, y: i32, rgb: Rgb) {
    if x < 0 || y < 0 || x >= SIDE as i32 || y >= SIDE as i32 {
        return;
    }
    pixels[y as usize * SIDE + x as usize] = [rgb[0], rgb[1], rgb[2], 0xff];
}

fn spark(pixels: &mut [[u8; 4]], centre: Vec2, size: i32) {
    let cx = centre.x.floor() as i32;
    let cy = centre.y.floor() as i32;

    for arm in 1..=size {
        put(pixels, cx - arm, cy, SPARK_ARM);
        put(pixels, cx + arm, cy, SPARK_ARM);
        put(pixels, cx, cy - arm, SPARK_ARM);
        put(pixels, cx, cy + arm, SPARK_ARM);
    }
    if size >= 2 {
        put(pixels, cx - 1, cy - 1, SPARK_ARM);
        put(pixels, cx + 1, cy + 1, SPARK_ARM);
        put(pixels, cx + 1, cy - 1, SPARK_ARM);
        put(pixels, cx - 1, cy + 1, SPARK_ARM);
    }
    put(pixels, cx, cy, SPARK_CORE);
}

const NEIGHBOURS: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

pub fn draw(index: usize) -> Vec<[u8; 4]> {
    let (parts, extra) = sheet(index.min(LOOKS - 1));
    let grid = fill(&parts);
    let mut pixels = vec![[0u8; 4]; SIDE * SIDE];

    let solid = |x: i32, y: i32| at(&grid, x, y).is_some();
    let edge =
        |x: i32, y: i32| !solid(x, y) && NEIGHBOURS.iter().any(|(dx, dy)| solid(x + dx, y + dy));
    let body = |x: i32, y: i32| solid(x, y) || edge(x, y);

    for y in 0..SIDE as i32 {
        for x in 0..SIDE as i32 {
            if !body(x, y) && body(x - 1, y - 1) {
                pixels[y as usize * SIDE + x as usize] = SHADOW;
            }
        }
    }

    if matches!(extra, Extra::Glow) {
        for y in 0..SIDE as i32 {
            for x in 0..SIDE as i32 {
                if !body(x, y) && NEIGHBOURS.iter().any(|(dx, dy)| edge(x + dx, y + dy)) {
                    put(&mut pixels, x, y, GLOW);
                }
            }
        }
    }

    for y in 0..SIDE as i32 {
        for x in 0..SIDE as i32 {
            if edge(x, y) {
                put(&mut pixels, x, y, OUTLINE);
                continue;
            }

            let Some(own) = at(&grid, x, y) else {
                continue;
            };
            let piece = parts[own];
            let [base, light, shade, crease] = piece.stuff.tones();
            let same = |dx: i32, dy: i32| {
                at(&grid, x + dx, y + dy).is_some_and(|other| parts[other].stuff == piece.stuff)
            };
            let seam = |dx: i32, dy: i32| {
                at(&grid, x + dx, y + dy).is_some_and(|other| {
                    other != own
                        && parts[other].stuff == piece.stuff
                        && (piece.crease || parts[other].crease)
                        && other > own
                })
            };

            let tone = if seam(1, 0) || seam(0, 1) {
                crease
            } else if !same(-1, -1) || !same(0, -1) {
                light
            } else if !same(1, 1) || !same(1, 0) {
                shade
            } else {
                base
            };
            put(&mut pixels, x, y, tone);
        }
    }

    match extra {
        Extra::Spark(centre, size) => spark(&mut pixels, centre, size),
        Extra::Grains(step) => {
            let fall = [(3, 9), (5, 11), (2, 12), (4, 14), (1, 15), (3, 17)];
            for (i, (x, y)) in fall.iter().enumerate() {
                if (i as i32 + step) % 2 == 0 {
                    put(&mut pixels, *x, *y + step % 2, GRAIN);
                }
            }
        }
        Extra::None | Extra::Glow => {}
    }

    pixels
}

pub fn rgba(index: usize) -> Vec<u8> {
    draw(index).into_iter().flatten().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alpha(pixels: &[[u8; 4]], x: usize, y: usize) -> u8 {
        pixels[y * SIDE + x][3]
    }

    #[test]
    fn every_look_is_a_full_square_with_something_drawn() {
        for index in 0..LOOKS {
            let pixels = draw(index);
            assert_eq!(pixels.len(), SIDE * SIDE);
            let opaque = pixels.iter().filter(|pixel| pixel[3] == 0xff).count();
            assert!(opaque > 40, "look {index} has only {opaque} pixels");
        }
    }

    #[test]
    fn the_pointing_finger_touches_the_hotspot() {
        let pixels = draw(0);

        assert_eq!(alpha(&pixels, 0, 0), 0xff);
        assert_eq!(alpha(&pixels, 24, 0), 0);
        assert_eq!(alpha(&pixels, 0, 24), 0);
    }

    #[test]
    fn the_press_glow_and_the_sparkles_change_the_picture() {
        let rest = draw(0);

        for index in 1..6 {
            assert_ne!(draw(index), rest, "look {index}");
        }
        assert_ne!(draw(13), draw(14));
        assert_ne!(draw(22), draw(24));
        assert_ne!(draw(27), rest);
    }

    #[test]
    fn the_raw_bytes_are_rgba() {
        assert_eq!(rgba(7).len(), SIDE * SIDE * 4);
    }
}
