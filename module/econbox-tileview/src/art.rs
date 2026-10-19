use crate::look::{self, Look, Paint, Style};
use econbox_tile::{Flora, Ground, Ore, Overlay};
use std::collections::HashMap;
use std::sync::OnceLock;

pub const CLEAR: [u8; 4] = [0, 0, 0, 0];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pixmap {
    pub width: u8,
    pub height: u8,
    pub left: u8,
    pub top: u8,
    pub pixels: Vec<[u8; 4]>,
}

impl Pixmap {
    pub fn blank(width: u8, height: u8, left: u8, top: u8) -> Self {
        Self {
            width,
            height,
            left,
            top,
            pixels: vec![CLEAR; usize::from(width) * usize::from(height)],
        }
    }

    pub fn at(&self, x: u8, y: u8) -> [u8; 4] {
        self.pixels[usize::from(y) * usize::from(self.width) + usize::from(x)]
    }

    pub fn put(&mut self, x: u8, y: u8, rgb: [u8; 3]) {
        self.paint(x, y, [rgb[0], rgb[1], rgb[2], 255]);
    }

    pub fn paint(&mut self, x: u8, y: u8, rgba: [u8; 4]) {
        if x < self.width && y < self.height {
            let width = usize::from(self.width);
            self.pixels[usize::from(y) * width + usize::from(x)] = rgba;
        }
    }

    pub fn filled(&self, x: u8, y: u8) -> bool {
        x < self.width && y < self.height && self.at(x, y)[3] > 0
    }

    pub fn opaque_count(&self) -> usize {
        self.pixels.iter().filter(|pixel| pixel[3] > 0).count()
    }

    fn spans(&mut self, rows: &[(u8, u8, u8)], rgb: [u8; 3]) {
        for &(row, first, last) in rows {
            for x in first..=last {
                self.put(x, row, rgb);
            }
        }
    }

    fn cells(&self) -> Vec<(u8, u8)> {
        let mut cells = Vec::new();

        for y in 0..self.height {
            for x in 0..self.width {
                if self.filled(x, y) {
                    cells.push((x, y));
                }
            }
        }

        cells
    }
}

pub const FLAT: [(u8, u8, u8); 6] = [
    (1, 2, 5),
    (2, 1, 6),
    (3, 1, 6),
    (4, 1, 6),
    (5, 1, 6),
    (6, 2, 5),
];

const LIFTED: [(u8, u8, u8); 7] = [
    (1, 2, 5),
    (2, 1, 6),
    (3, 1, 6),
    (4, 1, 6),
    (5, 1, 6),
    (6, 1, 6),
    (7, 2, 5),
];

const MOUND: [(u8, u8, u8); 6] = [
    (2, 2, 5),
    (3, 1, 6),
    (4, 1, 6),
    (5, 1, 6),
    (6, 1, 6),
    (7, 2, 5),
];

const PEAK: [(u8, u8, u8); 8] = [
    (0, 2, 5),
    (1, 1, 6),
    (2, 1, 6),
    (3, 1, 6),
    (4, 1, 6),
    (5, 1, 6),
    (6, 1, 6),
    (7, 2, 5),
];

const TALL_PEAK: [(u8, u8, u8); 9] = [
    (0, 2, 5),
    (1, 1, 6),
    (2, 1, 6),
    (3, 1, 6),
    (4, 1, 6),
    (5, 1, 6),
    (6, 1, 6),
    (7, 1, 6),
    (8, 2, 5),
];

const OUTLINE: [(u8, u8, u8); 7] = [
    (2, 2, 7),
    (3, 1, 8),
    (4, 1, 8),
    (5, 1, 8),
    (6, 1, 8),
    (7, 1, 8),
    (8, 2, 7),
];

const RIM: [(u8, u8, u8); 9] = [
    (1, 2, 7),
    (2, 1, 8),
    (3, 0, 9),
    (4, 0, 9),
    (5, 0, 9),
    (6, 0, 9),
    (7, 0, 9),
    (8, 1, 8),
    (9, 2, 7),
];

