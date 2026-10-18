use crate::{CLEAR, shade};

pub type Rgba = [u8; 4];

#[derive(Debug, Clone, PartialEq)]
pub struct Picture {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Rgba>,
}

impl Picture {
    pub fn blank(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![CLEAR; (width * height) as usize],
        }
    }

    pub fn set(&mut self, x: u32, y: u32, color: Rgba) {
        if x < self.width && y < self.height {
            self.pixels[(y * self.width + x) as usize] = color;
        }
    }

    pub fn get(&self, x: u32, y: u32) -> Rgba {
        self.pixels[(y * self.width + x) as usize]
    }

    pub fn bytes(&self) -> Vec<u8> {
        self.pixels.iter().flatten().copied().collect()
    }
}

const fn rgb(hex: u32) -> Rgba {
    [(hex >> 16) as u8, (hex >> 8) as u8, hex as u8, 255]
}

const fn alpha(hex: u32, a: u8) -> Rgba {
    [(hex >> 16) as u8, (hex >> 8) as u8, hex as u8, a]
}

fn tone(color: [u8; 3], by: f32) -> Rgba {
    let [r, g, b] = shade(color, by);
    [r, g, b, 255]
}

pub const BAR_FACE: [u8; 3] = [0x4b, 0x53, 0x45];
pub const BAR_HEIGHT: u32 = 120;
pub const BAR_WIDTH: u32 = 64;
pub const BAR_EDGE: u32 = 16;
pub const BAR_RIM: [Rgba; 9] = [
    rgb(0x2b3029),
    rgb(0x8a9480),
    rgb(0x6c7662),
    rgb(0x6c7662),
    rgb(0x596350),
    rgb(0x363c32),
    rgb(0x41483c),
    rgb(0x464d40),
    rgb(0x4b5345),
];

pub fn bar() -> Picture {
    let mut out = Picture::blank(BAR_WIDTH, BAR_HEIGHT);
    let face = tone(BAR_FACE, 1.0);
    let groove = tone(BAR_FACE, 0.78);
    let groove_light = tone(BAR_FACE, 1.18);
    let stud = tone(BAR_FACE, 1.32);
    let stud_dark = tone(BAR_FACE, 0.62);

    for y in 0..BAR_HEIGHT {
        for x in 0..BAR_WIDTH {
            let color = if (y as usize) < BAR_RIM.len() {
                BAR_RIM[y as usize]
            } else {
                face
            };
            out.set(x, y, color);
        }
    }

    for side in [0, BAR_WIDTH - BAR_EDGE] {
        let near = if side == 0 { 3 } else { side + 10 };
        let far = near + 2;

        for y in BAR_RIM.len() as u32 + 3..BAR_HEIGHT {
            out.set(near, y, groove);
            out.set(near + 1, y, groove);
            out.set(far, y, groove_light);
        }

        for y in (BAR_RIM.len() as u32 + 7..BAR_HEIGHT - 4).step_by(10) {
            let cx = near + 1;
            out.set(cx, y - 1, stud);
            out.set(cx - 1, y, stud);
            out.set(cx, y, stud);
            out.set(cx + 1, y, stud_dark);
            out.set(cx, y + 1, stud_dark);
        }
    }

    out
}

pub const TAB_WIDTH: u32 = 43;
pub const TAB_HEIGHT: u32 = 18;

pub fn tab(selected: bool) -> Picture {
    let face = if selected {
        [0xc4, 0x3e, 0x31]
    } else {
        [0x5a, 0x64, 0x50]
    };
    let edge = tone(face, 0.48);
    let top = tone(face, 1.42);
    let light = tone(face, 1.2);
    let fill = tone(face, 1.0);
    let corner = tone(face, 1.7);
    let mut out = Picture::blank(TAB_WIDTH, TAB_HEIGHT);

    for y in 0..TAB_HEIGHT {
        let inset = (TAB_HEIGHT - 1 - y.min(TAB_HEIGHT - 1)) / 2;
        let inset = inset.min(6);
        let left = inset;
        let right = TAB_WIDTH - 1 - inset;

        for x in left..=right {
            let rounded = y == 0 && (x == left || x == right);
            if rounded {
                continue;
            }

            let color = if x == left || x == right || y == 0 {
                edge
            } else if y == 1 {
                top
            } else if x == left + 1 || y == 2 {
                light
            } else {
                fill
            };
            out.set(x, y, color);
        }
    }

    let inset = (TAB_HEIGHT - 1 - 2) / 2;
    out.set(inset.min(6) + 3, 3, corner);
    out.set(TAB_WIDTH - 1 - inset.min(6) - 3, 3, corner);
    out
}

