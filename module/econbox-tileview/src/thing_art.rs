use crate::art::{Pixmap, lighten, mix, scale};
use crate::things::{Kind, Variant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    Round,
    Conifer,
    Palm,
    Cactus,
    Umbrella,
    Willow,
    Cap,
    Shard,
    Dead,
    Thorn,
    Glow,
    Toadstool,
    Reed,
    Blossom,
    Sprig,
    Bush,
    Rock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub leaf: [u8; 3],
    pub bark: [u8; 3],
    pub accent: Option<[u8; 3]>,
}

const fn rgb(c: u32) -> [u8; 3] {
    [(c >> 16) as u8, (c >> 8) as u8, c as u8]
}

const fn pal(leaf: u32, bark: u32, accent: u32) -> Palette {
    Palette {
        leaf: rgb(leaf),
        bark: rgb(bark),
        accent: if accent == 0 { None } else { Some(rgb(accent)) },
    }
}

pub fn palette(id: &str) -> Palette {
    match id {
        "tree_green_1" => pal(0x4f8c36, 0x6a462a, 0),
        "tree_green_2" => pal(0x3f7a30, 0x5e3e26, 0),
        "tree_green_3" => pal(0x69a03c, 0x70492b, 0),
        "birch_tree" => pal(0x86bf3e, 0xe4e0d2, 0x3a3634),
        "maple_tree" => pal(0xc9622a, 0x5c3a22, 0xe8a03a),
        "lemon_tree" => pal(0x6fae3a, 0x6a462a, 0xf2df3c),
        "clover_tree" => pal(0x2f9a6c, 0x5a3e28, 0xe8f4e6),
        "garlic_tree" => pal(0x878a5e, 0x62492e, 0xe8e4c8),
        "paradox_tree" => pal(0x7a5590, 0x4a3426, 0xc89ae0),
        "enchanted_tree" => pal(0x7ad25a, 0x5a4a2a, 0xdcffa8),
        "flower_tree_1" => pal(0x4c9e38, 0x6a462a, 0xf08cc0),
        "flower_tree_2" => pal(0x4a9a3a, 0x6a462a, 0xf6f0f6),
        "flower_tree_3" => pal(0x52a23a, 0x6a462a, 0xf6d640),
        "jungle_tree" => pal(0x2a7c34, 0x5a3c22, 0xe6d24a),
        "rocklands_tree" => pal(0x6e8052, 0x6a6660, 0),
        "wasteland_tree" => pal(0x7a6a52, 0x5e5040, 0),
        "corrupted_tree" | "corrupted_tree_big" => pal(0x6a3c6e, 0x3a263a, 0xa86ab0),
        "infernal_tree" | "infernal_tree_big" | "infernal_tree_small" => {
            pal(0x8c2218, 0x2a1a18, 0xf07a2a)
        }
        "candy_tree" => pal(0xf68cb8, 0xf0e6e0, 0xfff4fa),
        "mushroom_tree" => pal(0xb83a34, 0xe6dcc8, 0xf6f2ea),
        "crystal_tree" => pal(0x68e0d8, 0x3c8a86, 0xd8fffa),
        "celestial_tree" | "celestial_tree_small" => pal(0xb078e0, 0x5a3a7a, 0xf0d0ff),
        "singularity_tree" => pal(0x4a225e, 0x2a1438, 0x9a5ad0),
        "pine_tree" => pal(0x2e6e58, 0x5a3c26, 0),
        "savanna_tree_1" | "savanna_tree_2" | "savanna_tree_big_1" | "savanna_tree_big_2" => {
            pal(0x7a8a2c, 0x6a4a2a, 0)
        }
        "swamp_tree" => pal(0x4a5e3a, 0x4a3c2c, 0x6e8a4e),
        "desert_tree" | "palm_tree" => pal(0x5a9a3a, 0x9a7a4a, 0),
        "mushroom_red" => pal(0x3e8a34, 0xece4d2, 0xc63a34),
        "mushroom_green" => pal(0x3e8a34, 0xece4d2, 0x5ab84a),
        "mushroom_teal" => pal(0x3e8a34, 0xece4d2, 0x3ab8b0),
        "mushroom_white" => pal(0x3e8a34, 0xece4d2, 0xf2f0ea),
        "mushroom_yellow" => pal(0x3e8a34, 0xece4d2, 0xeed23e),
        "flower" | "flower_plant" => pal(0x3e8a34, 0x3e8a34, 0xe84a5a),
        "flame_flower" => pal(0x5a2a20, 0x5a2a20, 0xf8782a),
        "jungle_flower" => pal(0x2e7a34, 0x2e7a34, 0xf6c03a),
        "wasteland_flower" => pal(0x6e7656, 0x6e7656, 0xc8b45a),
        "green_herb" => pal(0x4c9a36, 0x3e7e2c, 0),
        "birch_plant" => pal(0x7cb83c, 0x5e9430, 0),
        "candy_plant" => pal(0xf290bc, 0xe06aa0, 0xfff4fa),
        "celestial_plant" => pal(0xa874d8, 0x7a52aa, 0xf0d0ff),
        "clover_plant" => pal(0x3aa070, 0x2a7e56, 0xf0f6ee),
        "corrupted_plant" => pal(0x6a4a68, 0x4e3450, 0),
        "crystal_plant" => pal(0x62d8cc, 0x3c9a92, 0xd8fffa),
        "desert_plant" => pal(0x8a9a4a, 0x6a7a36, 0),
        "garlic_plant" => pal(0x9a9a6c, 0x7a7a52, 0xece8d0),
        "jungle_plant" => pal(0x2e8a3a, 0x226a2c, 0),
        "maple_plant" => pal(0xc0682e, 0x9a4e22, 0),
        "paradox_plant" => pal(0x84607a, 0x644456, 0),
        "rocklands_plant" => pal(0x7a8a5e, 0x5e6c46, 0),
        "savanna_plant" => pal(0xb89a3a, 0x8a742a, 0),
        "singularity_plant" => pal(0x5a2a72, 0x3e1c52, 0x9a5ad0),
        "snow_plant" => pal(0x8ab0c8, 0x6a90a8, 0xeef6fa),
        "swamp_plant" | "swamp_plant_big" => pal(0x5a6e3e, 0x46562e, 0x7a6a3a),
        "fruit_bush" => pal(0x3e8e30, 0x2e6a26, 0xc8303a),
        "mineral_stone" => pal(0x8c8e92, 0x5e6064, 0),
        "mineral_metals" => pal(0x8a8c90, 0x5e6064, 0xc07a40),
        "mineral_silver" => pal(0x8a8e96, 0x5c6068, 0xeaeef4),
        "mineral_mythril" => pal(0x868c96, 0x5a5e68, 0x5ae0c8),
        "mineral_adamantine" => pal(0x7e7c86, 0x56545e, 0xd0444a),
        "mineral_gold" => pal(0x8e8a84, 0x625e58, 0xf2c23a),
        "mineral_gems" => pal(0x868890, 0x5a5c64, 0x6ad0f0),
        "mineral_bones" => pal(0xd8cca8, 0xa89a7a, 0xf0e8d4),
        _ => pal(0x4f8c36, 0x6a462a, 0),
    }
}