const PIT: [(u8, u8, u8); 3] = [(1, 1, 6), (2, 1, 6), (3, 2, 5)];

const PLINTH: [(u8, u8, u8); 5] = [(3, 1, 6), (4, 1, 6), (5, 1, 6), (6, 1, 6), (7, 2, 5)];

pub fn scale(rgb: [u8; 3], by: f32) -> [u8; 3] {
    rgb.map(|channel| (f32::from(channel) * by).round().clamp(0.0, 255.0) as u8)
}

pub fn mix(from: [u8; 3], to: [u8; 3], by: f32) -> [u8; 3] {
    [0, 1, 2].map(|i| {
        let a = f32::from(from[i]);
        let b = f32::from(to[i]);
        (a + (b - a) * by).round().clamp(0.0, 255.0) as u8
    })
}

pub fn lighten(rgb: [u8; 3], by: f32) -> [u8; 3] {
    mix(rgb, [255, 255, 255], by)
}

fn grey(rgb: [u8; 3]) -> [u8; 3] {
    let level = ((u32::from(rgb[0]) * 3 + u32::from(rgb[1]) * 6 + u32::from(rgb[2])) / 10) as u8;
    [level; 3]
}

struct Dice(u32);

impl Dice {
    fn new(id: &str, variant: u8) -> Self {
        let mut hash: u32 = 0x811c_9dc5;

        for byte in id.bytes().chain([variant, 0x5a]) {
            hash ^= u32::from(byte);
            hash = hash.wrapping_mul(0x0100_0193);
        }

        Self(hash | 1)
    }

    fn next(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }

    fn below(&mut self, bound: u32) -> u32 {
        self.next() % bound.max(1)
    }

    fn chance(&mut self, percent: u32) -> bool {
        self.below(100) < percent
    }
}

fn shaded(map: &mut Pixmap, dice: &mut Dice, shades: &[([u8; 3], u32)], clump: u32) {
    let total: u32 = shades.iter().map(|(_, weight)| weight).sum();
    let cells = map.cells();
    let mut previous = shades[0].0;

    for (x, y) in cells {
        let pick = if x > 0 && map.filled(x - 1, y) && dice.chance(clump) {
            previous
        } else {
            let mut roll = dice.below(total);
            let mut chosen = shades[0].0;

            for &(rgb, weight) in shades {
                if roll < weight {
                    chosen = rgb;
                    break;
                }
                roll -= weight;
            }

            chosen
        };

        map.put(x, y, pick);
        previous = pick;
    }
}

fn lip(map: &mut Pixmap, row: u8, rgb: [u8; 3]) {
    for x in 0..map.width {
        if map.filled(x, row) {
            map.put(x, row, rgb);
        }
    }
}

fn sprinkle(map: &mut Pixmap, dice: &mut Dice, rgb: [u8; 3], count: u32, rows: (u8, u8)) {
    let mut placed = 0;
    let mut tries = 0;

    while placed < count && tries < 64 {
        tries += 1;
        let x = 1 + dice.below(6) as u8;
        let y = rows.0 + dice.below(u32::from(rows.1 - rows.0) + 1) as u8;

        if map.filled(x, y) {
            map.put(x, y, rgb);
            placed += 1;
        }
    }
}

fn deeper(paint: Paint) -> [u8; 3] {
    match paint {
        Paint::Ground(Ground::ShallowWaters) => Ground::CloseOcean.color(),
        _ => Ground::DeepOcean.color(),
    }
}

fn grass_of(paint: Paint) -> [u8; 3] {
    let high = matches!(paint, Paint::Overlay(_, true));
    Overlay::Grass
        .color(high)
        .unwrap_or_else(|| Ground::SoilLow.color())
}

fn flat(
    look: &Look,
    variant: u8,
    shades: &[([u8; 3], u32)],
    clump: u32,
    edge: Option<[u8; 3]>,
) -> Pixmap {
    let mut dice = Dice::new(look.id, variant);
    let mut map = Pixmap::blank(8, 8, 2, 2);
    map.spans(&FLAT, shades[0].0);
    shaded(&mut map, &mut dice, shades, clump);

    if let Some(edge) = edge {
        lip(&mut map, 6, edge);
    }

    map
}