pub const BUTTON: u32 = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Well {
    Plain,
    Studded,
    Present,
}

const RIM: [Rgba; 5] = [
    rgb(0x6a5a3c),
    rgb(0x8c7a52),
    rgb(0xb09a68),
    rgb(0x8c7a52),
    rgb(0x6a5a3c),
];

fn clipped(x: u32, y: u32, size_x: u32, size_y: u32) -> bool {
    let near = |v: u32, size: u32| v.min(size - 1 - v);
    let (dx, dy) = (near(x, size_x), near(y, size_y));
    dx + dy < 2
}

pub fn button(well: Well) -> Picture {
    button_sized(BUTTON, BUTTON, well)
}

pub fn button_sized(width: u32, height: u32, well: Well) -> Picture {
    let mut out = Picture::blank(width, height);
    let outline = rgb(0x1b1f1b);
    let inner = rgb(0x151816);
    let shadow = rgb(0x1f2421);
    let fill_a = rgb(0x2c3230);
    let fill_b = rgb(0x262b29);
    let hatch = rgb(0x323836);

    for y in 0..height {
        for x in 0..width {
            if clipped(x, y, width, height) {
                continue;
            }

            let ring = x.min(y).min(width - 1 - x).min(height - 1 - y);
            let color = match ring {
                0 => outline,
                1 => {
                    let along = if x == 1 || x == width - 2 {
                        (y * (RIM.len() as u32 - 1) + height / 2) / height
                    } else {
                        if y == 1 { 1 } else { 0 }
                    };
                    RIM[along.min(RIM.len() as u32 - 1) as usize]
                }
                2 => inner,
                3 if x == 3 || y == 3 => shadow,
                _ => {
                    if (x + y) % 6 == 0 {
                        hatch
                    } else if x + y < width {
                        fill_a
                    } else {
                        fill_b
                    }
                }
            };
            out.set(x, y, color);
        }
    }

    match well {
        Well::Plain => {}
        Well::Studded => {
            for (cx, cy) in [
                (4, 4),
                (width - 5, 4),
                (4, height - 5),
                (width - 5, height - 5),
            ] {
                out.set(cx, cy, rgb(0x6fe0d0));
                out.set(cx + 1, cy, rgb(0x2e9a8c));
                out.set(cx, cy + 1, rgb(0x2e9a8c));
                out.set(cx + 1, cy + 1, rgb(0x1d5f57));
            }
        }
        Well::Present => {
            for step in 0..6 {
                for x in (width - 3 - step)..(width - 2) {
                    let y = height - 9 + step;
                    if !clipped(x, y, width, height) {
                        out.set(
                            x,
                            y,
                            if step == 0 {
                                rgb(0x9cf07a)
                            } else {
                                rgb(0x4cb83c)
                            },
                        );
                    }
                }
            }
        }
    }

    out
}

pub fn selection() -> Picture {
    let mut out = Picture::blank(BUTTON, BUTTON);
    let frame = rgb(0xf0c040);
    let glint = rgb(0xffe08a);
    let wash = alpha(0xffe9a0, 38);

    for y in 0..BUTTON {
        for x in 0..BUTTON {
            if clipped(x, y, BUTTON, BUTTON) {
                continue;
            }

            let ring = x.min(y).min(BUTTON - 1 - x).min(BUTTON - 1 - y);
            let color = match ring {
                0 | 1 => frame,
                2 => glint,
                _ => wash,
            };
            out.set(x, y, color);
        }
    }

    out
}

pub fn groove(thick: bool) -> Picture {
    let columns: &[Rgba] = if thick {
        &[
            alpha(0x000000, 46),
            alpha(0x1f231d, 220),
            alpha(0x1f231d, 220),
            alpha(0x9aa48c, 46),
            alpha(0x000000, 0),
        ]
    } else {
        &[alpha(0x22261f, 228), alpha(0x8c967e, 64)]
    };
    let mut out = Picture::blank(columns.len() as u32, 3);

    for y in 0..3 {
        for (x, color) in columns.iter().enumerate() {
            out.set(x as u32, y, *color);
        }
    }

    out
}