struct Dice(u32);

impl Dice {
    fn new(id: &str, salt: u32) -> Self {
        let mut hash: u32 = 0x811c_9dc5;
        for byte in id.bytes() {
            hash ^= u32::from(byte);
            hash = hash.wrapping_mul(0x0100_0193);
        }
        hash ^= salt.wrapping_mul(0x9e37_79b9);
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

    fn unit(&mut self) -> f32 {
        (self.next() >> 8) as f32 / (1 << 24) as f32
    }
}

struct Canvas {
    map: Pixmap,
    w: i32,
    h: i32,
}

impl Canvas {
    fn new(variant: &Variant) -> Self {
        let w = i32::from(variant.width).clamp(1, 255);
        let h = i32::from(variant.height).clamp(1, 255);
        Self {
            map: Pixmap::blank(w as u8, h as u8, 0, 0),
            w,
            h,
        }
    }

    fn put(&mut self, x: i32, y: i32, rgb: [u8; 3]) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.map.put(x as u8, y as u8, rgb);
        }
    }

    fn filled(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.w && y < self.h && self.map.filled(x as u8, y as u8)
    }

    fn up(&self, from_bottom: i32) -> i32 {
        self.h - 1 - from_bottom
    }

    fn blob(&mut self, cx: f32, cy: f32, rx: f32, ry: f32, rgb: [u8; 3]) {
        for y in 0..self.h {
            for x in 0..self.w {
                let dx = (x as f32 + 0.5 - cx) / rx.max(0.5);
                let dy = (y as f32 + 0.5 - cy) / ry.max(0.5);
                if dx * dx + dy * dy <= 1.0 {
                    self.put(x, y, rgb);
                }
            }
        }
    }

    fn shade(&mut self, dice: &mut Dice, base: [u8; 3], light_share: u32, dark_share: u32) {
        let light = lighten(scale(base, 1.12), 0.08);
        let dark = scale(base, 0.74);
        let deep = scale(base, 0.58);
        let cells: Vec<(i32, i32)> = (0..self.h)
            .flat_map(|y| (0..self.w).map(move |x| (x, y)))
            .filter(|&(x, y)| self.filled(x, y))
            .collect();

        for (x, y) in cells {
            if self.map.at(x as u8, y as u8)[..3] != base[..] {
                continue;
            }
            let open_top = !self.filled(x, y - 1);
            let open_left = !self.filled(x - 1, y);
            let open_bottom = !self.filled(x, y + 1);
            let open_right = !self.filled(x + 1, y);
            let rgb = if (open_bottom || open_right) && dice.chance(70) {
                if open_bottom && open_right {
                    deep
                } else {
                    dark
                }
            } else if ((open_top || open_left) && dice.chance(60)) || dice.chance(light_share) {
                light
            } else if dice.chance(dark_share) {
                dark
            } else {
                base
            };
            self.put(x, y, rgb);
        }
    }

    fn sprinkle(&mut self, dice: &mut Dice, rgb: [u8; 3], count: u32, above: i32) {
        let mut placed = 0;
        let mut tries = 0;
        while placed < count && tries < 200 {
            tries += 1;
            let x = dice.below(self.w as u32) as i32;
            let y = dice.below((self.h - above).max(1) as u32) as i32;
            if self.filled(x, y) {
                self.put(x, y, rgb);
                placed += 1;
            }
        }
    }

    fn line(&mut self, from: (f32, f32), to: (f32, f32), rgb: [u8; 3]) {
        let steps = ((to.0 - from.0).abs().max((to.1 - from.1).abs()).ceil() as i32).max(1);
        for step in 0..=steps {
            let t = step as f32 / steps as f32;
            let x = from.0 + (to.0 - from.0) * t;
            let y = from.1 + (to.1 - from.1) * t;
            self.put(x.floor() as i32, y.floor() as i32, rgb);
        }
    }
}