fn water(look: &Look, variant: u8) -> Pixmap {
    let base = look.color();
    let mut dice = Dice::new(look.id, variant);
    let mut map = Pixmap::blank(8, 8, 2, 2);
    map.spans(&FLAT, base);

    if look.paint == Paint::Ground(Ground::ShallowWaters) {
        shaded(
            &mut map,
            &mut dice,
            &[
                (base, 6),
                (lighten(base, 0.09), 4),
                (lighten(base, 0.06), 1),
            ],
            30,
        );
    } else {
        sprinkle(&mut map, &mut dice, lighten(base, 0.04), 1, (1, 5));
        sprinkle(&mut map, &mut dice, lighten(base, 0.08), 1, (2, 4));
    }

    lip(&mut map, 6, mix(base, deeper(look.paint), 0.5));
    map
}

fn lava(look: &Look, variant: u8, glow: [u8; 3]) -> Pixmap {
    let base = look.color();
    let mut dice = Dice::new(look.id, variant);
    let mut map = Pixmap::blank(8, 8, 2, 2);
    map.spans(&FLAT, base);
    sprinkle(&mut map, &mut dice, glow, 5, (1, 5));
    sprinkle(&mut map, &mut dice, mix(base, glow, 0.5), 2, (1, 5));
    lip(&mut map, 6, mix(base, lighten(glow, 0.35), 0.7));
    map
}

fn hills(look: &Look, variant: u8) -> Pixmap {
    let base = look.color();
    let mut dice = Dice::new(look.id, variant);
    let mut map = Pixmap::blank(8, 8, 2, 3);
    map.spans(if variant == 0 { &LIFTED } else { &MOUND }, base);
    shaded(
        &mut map,
        &mut dice,
        &[(base, 5), (scale(base, 1.07), 3), (scale(base, 0.92), 3)],
        35,
    );

    if variant == 2 {
        sprinkle(
            &mut map,
            &mut dice,
            mix(base, grass_of(look.paint), 0.3),
            6,
            (2, 5),
        );
    }

    for x in 1..7 {
        if map.filled(x, 6) && dice.chance(60) {
            map.put(x, 6, scale(base, 0.8));
        }
    }

    for x in 0..8 {
        if map.filled(x, 7) {
            map.put(x, 7, scale(base, if dice.chance(30) { 0.64 } else { 0.76 }));
        }
    }

    map
}

fn mountain(look: &Look, variant: u8) -> Pixmap {
    let base = look.color();
    let mut dice = Dice::new(look.id, variant);
    let (mut map, rows): (Pixmap, &[(u8, u8, u8)]) = match variant {
        1 => (Pixmap::blank(8, 9, 2, 4), &TALL_PEAK),
        2 => (Pixmap::blank(8, 8, 2, 3), &LIFTED),
        _ => (Pixmap::blank(8, 8, 2, 3), &PEAK),
    };
    map.spans(rows, base);
    let bottom = map.height - 1;

    for (x, y) in map.cells() {
        let face = bottom - y;
        let lit = if x < 4 { 4 } else { 1 };
        let rgb = match face {
            0 => scale(base, if dice.chance(35) { 0.62 } else { 0.74 }),
            1 | 2 => scale(base, if dice.chance(40) { 0.86 } else { 0.76 }),
            _ if dice.below(8) < lit => scale(base, 1.1),
            _ if dice.chance(25) => scale(base, 0.94),
            _ => base,
        };
        map.put(x, y, rgb);
    }

    map
}