pub const ARROW_WIDTH: u32 = 12;
pub const ARROW_HEIGHT: u32 = 120;

pub fn arrow() -> Picture {
    let mut out = Picture::blank(ARROW_WIDTH, ARROW_HEIGHT);
    let face = tone(BAR_FACE, 0.92);
    let edge = tone(BAR_FACE, 1.24);
    let dark = tone(BAR_FACE, 0.6);
    let gold = rgb(0xf6c443);
    let gold_dark = rgb(0xb07a1c);

    for y in 0..ARROW_HEIGHT {
        for x in 0..ARROW_WIDTH {
            let color = if x == 0 {
                edge
            } else if x == ARROW_WIDTH - 1 {
                dark
            } else {
                face
            };
            out.set(x, y, color);
        }
    }

    let centre = 44;
    for step in 0..6u32 {
        let x = 3 + step;
        for y in [centre - 6 + step, centre + 6 - step] {
            out.set(x, y, gold);
            out.set(x + 1, y, gold_dark);
        }
    }

    out
}

pub fn tooltip_body() -> Picture {
    rounded_panel(20, 20, alpha(0x0c0c0e, 236), alpha(0x1e1f22, 222), true)
}

pub fn tooltip_head() -> Picture {
    rounded_panel(20, 20, alpha(0x0c0c0e, 214), alpha(0x101114, 214), false)
}

fn rounded_panel(width: u32, height: u32, border: Rgba, fill: Rgba, round_bottom: bool) -> Picture {
    let mut out = Picture::blank(width, height);
    let corner = [3u32, 1, 1];

    for y in 0..height {
        for x in 0..width {
            let from_top = y as usize;
            let from_bottom = (height - 1 - y) as usize;
            let cut = if from_top < corner.len() {
                corner[from_top]
            } else if round_bottom && from_bottom < corner.len() {
                corner[from_bottom]
            } else {
                0
            };

            if x < cut || x >= width - cut {
                continue;
            }

            let on_edge =
                x == cut || x == width - 1 - cut || y == 0 || (round_bottom && y == height - 1);
            out.set(x, y, if on_edge { border } else { fill });
        }
    }

    out
}

pub fn window_frame() -> Picture {
    let width = 64;
    let height = 64;
    let mut out = Picture::blank(width, height);
    let outer = rgb(0x22261f);
    let bevel = rgb(0x7e8872);
    let face = tone(BAR_FACE, 1.0);
    let inner_edge = rgb(0x30362c);
    let inner = rgb(0x363c32);

    for y in 0..height {
        for x in 0..width {
            if clipped(x, y, width, height) {
                continue;
            }

            let ring = x.min(y).min(width - 1 - x).min(height - 1 - y);
            let color = match ring {
                0 => outer,
                1 => {
                    if x == 1 || y == 1 {
                        bevel
                    } else {
                        tone(BAR_FACE, 0.7)
                    }
                }
                2..=9 => face,
                10 => inner_edge,
                _ => inner,
            };
            out.set(x, y, color);
        }
    }

    for (cx, cy) in [
        (5, 5),
        (width - 6, 5),
        (5, height - 6),
        (width - 6, height - 6),
    ] {
        out.set(cx, cy, tone(BAR_FACE, 1.4));
        out.set(cx + 1, cy + 1, tone(BAR_FACE, 0.6));
    }

    out
}

pub fn inset_panel() -> Picture {
    let size = 16;
    let mut out = Picture::blank(size, size);

    for y in 0..size {
        for x in 0..size {
            if clipped(x, y, size, size) {
                continue;
            }

            let ring = x.min(y).min(size - 1 - x).min(size - 1 - y);
            let color = match ring {
                0 => rgb(0x1e221c),
                1 if x == 1 || y == 1 => rgb(0x262b24),
                _ => rgb(0x30362c),
            };
            out.set(x, y, color);
        }
    }

    out
}