fn trunk(canvas: &mut Canvas, pivot_x: f32, base: i32, top: i32, bark: [u8; 3], wide: bool) {
    let x = pivot_x.floor() as i32;
    let dark = scale(bark, 0.72);
    for y in top..=base {
        canvas.put(x, y, bark);
        if wide {
            canvas.put(x - 1, y, bark);
            canvas.put(x, y, dark);
        }
    }
    canvas.put(x - 1, base, dark);
    canvas.put(x + 1, base, dark);
}

fn round(canvas: &mut Canvas, dice: &mut Dice, variant: &Variant, palette: Palette, frame: u32) {
    let w = canvas.w as f32;
    let h = canvas.h as f32;
    let base = canvas.up(variant.pivot_y.floor() as i32);
    let crown_bottom = (h * 0.62).floor() as i32;
    trunk(
        canvas,
        variant.pivot_x,
        base,
        crown_bottom - 2,
        palette.bark,
        canvas.w >= 12,
    );

    let crown_h = h * 0.68;
    let cy = crown_h * 0.52;
    canvas.blob(w * 0.5, cy, w * 0.5, crown_h * 0.52, palette.leaf);
    let lumps = 2 + dice.below(3);
    for _ in 0..lumps {
        let lx = w * (0.22 + dice.unit() * 0.56);
        let ly = crown_h * (0.25 + dice.unit() * 0.55);
        let r = (w * (0.16 + dice.unit() * 0.14)).max(1.5);
        canvas.blob(lx, ly, r, r * 0.9, palette.leaf);
    }
    canvas.shade(dice, palette.leaf, 22, 18);

    if let Some(accent) = palette.accent {
        let count = 2 + (canvas.w * canvas.h / 40) as u32 + frame % 2;
        canvas.sprinkle(dice, accent, count, (h * 0.4) as i32);
    }
}