fn summit(look: &Look, variant: u8) -> Pixmap {
    let base = look.color();
    let mut dice = Dice::new(look.id, variant);
    let mut map = if variant == 1 || variant == 4 {
        Pixmap::blank(8, 9, 2, 4)
    } else {
        Pixmap::blank(8, 8, 2, 3)
    };
    let top = map.top;
    let bottom = map.height - 1;
    let apex_x = 2 + dice.below(4) as u8;
    let apex_y = match variant {
        3 | 4 => top - 1,
        2 => 1,
        _ => dice.below(2) as u8,
    };

    for y in apex_y..top {
        let reach = y - apex_y;
        let first = apex_x.saturating_sub(reach).max(1);
        let last = (apex_x + reach / 2 + 1).min(6);
        for x in first..=last {
            map.put(x, y, base);
        }
    }
    map.spans(
        &[
            (top, 2, 6),
            (top + 1, 1, 6),
            (top + 2, 1, 6),
            (top + 3, 1, 6),
            (bottom, 2, 5),
        ],
        base,
    );

    for (x, y) in map.cells() {
        let rgb = if y == bottom {
            scale(base, if dice.chance(45) { 0.84 } else { 0.66 })
        } else if x < apex_x {
            scale(base, if dice.chance(75) { 1.24 } else { 1.0 })
        } else if x == apex_x {
            if dice.chance(50) {
                base
            } else {
                scale(base, 1.24)
            }
        } else if dice.chance(25) {
            scale(base, 0.66)
        } else if dice.chance(70) {
            scale(base, 0.84)
        } else {
            base
        };
        map.put(x, y, rgb);
    }

    map
}

fn rocks(look: &Look, variant: u8) -> Pixmap {
    let base = look.color();
    let moss = mix(grey(base), grass_of(look.paint), 0.62);
    let raised = match look.paint {
        Paint::Overlay(_, true) => variant == 1 || variant == 3,
        _ => variant == 2 || variant == 4,
    };
    let mut dice = Dice::new(look.id, variant);
    let mut map = if raised {
        Pixmap::blank(8, 8, 2, 3)
    } else {
        Pixmap::blank(8, 8, 2, 2)
    };
    map.spans(if raised { &PEAK } else { &FLAT }, base);
    shaded(
        &mut map,
        &mut dice,
        &[
            (base, 4),
            (moss, if raised { 1 } else { 5 }),
            (scale(base, 1.05), 2),
            (scale(base, 0.9), 2),
        ],
        45,
    );
    let last = map.height - if raised { 1 } else { 2 };
    lip(&mut map, last, scale(base, 0.82));

    if raised {
        lip(&mut map, last - 1, scale(base, 0.9));
    }

    map
}

fn raised(look: &Look, variant: u8) -> Pixmap {
    let base = look.color();
    let mut dice = Dice::new(look.id, variant);
    let mut map = Pixmap::blank(8, 8, 2, 3);
    let rows: &[(u8, u8, u8)] = if variant == 0 { &LIFTED } else { &MOUND };
    map.spans(rows, base);
    shaded(
        &mut map,
        &mut dice,
        &[(base, 5), (scale(base, 0.95), 3), (scale(base, 1.04), 2)],
        30,
    );
    lip(&mut map, 6, scale(base, 0.85));
    lip(&mut map, 7, scale(base, 0.8));
    map
}

fn void(look: &Look, variant: u8) -> Pixmap {
    let base = look.color();
    let mut map = Pixmap::blank(8, 8, 2, 2);
    map.spans(&FLAT, base);

    if variant == 0 {
        for x in 1..7 {
            if map.filled(x, 4) {
                map.put(x, 4, scale(base, 0.92));
            }
        }
        for y in 1..7 {
            if map.filled(5, y) {
                map.put(5, y, scale(base, 0.92));
            }
        }
        map.put(5, 4, scale(base, 0.82));
    }

    map
}

fn single(look: &Look, size: u8, offset: u8, rows: &[(u8, u8, u8)]) -> Pixmap {
    let mut map = Pixmap::blank(size, size, offset, offset);
    map.spans(rows, look.color());
    map
}