pub fn close_tab() -> Picture {
    let width = 40;
    let height = 36;
    let mut out = Picture::blank(width, height);
    let face = tone(BAR_FACE, 1.0);

    for y in 0..height {
        for x in 0..width {
            if clipped(x, y, width, height) {
                continue;
            }

            let ring = x.min(y).min(width - 1 - x).min(height - 1 - y);
            let color = match ring {
                0 => rgb(0x22261f),
                1 => {
                    if x == 1 || y == 1 {
                        rgb(0x7e8872)
                    } else {
                        tone(BAR_FACE, 0.7)
                    }
                }
                _ => face,
            };
            out.set(x, y, color);
        }
    }

    out
}

pub fn red_plate(width: u32, height: u32) -> Picture {
    let mut out = Picture::blank(width, height);
    let face = [0xc8, 0x42, 0x34];

    for y in 0..height {
        for x in 0..width {
            if clipped(x, y, width, height) {
                continue;
            }

            let ring = x.min(y).min(width - 1 - x).min(height - 1 - y);
            let color = match ring {
                0 => tone(face, 0.4),
                1 => {
                    if x == 1 || y == 1 {
                        tone(face, 1.38)
                    } else {
                        tone(face, 0.68)
                    }
                }
                _ => {
                    if (x + 2 * y) % 9 == 0 {
                        tone(face, 1.08)
                    } else {
                        tone(face, 1.0)
                    }
                }
            };
            out.set(x, y, color);
        }
    }

    out
}

pub fn red_well(width: u32, height: u32) -> Picture {
    let mut out = red_plate(width, height);
    let well = button_sized(width - 6, height - 6, Well::Plain);

    for y in 0..well.height {
        for x in 0..well.width {
            let pixel = well.get(x, y);
            if pixel[3] > 0 && x.min(y).min(well.width - 1 - x).min(well.height - 1 - y) >= 2 {
                out.set(x + 3, y + 3, pixel);
            }
        }
    }

    out
}

pub fn gilded(width: u32, height: u32) -> Picture {
    let mut out = red_plate(width, height);
    let gold = rgb(0xf2c43a);
    let glint = rgb(0xfff09a);

    for y in 0..height {
        for x in 0..width {
            if clipped(x, y, width, height) {
                continue;
            }

            let ring = x.min(y).min(width - 1 - x).min(height - 1 - y);
            if ring == 2 {
                out.set(x, y, if x == 2 || y == 2 { glint } else { gold });
            }
        }
    }

    out
}

pub const TAB_ICON_WIDTH: u32 = 33;
pub const TAB_ICON_HEIGHT: u32 = 12;
pub const TAB_ICON_INK: Rgba = rgb(0xeceae2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabMark {
    Terrain,
    Minds,
    Creatures,
    Nature,
    Ruin,
    Misc,
}

const TERRAIN_MARK: [&str; 12] = [
    "..........#.....................#",
    ".........###...................##",
    "........#####.......#.........###",
    ".......###.###.....###.......####",
    "......###...###...#####.....#####",
    ".....###.....###.###.###...######",
    "....###.......#####...###.##.####",
    "...###.........###.....#####.####",
    "..###...........#.......###..####",
    ".###.....................#...####",
    "###..........................####",
    "#################################",
];

const MINDS_MARK: [&str; 12] = [
    ".......###..............###......",
    "......#####............#####.....",
    "......#####............#####.....",
    ".......###....#####.....###......",
    "..............#####..............",
    ".....#######...###...#######.....",
    "....#########.......#########....",
    "....#########.#####.#########....",
    "....#########.#####.#########....",
    "....#########.#####.#########....",
    "....#########.#####.#########....",
    "..............#####..............",
];

const CREATURES_MARK: [&str; 12] = [
    "...........##.......##...........",
    "..........####.....####..........",
    "...##.....####.....####.....##...",
    "..####.....##.......##.....####..",
    "..####.....................####..",
    "...##.........#####.........##...",
    ".............#######.............",
    "............#########............",
    "...........###########...........",
    "...........###########...........",
    "............#########............",
    ".............##...##.............",
];

const NATURE_MARK: [&str; 12] = [
    "........###................####..",
    "......#######............########",
    ".....#########..........#########",
    "....###########..........#######.",
    "....###########..................",
    ".....#########.......#..#..#.....",
    "......#######.......#..#..#......",
    "........###........#..#..#.......",
    "........###......................",
    "........###......................",
    "........###......................",
    "......#######....................",
];

const RUIN_MARK: [&str; 12] = [
    "....................#....#.......",
    "...................#.#..#........",
    "..........######....#..#.........",
    "........##########...##..........",
    ".......############.##...........",
    "......##############.............",
    "......##############.............",
    "......##############.............",
    ".......############..............",
    "........##########...............",
    "..........######.................",
    ".................................",
];

const MISC_MARK: [&str; 12] = [
    ".................................",
    ".................................",
    ".................................",
    ".....####......####......####....",
    "....######....######....######...",
    "....######....######....######...",
    "....######....######....######...",
    ".....####......####......####....",
    ".................................",
    ".................................",
    ".................................",
    ".................................",
];

impl TabMark {
    fn rows(self) -> &'static [&'static str; 12] {
        match self {
            Self::Terrain => &TERRAIN_MARK,
            Self::Minds => &MINDS_MARK,
            Self::Creatures => &CREATURES_MARK,
            Self::Nature => &NATURE_MARK,
            Self::Ruin => &RUIN_MARK,
            Self::Misc => &MISC_MARK,
        }
    }
}