fn conifer(canvas: &mut Canvas, dice: &mut Dice, variant: &Variant, palette: Palette) {
    let w = canvas.w as f32;
    let base = canvas.up(variant.pivot_y.floor() as i32);
    let tiers = 3 + (canvas.h > 20) as i32;
    let foliage_bottom = base - 2;
    trunk(
        canvas,
        variant.pivot_x,
        base,
        foliage_bottom,
        palette.bark,
        false,
    );
    let span = foliage_bottom as f32;
    for tier in 0..tiers {
        let top = span * tier as f32 / tiers as f32 * 0.85;
        let bottom = span * (tier + 1) as f32 / tiers as f32;
        let half = w * 0.5 * (0.45 + 0.55 * (tier + 1) as f32 / tiers as f32);
        for y in top.floor() as i32..=bottom.floor() as i32 {
            let t = (y as f32 - top) / (bottom - top).max(1.0);
            let reach = (half * t.clamp(0.15, 1.0)).max(0.5);
            for x in 0..canvas.w {
                if (x as f32 + 0.5 - w * 0.5).abs() <= reach {
                    canvas.put(x, y, palette.leaf);
                }
            }
        }
    }
    canvas.shade(dice, palette.leaf, 18, 24);
}

fn palm(canvas: &mut Canvas, dice: &mut Dice, variant: &Variant, palette: Palette) {
    let base = canvas.up(variant.pivot_y.floor() as i32);
    let top_x = canvas.w as f32 * (0.35 + dice.unit() * 0.3);
    let crown_y = (canvas.h as f32 * 0.28).max(2.0);
    canvas.line(
        (variant.pivot_x, base as f32),
        (top_x, crown_y),
        palette.bark,
    );
    canvas.line(
        (variant.pivot_x + 0.9, base as f32),
        (top_x + 0.9, crown_y + 1.0),
        scale(palette.bark, 0.75),
    );
    let fronds = 5 + dice.below(2);
    for frond in 0..fronds {
        let angle = std::f32::consts::PI * (frond as f32 / (fronds - 1) as f32) + 0.15;
        let reach = canvas.w as f32 * 0.5;
        let end = (
            top_x - angle.cos() * reach,
            crown_y - angle.sin() * reach * 0.45 + reach * 0.25,
        );
        canvas.line((top_x, crown_y), end, palette.leaf);
    }
    canvas.blob(top_x, crown_y, 1.6, 1.2, palette.leaf);
    canvas.shade(dice, palette.leaf, 20, 20);
}

fn cactus(canvas: &mut Canvas, dice: &mut Dice, variant: &Variant, palette: Palette) {
    let base = canvas.up(variant.pivot_y.floor() as i32);
    let x = variant.pivot_x.floor() as i32;
    for y in 1..=base {
        canvas.put(x, y, palette.leaf);
        canvas.put(x - 1, y, palette.leaf);
    }
    let arm = (canvas.h / 2).max(2);
    for y in arm - 2..arm + 1 {
        canvas.put(x - 3, y, palette.leaf);
        canvas.put(x + 2, y + 1, palette.leaf);
    }
    canvas.put(x - 2, arm, palette.leaf);
    canvas.put(x + 1, arm + 1, palette.leaf);
    canvas.shade(dice, palette.leaf, 20, 20);
}

fn umbrella(canvas: &mut Canvas, dice: &mut Dice, variant: &Variant, palette: Palette) {
    let w = canvas.w as f32;
    let h = canvas.h as f32;
    let base = canvas.up(variant.pivot_y.floor() as i32);
    let crown_y = h * 0.28;
    canvas.line(
        (variant.pivot_x, base as f32),
        (w * 0.5, crown_y + 1.0),
        palette.bark,
    );
    canvas.line(
        (w * 0.5, crown_y + h * 0.25),
        (w * 0.25, crown_y + 1.0),
        palette.bark,
    );
    canvas.blob(w * 0.5, crown_y, w * 0.5, (h * 0.2).max(1.5), palette.leaf);
    canvas.blob(
        w * 0.3,
        crown_y - 0.5,
        w * 0.22,
        (h * 0.13).max(1.0),
        palette.leaf,
    );
    canvas.shade(dice, palette.leaf, 18, 22);
}