pub fn draw(look: &Look, variant: u8) -> Pixmap {
    let base = look.color();

    match look.style {
        Style::Water => water(look, variant),
        Style::Sand => flat(
            look,
            variant,
            &[(base, 7), (scale(mix(base, grey(base), 0.25), 0.99), 3)],
            20,
            Some(scale(base, 0.88)),
        ),
        Style::Soil => flat(
            look,
            variant,
            &[(base, 6), (scale(base, 0.95), 3), (lighten(base, 0.05), 1)],
            25,
            Some(scale(base, 0.87)),
        ),
        Style::Grass => flat(
            look,
            variant,
            &[(base, 4), (scale(base, 1.04), 3), (scale(base, 0.96), 3)],
            15,
            Some(scale(base, 0.89)),
        ),
        Style::Speckle(accent) => {
            let mut map = flat(
                look,
                variant,
                &[(base, 5), (scale(base, 1.03), 3), (scale(base, 0.96), 2)],
                20,
                Some(scale(base, 0.89)),
            );
            let mut dice = Dice::new(look.id, variant.wrapping_add(17));
            let count = 1 + dice.below(3);
            sprinkle(&mut map, &mut dice, accent, count, (1, 5));
            map
        }
        Style::Lava(glow) => lava(look, variant, glow),
        Style::Hills => hills(look, variant),
        Style::Mountains => mountain(look, variant),
        Style::Summit => summit(look, variant),
        Style::Rocks => rocks(look, variant),
        Style::Raised => raised(look, variant),
        Style::Void => void(look, variant),
        Style::Plinth => {
            let mut map = Pixmap::blank(8, 8, 2, 3);
            map.spans(&PLINTH, base);
            map
        }
        Style::Outline => single(look, 10, 3, &OUTLINE),
        Style::Rim => single(look, 10, 3, &RIM),
        Style::Pit => single(look, 8, 2, &PIT),
    }
}

pub fn sprites(look: &Look) -> &'static [Pixmap] {
    static SHEET: OnceLock<HashMap<&'static str, Vec<Pixmap>>> = OnceLock::new();

    let sheet = SHEET.get_or_init(|| {
        look::all()
            .into_iter()
            .map(|look| (look.id, (0..look.variants).map(|v| draw(look, v)).collect()))
            .collect()
    });

    sheet.get(look.id).map(Vec::as_slice).unwrap_or(&[])
}

pub const BERRY: [u8; 3] = [0xd2, 0x38, 0x3e];
const BARK: [u8; 3] = [0x6b, 0x48, 0x2c];
const STONE: [u8; 3] = [0x8f, 0x92, 0x8c];

pub fn wall_block(look: &Look) -> Pixmap {
    let body = look.color();
    let edge = look.edge.unwrap_or_else(|| scale(body, 0.7));
    let cap = lighten(body, 0.35);
    let side = scale(edge, 0.62);
    let shadow = [0, 0, 0, 118];
    let mut map = Pixmap::blank(8, 8, 2, 4);
    map.spans(&[(0, 3, 4)], lighten(body, 0.18));
    map.spans(&[(1, 2, 4), (2, 2, 4), (3, 2, 4), (4, 2, 4)], body);
    map.spans(&[(1, 3, 4), (2, 3, 3)], cap);
    map.spans(&[(5, 2, 4), (6, 2, 4), (7, 2, 4)], edge);
    map.put(3, 6, body);
    map.spans(
        &[
            (1, 5, 5),
            (2, 5, 5),
            (3, 5, 5),
            (4, 5, 5),
            (5, 5, 5),
            (6, 5, 5),
            (7, 5, 5),
        ],
        side,
    );

    for (x, y) in [(6, 4), (6, 5), (7, 5), (6, 6), (7, 6), (6, 7), (7, 7)] {
        map.paint(x, y, shadow);
    }

    map
}