pub fn tab_mark(mark: TabMark) -> Picture {
    let mut out = Picture::blank(TAB_ICON_WIDTH, TAB_ICON_HEIGHT);

    for (y, row) in mark.rows().iter().enumerate() {
        for (x, glyph) in row.chars().enumerate() {
            if glyph == '#' {
                out.set(x as u32, y as u32, TAB_ICON_INK);
            }
        }
    }

    out
}

const DIGITS: [[&str; 5]; 10] = [
    ["###", "#.#", "#.#", "#.#", "###"],
    [".#.", "##.", ".#.", ".#.", "###"],
    ["###", "..#", "###", "#..", "###"],
    ["###", "..#", ".##", "..#", "###"],
    ["#.#", "#.#", "###", "..#", "..#"],
    ["###", "#..", "###", "..#", "###"],
    ["###", "#..", "###", "#.#", "###"],
    ["###", "..#", "..#", ".#.", ".#."],
    ["###", "#.#", "###", "#.#", "###"],
    ["###", "#.#", "###", "..#", "###"],
];

const TIMES: [&str; 5] = ["...", "#.#", ".#.", "#.#", "..."];
const POINT: [&str; 5] = [".", ".", ".", ".", "#"];

pub fn badge_width(text: &str) -> u32 {
    text.chars()
        .map(|glyph| if glyph == '.' { 2 } else { 4 })
        .sum::<u32>()
        .saturating_sub(1)
        + 1
}

pub fn stamp_text(onto: &mut Picture, text: &str, right: u32, bottom: u32) {
    let width = badge_width(text);
    let left = right.saturating_sub(width);
    let top = bottom.saturating_sub(6);
    let ink = rgb(0xffffff);
    let shadow = rgb(0x101010);
    let mut x = left;

    for glyph in text.chars() {
        let rows: &[&str; 5] = match glyph {
            'x' => &TIMES,
            '.' => &POINT,
            digit => match digit.to_digit(10) {
                Some(value) => &DIGITS[value as usize],
                None => continue,
            },
        };
        let advance = rows[0].len() as u32 + 1;

        for (dy, row) in rows.iter().enumerate() {
            for (dx, cell) in row.chars().enumerate() {
                if cell == '#' {
                    onto.set(x + dx as u32 + 1, top + dy as u32 + 1, shadow);
                }
            }
        }

        for (dy, row) in rows.iter().enumerate() {
            for (dx, cell) in row.chars().enumerate() {
                if cell == '#' {
                    onto.set(x + dx as u32, top + dy as u32, ink);
                }
            }
        }

        x += advance;
    }
}

pub fn framed(sprite: &[[u8; 4]], side: u32, canvas: u32) -> Picture {
    let mut out = Picture::blank(canvas, canvas);
    let offset = (canvas - side) / 2;

    for y in 0..side {
        for x in 0..side {
            let pixel = sprite[(y * side + x) as usize];
            if pixel[3] > 0 {
                out.set(x + offset, y + offset, pixel);
            }
        }
    }

    out
}