fn willow(canvas: &mut Canvas, dice: &mut Dice, variant: &Variant, palette: Palette) {
    let w = canvas.w as f32;
    let h = canvas.h as f32;
    let base = canvas.up(variant.pivot_y.floor() as i32);
    trunk(
        canvas,
        variant.pivot_x,
        base,
        (h * 0.45) as i32,
        palette.bark,
        canvas.w >= 12,
    );
    canvas.blob(w * 0.5, h * 0.3, w * 0.5, h * 0.3, palette.leaf);
    let strands = (canvas.w / 2).max(2);
    for _ in 0..strands {
        let x = dice.below(canvas.w as u32) as i32;
        let from = (h * 0.4) as i32;
        let to = from + (h * (0.2 + dice.unit() * 0.25)) as i32;
        for y in from..to.min(base) {
            if canvas.filled(x, from - 1) || dice.chance(80) {
                canvas.put(x, y, palette.leaf);
            }
        }
    }
    canvas.shade(dice, palette.leaf, 15, 25);
    if let Some(accent) = palette.accent {
        canvas.sprinkle(dice, accent, 3, (h * 0.3) as i32);
    }
}

fn cap(canvas: &mut Canvas, dice: &mut Dice, variant: &Variant, palette: Palette) {
    let w = canvas.w as f32;
    let h = canvas.h as f32;
    let base = canvas.up(variant.pivot_y.floor() as i32);
    let stem = (w * 0.12).max(1.0) as i32;
    let x = variant.pivot_x.floor() as i32;
    for y in (h * 0.45) as i32..=base {
        for dx in -stem..=stem {
            canvas.put(x + dx, y, palette.bark);
        }
    }
    canvas.blob(w * 0.5, h * 0.32, w * 0.5, h * 0.32, palette.leaf);
    for x in 0..canvas.w {
        let y = (h * 0.6) as i32;
        if canvas.filled(x, y - 1) {
            canvas.put(x, y - 1, scale(palette.leaf, 0.7));
        }
    }
    canvas.shade(dice, palette.leaf, 20, 10);
    if let Some(accent) = palette.accent {
        canvas.sprinkle(dice, accent, 3 + (canvas.w as u32 / 4), (h * 0.5) as i32);
    }
}

fn shard(canvas: &mut Canvas, dice: &mut Dice, variant: &Variant, palette: Palette) {
    let base = canvas.up(variant.pivot_y.floor() as i32);
    let spikes = 3 + dice.below(3);
    for _ in 0..spikes {
        let x = (canvas.w as f32 * (0.15 + dice.unit() * 0.7)) as i32;
        let top = (canvas.h as f32 * (0.05 + dice.unit() * 0.45)) as i32;
        for y in top..=base {
            let half = ((y - top) / 3).min(2);
            for dx in -half..=half {
                canvas.put(x + dx, y, palette.leaf);
            }
        }
    }
    canvas.shade(dice, palette.leaf, 30, 15);
    if let Some(accent) = palette.accent {
        canvas.sprinkle(dice, accent, 4, 1);
    }
}

fn dead(canvas: &mut Canvas, dice: &mut Dice, variant: &Variant, palette: Palette) {
    let base = canvas.up(variant.pivot_y.floor() as i32) as f32;
    let top = (canvas.h as f32 * 0.2, variant.pivot_x);
    canvas.line((variant.pivot_x, base), (top.1, top.0), palette.leaf);
    let branches = 3 + dice.below(3);
    for _ in 0..branches {
        let y = top.0 + (base - top.0) * dice.unit() * 0.6;
        let dir = if dice.chance(50) { -1.0 } else { 1.0 };
        let reach = canvas.w as f32 * (0.2 + dice.unit() * 0.3);
        canvas.line(
            (variant.pivot_x, y),
            (variant.pivot_x + dir * reach, y - reach * 0.6),
            palette.leaf,
        );
    }
    canvas.shade(dice, palette.leaf, 15, 25);
}