pub fn flora(kind: Flora, cover: [u8; 3], variant: u8) -> Pixmap {
    let leaf = kind.tint(cover);
    let light = lighten(scale(leaf, 1.18), 0.06);
    let dark = scale(leaf, 0.78);
    let mut dice = Dice::new(kind.name(), variant);

    match kind {
        Flora::Tree => {
            let mut map = Pixmap::blank(8, 9, 2, 5);
            let crown: [(u8, u8, u8); 6] = match variant {
                0 => [
                    (0, 3, 4),
                    (1, 2, 5),
                    (2, 1, 6),
                    (3, 1, 6),
                    (4, 1, 6),
                    (5, 2, 5),
                ],
                1 => [
                    (0, 2, 4),
                    (1, 1, 5),
                    (2, 1, 6),
                    (3, 1, 6),
                    (4, 2, 6),
                    (5, 2, 5),
                ],
                _ => [
                    (0, 3, 5),
                    (1, 2, 6),
                    (2, 1, 6),
                    (3, 1, 6),
                    (4, 1, 5),
                    (5, 2, 5),
                ],
            };
            map.spans(&crown, leaf);

            for (x, y) in map.cells() {
                if y <= 2 && x <= 3 && dice.chance(65) {
                    map.put(x, y, light);
                } else if (y >= 4 || x >= 5) && dice.chance(55) {
                    map.put(x, y, dark);
                }
            }

            map.spans(&[(6, 3, 3), (7, 3, 3), (8, 3, 3)], BARK);
            map.spans(&[(6, 4, 4), (7, 4, 4), (8, 4, 4)], scale(BARK, 0.74));
            map
        }
        Flora::Bush => {
            let mut map = Pixmap::blank(8, 6, 2, 2);
            map.spans(
                &[
                    (0, 3, 4),
                    (1, 2, 5),
                    (2, 1, 6),
                    (3, 1, 6),
                    (4, 1, 6),
                    (5, 2, 5),
                ],
                leaf,
            );

            for (x, y) in map.cells() {
                if y <= 1 && x <= 3 {
                    map.put(x, y, light);
                } else if y >= 4 {
                    map.put(x, y, dark);
                }
            }

            sprinkle(&mut map, &mut dice, BERRY, 3, (1, 4));
            map
        }
        Flora::Sapling => {
            let mut map = Pixmap::blank(6, 5, 1, 1);
            map.spans(&[(0, 2, 3), (1, 1, 2), (1, 3, 4)], light);
            map.spans(&[(2, 2, 3)], leaf);
            map.spans(&[(3, 2, 2), (4, 2, 2)], scale(BARK, 1.1));
            map
        }
    }
}