pub fn mask_picture(cells: &[bool], side: u32) -> Picture {
    let mut out = Picture::blank(side, side);

    for y in 0..side {
        for x in 0..side {
            if cells[(y * side + x) as usize] {
                out.set(x, y, rgb(0xffffff));
            }
        }
    }

    out
}

pub fn darkened(picture: &Picture, by: f32, fade: f32) -> Picture {
    let mut out = picture.clone();

    for pixel in &mut out.pixels {
        if pixel[3] == 0 {
            continue;
        }

        for channel in pixel.iter_mut().take(3) {
            *channel = (*channel as f32 * by).round().clamp(0.0, 255.0) as u8;
        }
        pixel[3] = (pixel[3] as f32 * fade).round() as u8;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bar_is_lit_from_above_and_keeps_its_rim_height() {
        let bar = bar();

        assert_eq!((bar.width, bar.height), (BAR_WIDTH, BAR_HEIGHT));
        assert_eq!(bar.get(32, 1), BAR_RIM[1]);
        assert_eq!(bar.get(32, 60), tone(BAR_FACE, 1.0));
    }

    #[test]
    fn the_centre_of_the_bar_stretches_without_seams() {
        let bar = bar();

        for y in 0..BAR_HEIGHT {
            let first = bar.get(BAR_EDGE, y);
            for x in BAR_EDGE..BAR_WIDTH - BAR_EDGE {
                assert_eq!(bar.get(x, y), first, "row {y} col {x}");
            }
        }
    }

    #[test]
    fn a_tab_narrows_towards_its_top() {
        let idle = tab(false);
        let width_at = |y: u32| (0..TAB_WIDTH).filter(|x| idle.get(*x, y)[3] > 0).count();

        assert!(width_at(1) < width_at(TAB_HEIGHT - 1));
        assert_eq!(width_at(TAB_HEIGHT - 1), TAB_WIDTH as usize);
        assert_ne!(tab(true).get(20, 10), idle.get(20, 10));
    }

    #[test]
    fn a_button_has_clipped_corners_and_an_opaque_well() {
        let plain = button(Well::Plain);

        assert_eq!(plain.get(0, 0)[3], 0);
        assert_eq!(plain.get(1, 0)[3], 0);
        assert_eq!(plain.get(16, 16)[3], 255);
        assert_ne!(button(Well::Present).pixels, plain.pixels);
        assert_ne!(button(Well::Studded).pixels, plain.pixels);
    }

    #[test]
    fn the_selection_frame_is_see_through_in_the_middle() {
        let frame = selection();

        assert_eq!(frame.get(16, 0)[3], 255);
        assert!(frame.get(16, 16)[3] < 60);
    }

    #[test]
    fn grooves_keep_the_separator_widths() {
        assert_eq!(groove(true).width, 5);
        assert_eq!(groove(false).width, 2);
    }

    #[test]
    fn every_tab_mark_is_drawn_on_its_grid() {
        for mark in [
            TabMark::Terrain,
            TabMark::Minds,
            TabMark::Creatures,
            TabMark::Nature,
            TabMark::Ruin,
            TabMark::Misc,
        ] {
            for (y, row) in mark.rows().iter().enumerate() {
                assert_eq!(row.chars().count(), TAB_ICON_WIDTH as usize, "{mark:?} {y}");
            }
            assert!(tab_mark(mark).pixels.iter().any(|px| px[3] > 0));
        }
    }

    #[test]
    fn badge_text_fits_inside_a_button_icon() {
        let mut canvas = Picture::blank(28, 28);
        stamp_text(&mut canvas, "x0.5", 27, 27);

        assert!(badge_width("x0.5") < 28);
        assert!(canvas.pixels.iter().any(|px| *px == rgb(0xffffff)));
    }

    #[test]
    fn darkening_keeps_transparent_pixels_transparent() {
        let mut picture = Picture::blank(2, 1);
        picture.set(0, 0, rgb(0xc8c8c8));
        let dim = darkened(&picture, 0.5, 1.0);

        assert_eq!(dim.get(0, 0), rgb(0x646464));
        assert_eq!(dim.get(1, 0)[3], 0);
    }
}