fn thorn(canvas: &mut Canvas, dice: &mut Dice, variant: &Variant, palette: Palette) {
    round(canvas, dice, variant, palette, 0);
    let w = canvas.w;
    for _ in 0..(w / 2).max(2) {
        let x = dice.below(w as u32) as i32;
        let mut y = 0;
        while y < canvas.h && !canvas.filled(x, y) {
            y += 1;
        }
        if y > 0 && y < canvas.h {
            canvas.put(x, y - 1, scale(palette.leaf, 0.6));
        }
    }
}

fn glow(canvas: &mut Canvas, dice: &mut Dice, variant: &Variant, palette: Palette, frame: u32) {
    round(canvas, dice, variant, palette, 0);
    let mut twinkle = Dice::new("glow", frame.wrapping_mul(7919) ^ u32::from(variant.width));
    if let Some(accent) = palette.accent {
        let count = 4 + (canvas.w * canvas.h / 60) as u32;
        canvas.sprinkle(&mut twinkle, accent, count, (canvas.h / 3).max(1));
    }
}

fn toadstool(canvas: &mut Canvas, palette: Palette) {
    let accent = palette.accent.unwrap_or(palette.leaf);
    let x = canvas.w / 2;
    let base = canvas.h - 1;
    canvas.put(x, base, palette.bark);
    canvas.put(x, base - 1, palette.bark);
    for dx in -1..=1 {
        canvas.put(x + dx, base - 2, accent);
    }
    canvas.put(x, base - 3, lighten(accent, 0.2));
    canvas.put(x + 1, base - 2, scale(accent, 0.8));
}

fn reed(canvas: &mut Canvas, dice: &mut Dice, palette: Palette) {
    let stalks = (canvas.w / 2).max(2);
    for _ in 0..stalks {
        let x = dice.below(canvas.w as u32) as i32;
        let top = (canvas.h as f32 * (0.1 + dice.unit() * 0.5)) as i32;
        for y in top..canvas.h {
            canvas.put(x, y, palette.leaf);
        }
        if let Some(accent) = palette.accent {
            canvas.put(x, top, accent);
            canvas.put(x, top + 1, accent);
        }
    }
    canvas.shade(dice, palette.leaf, 20, 20);
}

fn blossom(canvas: &mut Canvas, dice: &mut Dice, palette: Palette, variant_index: usize) {
    let petals = [
        [0xe8, 0x4a, 0x5a],
        [0xf2, 0xd2, 0x3e],
        [0x5a, 0x8a, 0xe8],
        [0xf4, 0xf2, 0xf6],
        [0xe8, 0x7a, 0xc8],
        [0xa0, 0x6a, 0xe0],
    ];
    let petal = if palette.accent == Some([0xe8, 0x4a, 0x5a]) {
        petals[variant_index % petals.len()]
    } else {
        palette.accent.unwrap_or(petals[0])
    };
    let stems = 1 + dice.below(2) as i32;
    for stem in 0..stems {
        let x = (canvas.w / 2) + stem * 2 - stems / 2;
        let top = canvas.h - 3 - dice.below(2) as i32;
        for y in top..canvas.h {
            canvas.put(x, y, palette.leaf);
        }
        canvas.put(x, top - 1, petal);
        canvas.put(x - 1, top, scale(petal, 0.85));
    }
}

fn sprig(canvas: &mut Canvas, dice: &mut Dice, palette: Palette) {
    let base = canvas.h - 1;
    let blades = 3 + dice.below(2) as i32;
    for blade in 0..blades {
        let x = 1 + blade * (canvas.w - 2).max(1) / blades.max(1);
        let height = 2 + dice.below(canvas.h.max(3) as u32 - 2) as i32;
        for y in (base - height + 1)..=base {
            canvas.put(
                x,
                y,
                if y == base {
                    palette.bark
                } else {
                    palette.leaf
                },
            );
        }
    }
    if let Some(accent) = palette.accent {
        canvas.sprinkle(dice, accent, 1, 2);
    }
}