pub fn deposit(ore: Ore, variant: u8) -> Pixmap {
    let mut dice = Dice::new(ore.name(), variant);
    let mut map = Pixmap::blank(8, 6, 2, 2);
    let rows: [(u8, u8, u8); 5] = match variant {
        0 => [(1, 3, 4), (2, 2, 5), (3, 1, 6), (4, 1, 6), (5, 2, 5)],
        1 => [(1, 2, 3), (2, 1, 5), (3, 1, 6), (4, 1, 6), (5, 2, 6)],
        _ => [(1, 4, 5), (2, 2, 6), (3, 1, 6), (4, 1, 6), (5, 1, 5)],
    };
    map.spans(&rows, STONE);

    for (x, y) in map.cells() {
        if y <= 2 && x <= 3 {
            map.put(x, y, lighten(STONE, 0.2));
        } else if y == 5 || x >= 6 {
            map.put(x, y, scale(STONE, 0.7));
        }
    }

    if ore != Ore::Stone {
        let vein = ore.color();
        sprinkle(&mut map, &mut dice, vein, 4, (2, 4));
        sprinkle(&mut map, &mut dice, lighten(vein, 0.35), 1, (2, 3));
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_type_has_the_worldbox_number_of_variants() {
        for look in look::all() {
            assert_eq!(
                sprites(look).len(),
                usize::from(look.variants),
                "{}",
                look.id
            );
        }

        assert_eq!(sprites(&look::SAND).len(), 3);
        assert_eq!(sprites(&look::SUMMIT).len(), 6);
        assert_eq!(
            sprites(
                look::biome_pair(Overlay::Mushroom)
                    .unwrap()
                    .first()
                    .unwrap()
            )
            .len(),
            6
        );
        assert_eq!(sprites(&look::CLOSE_OCEAN).len(), 1);
    }

    #[test]
    fn a_flat_tile_covers_its_cell_and_one_pixel_around_it_with_cut_corners() {
        let sprite = &sprites(&look::SOIL_LOW)[0];

        assert_eq!(
            (sprite.width, sprite.height, sprite.left, sprite.top),
            (8, 8, 2, 2)
        );
        assert_eq!(sprite.opaque_count(), 32);

        for y in 2..6 {
            for x in 2..6 {
                assert!(sprite.filled(x, y), "{x},{y}");
            }
        }

        for (x, y) in [
            (1, 1),
            (6, 1),
            (1, 6),
            (6, 6),
            (0, 3),
            (7, 3),
            (3, 0),
            (3, 7),
        ] {
            assert!(!sprite.filled(x, y), "{x},{y}");
        }
    }

    #[test]
    fn the_lower_lip_of_a_flat_tile_is_darker_than_its_body() {
        for look in [
            &look::SAND,
            &look::SOIL_HIGH,
            &look::CLOSE_OCEAN,
            &look::SHALLOW_WATERS,
        ] {
            let sprite = &sprites(look)[0];
            let lip = sprite.at(3, 6);
            let body = look.color();
            let sum = |rgb: [u8; 4]| u32::from(rgb[0]) + u32::from(rgb[1]) + u32::from(rgb[2]);

            assert!(
                sum(lip) < sum([body[0], body[1], body[2], 255]),
                "{}",
                look.id
            );
        }
    }

    #[test]
    fn rocks_stand_up_one_pixel_and_the_tall_peaks_two() {
        assert!(sprites(&look::HILLS).iter().all(|s| s.top == 3));
        assert_eq!(
            sprites(&look::MOUNTAINS)
                .iter()
                .map(|s| (s.height, s.top))
                .collect::<Vec<_>>(),
            [(8, 3), (9, 4), (8, 3)]
        );
        assert_eq!(
            sprites(&look::SUMMIT)
                .iter()
                .filter(|s| s.height == 9)
                .count(),
            2
        );
        assert!(!sprites(&look::MOUNTAINS)[2].filled(3, 0));
        assert!(sprites(&look::MOUNTAINS)[0].filled(3, 0));
    }

    #[test]
    fn the_coast_outline_and_rim_reach_past_the_land() {
        let outline = &sprites(&look::BORDER_WATER)[0];
        let rim = &sprites(&look::BORDER_WATER_RUNUP)[0];

        assert_eq!((outline.width, outline.left, outline.top), (10, 3, 3));
        assert!(outline.filled(1, 5) && outline.filled(8, 5) && outline.filled(4, 8));
        assert!(!outline.filled(0, 5) && !outline.filled(4, 1));
        assert!(rim.filled(0, 5) && rim.filled(9, 5) && rim.filled(4, 9) && rim.filled(4, 1));
    }

    #[test]
    fn every_rock_tile_covers_its_whole_cell() {
        for look in [&look::HILLS, &look::MOUNTAINS, &look::SUMMIT] {
            for sprite in sprites(look) {
                for y in sprite.top..sprite.top + 4 {
                    for x in 2..6 {
                        assert!(sprite.filled(x, y), "{} {x},{y}", look.id);
                    }
                }
            }
        }
    }

    #[test]
    fn the_art_never_changes_between_calls() {
        assert_eq!(draw(&look::SUMMIT, 4), draw(&look::SUMMIT, 4));
        assert_ne!(draw(&look::SAND, 0), draw(&look::SAND, 1));
    }

    #[test]
    fn objects_stand_on_the_bottom_of_their_cell() {
        for sprite in [
            flora(Flora::Tree, Overlay::Grass.color(false).unwrap(), 0),
            flora(Flora::Bush, Overlay::Grass.color(false).unwrap(), 0),
            flora(Flora::Sapling, Overlay::Grass.color(false).unwrap(), 0),
            deposit(Ore::Gold, 1),
        ] {
            assert_eq!(i32::from(sprite.height) - i32::from(sprite.top), 4);
        }

        assert_eq!(wall_block(&look::WALL_IRON).top, 4);
    }
}