fn bush(canvas: &mut Canvas, dice: &mut Dice, palette: Palette, bare: bool) {
    let w = canvas.w as f32;
    let h = canvas.h as f32;
    canvas.blob(w * 0.5, h * 0.55, w * 0.48, h * 0.5, palette.leaf);
    canvas.blob(w * 0.3, h * 0.45, w * 0.22, h * 0.36, palette.leaf);
    canvas.shade(dice, palette.leaf, 18, 18);
    if !bare && let Some(accent) = palette.accent {
        canvas.sprinkle(dice, accent, 3 + (canvas.w as u32 / 4), 1);
    }
}

fn rock(canvas: &mut Canvas, dice: &mut Dice, palette: Palette) {
    let w = canvas.w as f32;
    let h = canvas.h as f32;
    canvas.blob(w * 0.5, h * 0.62, w * 0.5, h * 0.42, palette.leaf);
    canvas.blob(w * 0.38, h * 0.48, w * 0.3, h * 0.36, palette.leaf);
    canvas.shade(dice, palette.leaf, 25, 20);
    if let Some(accent) = palette.accent {
        let count = 2 + (canvas.w * canvas.h / 18) as u32;
        canvas.sprinkle(dice, accent, count, 1);
        canvas.sprinkle(dice, lighten(accent, 0.4), 1, 1);
    }
}

pub fn frame(kind: &Kind, variant_index: usize, frame: u32, bare: bool) -> Pixmap {
    let list = if bare && !kind.bare.is_empty() {
        kind.bare
    } else {
        kind.variants
    };
    let variant = list[variant_index % list.len()];
    let palette = palette(kind.id);
    let mut dice = Dice::new(kind.id, variant_index as u32 * 131 + u32::from(bare));
    let mut canvas = Canvas::new(&variant);

    match kind.shape {
        Shape::Round => round(&mut canvas, &mut dice, &variant, palette, frame),
        Shape::Conifer => conifer(&mut canvas, &mut dice, &variant, palette),
        Shape::Palm => palm(&mut canvas, &mut dice, &variant, palette),
        Shape::Cactus => cactus(&mut canvas, &mut dice, &variant, palette),
        Shape::Umbrella => umbrella(&mut canvas, &mut dice, &variant, palette),
        Shape::Willow => willow(&mut canvas, &mut dice, &variant, palette),
        Shape::Cap => cap(&mut canvas, &mut dice, &variant, palette),
        Shape::Shard => shard(&mut canvas, &mut dice, &variant, palette),
        Shape::Dead => dead(&mut canvas, &mut dice, &variant, palette),
        Shape::Thorn => thorn(&mut canvas, &mut dice, &variant, palette),
        Shape::Glow => glow(&mut canvas, &mut dice, &variant, palette, frame),
        Shape::Toadstool => toadstool(&mut canvas, palette),
        Shape::Reed => reed(&mut canvas, &mut dice, palette),
        Shape::Blossom => blossom(&mut canvas, &mut dice, palette, variant_index),
        Shape::Sprig => sprig(&mut canvas, &mut dice, palette),
        Shape::Bush => bush(&mut canvas, &mut dice, palette, bare),
        Shape::Rock => rock(&mut canvas, &mut dice, palette),
    }

    canvas.map
}

pub fn mini(kind: &Kind) -> Vec<[u8; 4]> {
    let (mw, mh) = (usize::from(kind.mini.0), usize::from(kind.mini.1));
    let sprite = frame(kind, 0, 0, false);
    let (sw, sh) = (usize::from(sprite.width), usize::from(sprite.height));
    let mut icon = vec![[0, 0, 0, 0]; mw * mh];

    for my in 0..mh {
        for mx in 0..mw {
            let x0 = mx * sw / mw;
            let x1 = ((mx + 1) * sw / mw).max(x0 + 1);
            let y0 = my * sh / mh;
            let y1 = ((my + 1) * sh / mh).max(y0 + 1);
            let mut counts: Vec<([u8; 4], u32)> = Vec::new();
            for y in y0..y1.min(sh) {
                for x in x0..x1.min(sw) {
                    let pixel = sprite.at(x as u8, y as u8);
                    if pixel[3] == 0 {
                        continue;
                    }
                    match counts.iter_mut().find(|(c, _)| *c == pixel) {
                        Some((_, n)) => *n += 1,
                        None => counts.push((pixel, 1)),
                    }
                }
            }
            if let Some((pixel, _)) = counts.iter().max_by_key(|(_, n)| *n) {
                icon[my * mw + mx] = *pixel;
            }
        }
    }

    icon
}

pub fn wall_frame(body: [u8; 3], edge: [u8; 3], frame: u32) -> Pixmap {
    let cap = lighten(body, 0.35);
    let side = scale(edge, 0.62);
    let shadow = [0, 0, 0, 118];
    let mut map = Pixmap::blank(8, 8, 0, 0);
    let rows: [(u8, u8, u8); 4] = [(1, 2, 4), (2, 2, 4), (3, 2, 4), (4, 2, 4)];
    for (y, a, b) in rows {
        for x in a..=b {
            map.put(x, y, body);
        }
    }
    map.put(3, 0, lighten(body, 0.18));
    map.put(4, 0, lighten(body, 0.18));
    map.put(3, 1, cap);
    map.put(4, 1, cap);
    map.put(3, 2, cap);
    for y in 5..8 {
        for x in 2..=4 {
            map.put(x, y, edge);
        }
    }
    for y in 1..8 {
        map.put(5, y, side);
    }
    let mut dice = Dice::new("wall", frame);
    for _ in 0..(frame % 3) {
        let x = 2 + dice.below(3) as u8;
        let y = 2 + dice.below(5) as u8;
        map.put(x, y, mix(body, edge, 0.5));
    }
    for (x, y) in [(6, 4), (6, 5), (7, 5), (6, 6), (7, 6), (6, 7), (7, 7)] {
        map.paint(x, y, shadow);
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::things::{self, KINDS};

    #[test]
    fn every_variant_and_frame_has_the_worldbox_size_and_some_pixels() {
        for kind in KINDS {
            for (index, variant) in kind.variants.iter().enumerate() {
                for step in [0, u32::from(variant.frames).saturating_sub(1)] {
                    let sprite = frame(kind, index, step, false);
                    assert_eq!(
                        (u16::from(sprite.width), u16::from(sprite.height)),
                        (variant.width.min(255), variant.height.min(255)),
                        "{} {index}",
                        kind.id
                    );
                    assert!(sprite.opaque_count() > 0, "{} {index}", kind.id);
                }
            }
        }
    }

    #[test]
    fn a_tree_stands_on_its_pivot() {
        let sprite = frame(&things::TREE_GREEN_1, 1, 0, false);
        let variant = things::TREE_GREEN_1.variants[1];
        let row = sprite.height - 1 - variant.pivot_y as u8;

        assert!((0..sprite.width).any(|x| sprite.filled(x, row)));
    }

    #[test]
    fn the_map_icon_has_the_worldbox_size() {
        assert_eq!(mini(&things::TREE_GREEN_1).len(), 3 * 2);
        assert_eq!(mini(&things::CELESTIAL_TREE).len(), 5 * 3);
        assert!(
            mini(&things::MINERAL_GOLD)
                .iter()
                .all(|pixel| pixel[3] == 255)
        );
    }

    #[test]
    fn a_picked_bush_loses_its_berries() {
        let full = frame(&things::FRUIT_BUSH, 0, 0, false);
        let bare = frame(&things::FRUIT_BUSH, 0, 0, true);
        let berry = palette("fruit_bush").accent.unwrap();
        let has = |sprite: &Pixmap| sprite.pixels.iter().any(|p| p[..3] == berry[..]);

        assert!(has(&full));
        assert!(!has(&bare));
    }

    #[test]
    fn walls_have_six_distinct_looking_frames_on_the_same_footprint() {
        let frames: Vec<Pixmap> = (0..6)
            .map(|step| wall_frame([0xa1, 0xb1, 0xa2], [0x78, 0x7f, 0x87], step))
            .collect();

        assert!(frames.iter().all(|f| f.width == 8 && f.height == 8));
        assert!(frames.windows(2).any(|pair| pair[0] != pair[1]));
    }
}
