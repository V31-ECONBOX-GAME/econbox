//! Original pixel sprites, drawn from a handful of stance templates and a per creature palette.

pub mod glyph;
pub mod ui;

pub use glyph::{GLYPHS, Glyph, glyph};

pub const SPRITE: usize = 16;
pub const PIXELS: usize = SPRITE * SPRITE;

pub const CLEAR: [u8; 4] = [0, 0, 0, 0];
pub const EYE: [u8; 3] = [0x1a, 0x1c, 0x24];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stance {
    Upright,
    Quadruped,
    Bird,
    Squat,
    Swimmer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Plain,
    Horns,
    LongEars,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Skin {
    pub coat: [u8; 3],
    pub accent: [u8; 3],
}

const UPRIGHT: [&str; SPRITE] = [
    "................",
    ".....DDDDDD.....",
    "....DCCCCCCD....",
    "....CCCCCCCC....",
    "....CCECCECC....",
    "....CCCCCCCC....",
    ".....CCCCCC.....",
    "...AAAAAAAAAA...",
    "...AACCCCCCAA...",
    "...AAAAAAAAAA...",
    "....AAAAAAAA....",
    "....AAAAAAAA....",
    "....DDD..DDD....",
    "....DDD..DDD....",
    "...DDDD..DDDD...",
    "................",
];

const QUADRUPED: [&str; SPRITE] = [
    "................",
    "................",
    "............DDD.",
    "...........DCCCD",
    "...........CCECC",
    "..D........CCCCC",
    "..DD.DDDDDDCCCC.",
    "..DDDCCCCCCCCCC.",
    "...DCCCCCCCCCC..",
    "....AAAAAAAAA...",
    "....CC...CC.....",
    "....CC...CC.....",
    "....DD...DD.....",
    "................",
    "................",
    "................",
];

const BIRD: [&str; SPRITE] = [
    "................",
    "..........DD....",
    ".........DCCD...",
    ".........CECC...",
    ".........CCCCA..",
    "....DDDDDCCCC...",
    "...DCCCCCCCCC...",
    "..DCCCCCCCCCC...",
    "..CCCCCCCCCCC...",
    "..CCCCCCCCCC....",
    "...CCCCCCCC.....",
    ".....AA.AA......",
    ".....AA.AA......",
    "....AAA.AAA.....",
    "................",
    "................",
];

const SQUAT: [&str; SPRITE] = [
    "................",
    "................",
    "...D........D...",
    "..DCD......DCD..",
    "..DCCD....DCCD..",
    "...DCCCCCCCCD...",
    "..DCCCCCCCCCCD..",
    ".DCCECCCCCCECCD.",
    ".DCCCCCCCCCCCCD.",
    ".DCCCCCCCCCCCCD.",
    "..DCCCCCCCCCCD..",
    "...DDCCCCCCDD...",
    "..DD..D..D..DD..",
    "................",
    "................",
    "................",
];

const SWIMMER: [&str; SPRITE] = [
    "................",
    "................",
    "................",
    "...D............",
    "..DD.....DDD....",
    ".DDDDDDDDCCCCD..",
    ".DDDCCCCCCCCCCD.",
    ".DDCCCCCCCCECCCD",
    ".DDDCCCCCCCCCCD.",
    ".DDDDDDDDCCCCD..",
    "..DD.....DDD....",
    "...D............",
    "................",
    "................",
    "................",
    "................",
];

impl Stance {
    pub fn template(self) -> &'static [&'static str; SPRITE] {
        match self {
            Self::Upright => &UPRIGHT,
            Self::Quadruped => &QUADRUPED,
            Self::Bird => &BIRD,
            Self::Squat => &SQUAT,
            Self::Swimmer => &SWIMMER,
        }
    }
}

impl Mark {
    pub fn pixels(self, stance: Stance) -> &'static [(usize, usize)] {
        match (self, stance) {
            (Self::Horns, Stance::Upright) => &[(4, 0), (4, 1), (11, 0), (11, 1)],
            (Self::Horns, Stance::Quadruped) => &[(11, 2), (15, 2), (11, 1), (15, 1)],
            (Self::LongEars, Stance::Quadruped) => &[(12, 0), (12, 1), (14, 0), (14, 1)],
            (Self::LongEars, Stance::Upright) => &[(4, 0), (4, 1), (11, 0), (11, 1)],
            _ => &[],
        }
    }
}

pub fn shade(color: [u8; 3], by: f32) -> [u8; 3] {
    color.map(|channel| (f32::from(channel) * by).clamp(0.0, 255.0) as u8)
}

fn opaque(color: [u8; 3]) -> [u8; 4] {
    [color[0], color[1], color[2], 255]
}

pub fn sprite(stance: Stance, mark: Mark, skin: Skin) -> Vec<[u8; 4]> {
    let dark = shade(skin.coat, 0.62);
    let mut out = vec![CLEAR; PIXELS];

    for (y, row) in stance.template().iter().enumerate() {
        for (x, glyph) in row.chars().enumerate() {
            out[y * SPRITE + x] = match glyph {
                'C' => opaque(skin.coat),
                'D' => opaque(dark),
                'A' => opaque(skin.accent),
                'E' => opaque(EYE),
                _ => CLEAR,
            };
        }
    }

    for (x, y) in mark.pixels(stance) {
        out[y * SPRITE + x] = opaque(dark);
    }

    out
}

pub fn atlas(frames: &[Vec<[u8; 4]>]) -> (Vec<u8>, u32, u32) {
    let width = (frames.len() * SPRITE) as u32;
    let mut data = vec![0_u8; width as usize * SPRITE * 4];

    for (slot, frame) in frames.iter().enumerate() {
        for y in 0..SPRITE {
            for x in 0..SPRITE {
                let at = (y * width as usize + slot * SPRITE + x) * 4;
                data[at..at + 4].copy_from_slice(&frame[y * SPRITE + x]);
            }
        }
    }

    (data, width, SPRITE as u32)
}

const PEAKS: [&str; SPRITE] = [
    "................",
    ".....W..........",
    "....WWW.........",
    "....WWWD...W....",
    "...LWWDD..WWD...",
    "...LLWCD..LWDD..",
    "..LLLWCDELLCDD..",
    "..LLLCCDELLCDD..",
    "..LLCCDDELLCCDD.",
    ".LLLCCDDELCCDDD.",
    ".LLCCCDDELCCDDD.",
    ".LLCCDDDECCDDDD.",
    ".LCCCDDDECCCDDD.",
    ".AAAAAAAAAAAAAA.",
    "..BBBBBBBBBBBB..",
    "................",
];

const SPRIG: [&str; SPRITE] = [
    "................",
    ".......LL.......",
    "......LWCC......",
    ".....LLWCCD.....",
    ".LL..LCCCDD.....",
    ".LLCC.CCDD...LL.",
    ".LCCCD.AB..LLCC.",
    "..LCCCDAB.LLCCD.",
    "...CCDDABLCCCD..",
    ".....DDABCCDD...",
    ".......ABDD.....",
    ".....LLAB.......",
    "...LLCCAB.......",
    "..LCCCDAB.......",
    "....DDDAB.......",
    "................",
];

const SPADE: [&str; SPRITE] = [
    "................",
    "............AAB.",
    "...........AAB..",
    "..........AAB...",
    ".........AAB....",
    "........AAB.....",
    ".......AAB......",
    "......AAB.......",
    ".....LCCD.......",
    "...LLCCCD.......",
    "..LWWCCCD.......",
    ".LWWCCCCDD......",
    ".LLCCCCDD.......",
    ".CCCCDDD........",
    "..CCDD..........",
    "................",
];

const PAW: [&str; SPRITE] = [
    "................",
    ".....L....L.....",
    "....LLL..LLL....",
    "....CCC..CCC....",
    "....DDD..DDD....",
    "..L..........L..",
    ".LLL........LLL.",
    ".CCC........CCC.",
    ".DDD........DDD.",
    ".....LLLLLL.....",
    "....LWWLLCCC....",
    "...LLCCCCCCCD...",
    "...CCCCCCCCDD...",
    "...CCCCCCDDDD...",
    "....DDDDDDDD....",
    "................",
];

const CHEVRON: [&str; SPRITE] = [
    "................",
    "................",
    "..LW............",
    "..LCWL..........",
    "..LCCLLL........",
    "..LCCCCLLL......",
    "..LCCCCCCLLL....",
    "..LCCCCCCCLLLL..",
    "..DCCCCCCCDDDD..",
    "..DCCCCCCDDD....",
    "..DCCCCDDD......",
    "..DCCDDD........",
    "..EDDD..........",
    "..EE............",
    "................",
    "................",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Plate {
    pub face: [u8; 3],
    pub edge: [u8; 3],
    pub inset: bool,
    pub grain: i32,
}

pub const PANEL: Plate = Plate {
    face: [0x4c, 0x56, 0x44],
    edge: [0x20, 0x26, 0x1c],
    inset: false,
    grain: 5,
};

pub const SLOT: Plate = Plate {
    face: [0x2b, 0x2e, 0x2b],
    edge: [0x14, 0x16, 0x14],
    inset: true,
    grain: 9,
};

pub const TAB: Plate = Plate {
    face: [0x5a, 0x64, 0x50],
    edge: [0x20, 0x26, 0x1c],
    inset: false,
    grain: 4,
};

pub const TAB_ON: Plate = Plate {
    face: [0xc0, 0x3c, 0x30],
    edge: [0x4e, 0x14, 0x10],
    inset: false,
    grain: 5,
};

pub const BUTTON: Plate = Plate {
    face: [0xc8, 0x42, 0x34],
    edge: [0x52, 0x16, 0x12],
    inset: false,
    grain: 4,
};

pub const RAIL: Plate = Plate {
    face: [0x3a, 0x42, 0x34],
    edge: [0x20, 0x26, 0x1c],
    inset: true,
    grain: 6,
};

pub const RAIL_PIECE: [&str; 24] = [
    "EEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEE",
    "EHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHE",
    "EHLLLLLLLLLLLLLLLLLLLLLLLLLLLLDE",
    "EHLMMMMMMMMMMMMMMMMMMMMMMMMMMLDE",
    "EHLMDDDDDDDDDDDDDDDDDDDDDDDDMLDE",
    "EHLMDBBMMMMMMMMMMMMMMMMMMBBDMLDE",
    "EHLMDMMMMMMMMMMMMMMMMMMMMMMDMLDE",
    "EHLMLLLLLLLLLLLLLLLLLLLLLLLLMLDE",
    "EHLMMMMMMMMMMMMMMMMMMMMMMMMMMLDE",
    "EHLLLLLLLLLLLLLLLLLLLLLLLLLLLLDE",
    "EHLMMMMMMMMMMMMMMMMMMMMMMMMMMLDE",
    "EHLMMMMMMMMMMMMMMMMMMMMMMMMMMLDE",
    "EHLMMMMMMMMMMMMMMMMMMMMMMMMMMLDE",
    "EHLMMMMMMMMMMMMMMMMMMMMMMMMMMLDE",
    "EHLMMMMMMMMMMMMMMMMMMMMMMMMMMLDE",
    "EHLMMMMMMMMMMMMMMMMMMMMMMMMMMLDE",
    "EHLMMMMMMMMMMMMMMMMMMMMMMMMMMLDE",
    "EHLLLLLLLLLLLLLLLLLLLLLLLLLLLLDE",
    "EHLMMMMMMMMMMMMMMMMMMMMMMMMMMLDE",
    "EHLMDDDDDDDDDDDDDDDDDDDDDDDDMLDE",
    "EHLMMMMMMMMMMMMMMMMMMMMMMMMMMLDE",
    "EHDDDDDDDDDDDDDDDDDDDDDDDDDDDDDE",
    "EDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDE",
    "EEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEE",
];

pub const CAP_PIECE: [&str; 24] = [
    "EEEEEEEEEEEEEE",
    "EHHHHHHHHHHHDE",
    "EHLBBMMMMBBLDE",
    "EHLMDMMMMDMLDE",
    "EHLLLLLLLLLLDE",
    "EHLMDDDDDDMLDE",
    "EHLMDHHHHDMLDE",
    "EHLMDDDDDDMLDE",
    "EHLMMMMMMMMLDE",
    "EHLMMMMMMMMLDE",
    "EHLMMMMMMMMLDE",
    "EHLMMMMMMMMLDE",
    "EHLMMMMMMMMLDE",
    "EHLMMMMMMMMLDE",
    "EHLMMMMMMMMLDE",
    "EHLMMMMMMMMLDE",
    "EHLMDDDDDDMLDE",
    "EHLMDHHHHDMLDE",
    "EHLMDDDDDDMLDE",
    "EHLLLLLLLLLLDE",
    "EHLMDMMMMDMLDE",
    "EHLBBMMMMBBLDE",
    "EHDDDDDDDDDDDE",
    "EEEEEEEEEEEEEE",
];

pub const KEY_PIECE: [&str; 24] = [
    "..OOOOOOOOOOOOOOOOOOOO..",
    ".ORRRRRRRRRRRRRRRRRRRRO.",
    "ORSSSSSSSSSSSSSSSSSSSSRO",
    "ORSHHHHHHHHHHHHHHHHHHSRO",
    "ORSHFFFFFFFFFFFFFFFFFSRO",
    "ORSHFFFFFFFFFFFFFFFFFSRO",
    "ORSHFFFFFFFFFFFFFFFFFSRO",
    "ORSHFFFFFFFFFFFKKKFFFSRO",
    "ORSHFFFFFFFFFFFFFKKFFSRO",
    "ORSHFFFFFFFFFFFFFFFKFSRO",
    "ORSHFFFFFFFFFFFFFFFFFSRO",
    "ORSHFFFFFFFFFFFFFFFFFSRO",
    "ORSHFKKFFFFFFFFFFFFFFSRO",
    "ORSHFFFKFFFFFFFFFFFFFSRO",
    "ORSHFFFFKKKFFFFFFFFFFSRO",
    "ORSHFFFFFFFKFFFFFFFFFSRO",
    "ORSHFFFFFFFFFFFFFFFFFSRO",
    "ORSHFFFFFFFFFFKKFFFFFSRO",
    "ORSHFFFFFFFFFFFFKKFFFSRO",
    "ORSHFFKFFFFFFFFFFFFFFSRO",
    "ORSHFFFFFFFFFFFFFFFFFSRO",
    "ORSSSSSSSSSSSSSSSSSSSSRO",
    ".ORRRRRRRRRRRRRRRRRRRRO.",
    "..OOOOOOOOOOOOOOOOOOOO..",
];

pub const LED_PIECE: [&str; 6] = ["..OO..", ".OHGO.", "OHGGGO", "OGGGDO", ".OGDO.", "..OO.."];

pub const DOT_PIECE: [&str; 5] = [".OOO.", "OFFFO", "OFFFO", "OFFFO", ".OOO."];

pub const PLUG_PIECE: [&str; 50] = [
    "..OOOOOOOOOOOOOOOOOOOOOO..",
    ".OHHHHHHHHHHHHHHHHHHHHHHO.",
    "OHUUUUUUUUUUUUUUUUUUUUUUDO",
    "OHUUUUUUUUUUUUUUUUUUUUUUDO",
    "OHUUUUUUUUUUUUUUUUUUUUUUDO",
    "OHUUUUUUUUUUUUUUUUUUUUUUDO",
    "OHUUUUUUUUUUUUUUUUUUUUUUDO",
    "OHUUUUUUUUUUUUUUUUUUUUUUDO",
    "OHUUUUUUUUUUUUUUUUUUUUUUDO",
    "OHUUUUUUUUUUUUUUUUUUUUUUDO",
    "OHUUUUUUUUUUUUUUUUUUUUUUDO",
    "OHUUUUUUUUUUUUUUUUUUUUUUDO",
    "OHUUUUUUUUUUUUUUUUUUUUUUDO",
    "OHUUUUUUUUUUUUUUUUUUUUUUDO",
    "OHUUUUUUUUUUUUUUUUUUUUUUDO",
    "OHUUUUUUUUUUUUUUUUUUUUUUDO",
    "OHUUUUUUUUUUUUUUUUUUUUUUDO",
    "OHUUUUUUUUUUUUUUUUUUUUUUDO",
    "OHUUUUUUUUUUUUUUUUUUUUUUDO",
    "OHUUUUUUUUUUUUUUUUUUUUUUDO",
    "OHUUUUUUUUUUUUUUUUUUUUUUDO",
    "OHUUUUUUUUUUUUUUUUUUUUUUDO",
    "OHUUUUUUUUUUUUUUUUUUUUUUDO",
    "OHUUUUUUUUUUUUUUUUUUUUUUDO",
    "OHUUUUUUUUUUUUUUUUUUUUUUDO",
    "OHWWWWWWWWWWWWWWWWWWWWWWDO",
    "OHWWWWWWWWWWWWWWWWWWWWWWDO",
    "OHWWWWWWWWWWWWWWWWWWWWWWDO",
    "OHWWWWWWWWWWWWWWWWWWWWWWDO",
    "OHWWWWWWWWWWWWWWWWWWWWWWDO",
    "OHWWWWWWWWWWWWWWWWWWWWWWDO",
    "OHWWWWWWWWWWWWWWWWWWWWWWDO",
    "OHWWWWWWWWWWWWWWWWWWWWWWDO",
    "OHWWWWWWWWWWWWWWWWWWWWWWDO",
    "OHWWWWWWWWWWWWWWWWWWWWWWDO",
    "OHWWWWWWWWWWWWWWWWWWWWWWDO",
    "OHWWWWWWWWWWWWWWWWWWWWWWDO",
    "OHWWWWWWWWWWWWWWWWWWWWWWDO",
    "OHWWWWWWWWWWWWWWWWWWWWWWDO",
    "OHWWWWWWWWWWWWWWWWWWWWWWDO",
    "OHWWWWWWWWWWWWWWWWWWWWWWDO",
    "OHWWWWWWWWWWWWWWWWWWWWWWDO",
    "OHWWWWWWWWWWWWWWWWWWWWWWDO",
    "OHWWWWWWWWWWWWWWWWWWWWWWDO",
    "OHWWWWWWWWWWWWWWWWWWWWWWDO",
    "OHWWWWWWWWWWWWWWWWWWWWWWDO",
    "OHWWWWWWWWWWWWWWWWWWWWWWDO",
    "ODDDDDDDDDDDDDDDDDDDDDDDDO",
    ".ODDDDDDDDDDDDDDDDDDDDDDO.",
    "..OOOOOOOOOOOOOOOOOOOOOO..",
];

pub const FLAG_PIECE: [&str; 22] = [
    "........OOOOOOOOOOOOOOOOOOOOOOOO........",
    ".......OHHHHHHHHHHHHHHHHHHHHHHHHO.......",
    "......OHFFFFFFFFFFFFFFFFFFFFFFFFDO......",
    ".....OHFFFFFFFFFFFFFFFFFFFFFFFFFFDO.....",
    "....OHFFFFPPPFFFFFFFFFFFFFFPPPFFFFDO....",
    "...OHFFFFFFFPFFFFFFFFFFFFFFPFFFFFFFDO...",
    "..OHFFFFFFPPPFFFFFFFFFFFFFFPPPFFFFFFDO..",
    ".OHFFFFFFFPFFFFFFFFFFFFFFFFFFPFFFFFFFDO.",
    "OHFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFDO",
    "OHFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFDO",
    "OHFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFDO",
    "OHFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFDO",
    "OHFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFDO",
    "OHFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFDO",
    "OHFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFDO",
    "OHFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFDO",
    "OHFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFDO",
    "OHFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFDO",
    "OHFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFDO",
    "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
    "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
    "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
];

pub const GROOVE_PIECE: [&str; 20] = [
    "ODL.", "ODL.", "ODL.", "ODL.", "ODL.", "ODL.", "ODL.", "ODL.", "ODL.", "ODL.", "ODL.", "ODL.",
    "ODL.", "ODL.", "ODL.", "ODL.", "ODL.", "ODL.", "ODL.", "ODL.",
];

pub type Palette = &'static [(char, [u8; 3])];

pub const KEY_REST: Palette = &[
    ('O', [0x16, 0x17, 0x16]),
    ('R', [0x7c, 0x75, 0x65]),
    ('S', [0x1d, 0x1f, 0x1e]),
    ('H', [0x2e, 0x31, 0x2f]),
    ('F', [0x25, 0x27, 0x26]),
    ('K', [0x33, 0x36, 0x34]),
];

pub const KEY_LIT: Palette = &[
    ('O', [0x16, 0x17, 0x16]),
    ('R', [0xb4, 0xae, 0x98]),
    ('S', [0x26, 0x28, 0x27]),
    ('H', [0x3a, 0x3d, 0x3a]),
    ('F', [0x30, 0x33, 0x31]),
    ('K', [0x3e, 0x42, 0x3f]),
];

pub const KEY_ON: Palette = &[
    ('O', [0x5a, 0x10, 0x12]),
    ('R', [0xe0, 0x46, 0x32]),
    ('S', [0xc0, 0x2c, 0x2a]),
    ('H', [0x7c, 0x58, 0x58]),
    ('F', [0x69, 0x49, 0x4b]),
    ('K', [0x74, 0x52, 0x53]),
];

pub const LED_LIT: Palette = &[
    ('O', [0x0e, 0x3a, 0x12]),
    ('G', [0x45, 0xd4, 0x45]),
    ('H', [0xe8, 0xff, 0xe0]),
    ('D', [0x22, 0x8c, 0x2a]),
];

pub const DOT_OFF: Palette = &[('O', [0x1a, 0x1b, 0x1a]), ('F', [0x2c, 0x2e, 0x2c])];

pub const DOT_ON: Palette = &[('O', [0x5a, 0x44, 0x10]), ('F', [0xf0, 0xb4, 0x3a])];

pub const PLUG_RED: Palette = &[
    ('O', [0x7a, 0x1c, 0x16]),
    ('H', [0xf6, 0x8a, 0x72]),
    ('U', [0xe6, 0x53, 0x40]),
    ('W', [0xdc, 0x46, 0x36]),
    ('D', [0xb0, 0x30, 0x26]),
];

pub const FLAG_REST: Palette = &[
    ('O', [0x2c, 0x31, 0x29]),
    ('H', [0x6a, 0x72, 0x62]),
    ('F', [0x4f, 0x56, 0x49]),
    ('D', [0x3c, 0x42, 0x37]),
    ('P', [0x4a, 0x51, 0x44]),
];

pub const FLAG_ON: Palette = &[
    ('O', [0x6a, 0x16, 0x12]),
    ('H', [0xf0, 0x7c, 0x68]),
    ('F', [0xd8, 0x40, 0x30]),
    ('D', [0xa8, 0x2c, 0x22]),
    ('P', [0xce, 0x3b, 0x2c]),
];

pub const GROOVE_TONE: Palette = &[
    ('O', [0x2e, 0x32, 0x2a]),
    ('D', [0x26, 0x29, 0x23]),
    ('L', [0x5c, 0x63, 0x55]),
];

pub const TAB_INK: [u8; 3] = [0xf4, 0xf2, 0xea];

pub fn paint(rows: &[&str], palette: Palette) -> (Vec<u8>, u32, u32) {
    let width = rows.first().map_or(0, |row| row.chars().count()) as u32;
    let height = rows.len() as u32;
    let mut data = Vec::with_capacity((width * height) as usize * 4);

    for row in rows {
        for glyph in row.chars() {
            match palette.iter().find(|(key, _)| *key == glyph) {
                Some((_, color)) => data.extend_from_slice(&opaque(*color)),
                None => data.extend_from_slice(&CLEAR),
            }
        }
    }

    (data, width, height)
}

pub fn silhouette(frame: &[[u8; 4]], tone: [u8; 3]) -> Vec<[u8; 4]> {
    frame
        .iter()
        .map(|pixel| if pixel[3] == 0 { CLEAR } else { opaque(tone) })
        .collect()
}

pub const BRASS: [u8; 3] = [0xe2, 0xaf, 0x52];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Metal {
    pub face: [u8; 3],
    pub stud: [u8; 3],
}

pub const FRAME: Metal = Metal {
    face: [0x4a, 0x51, 0x45],
    stud: [0x8c, 0x7e, 0x5c],
};

pub fn chrome(rows: &[&str], metal: Metal) -> (Vec<u8>, u32, u32) {
    let width = rows.first().map_or(0, |row| row.chars().count()) as u32;
    let height = rows.len() as u32;
    let seam = shade(metal.face, 0.42);
    let dark = shade(metal.face, 0.74);
    let light = shade(metal.face, 1.14);
    let glint = shade(metal.face, 1.30);
    let mut data = Vec::with_capacity((width * height) as usize * 4);

    for row in rows {
        for glyph in row.chars() {
            let color = match glyph {
                'E' => seam,
                'D' => dark,
                'M' => metal.face,
                'L' => light,
                'H' => glint,
                'B' => metal.stud,
                _ => {
                    data.extend_from_slice(&CLEAR);
                    continue;
                }
            };

            data.extend_from_slice(&opaque(color));
        }
    }

    (data, width, height)
}

fn speckle(x: u32, y: u32) -> u32 {
    let mut h = x.wrapping_mul(0x27d4_eb2d) ^ y.wrapping_mul(0x1656_67b1);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h.wrapping_mul(0x297a_2d39)
}

fn nudge(color: [u8; 3], by: i32) -> [u8; 3] {
    color.map(|channel| (i32::from(channel) + by).clamp(0, 255) as u8)
}

pub fn plate(width: u32, height: u32, spec: Plate) -> Vec<u8> {
    let light = shade(spec.face, 1.34);
    let dark = shade(spec.face, 0.68);
    let (upper, lower) = if spec.inset {
        (dark, light)
    } else {
        (light, dark)
    };
    let mut data = Vec::with_capacity((width * height) as usize * 4);

    for y in 0..height {
        for x in 0..width {
            let rim = x == 0 || y == 0 || x == width - 1 || y == height - 1;
            let bevel = x == 1 || y == 1 || x == width - 2 || y == height - 2;

            let color = if rim {
                spec.edge
            } else if bevel {
                if x == 1 || y == 1 { upper } else { lower }
            } else if spec.grain > 0 {
                let noise = speckle(x, y);
                let span = spec.grain * 2 + 1;
                nudge(spec.face, (noise % span as u32) as i32 - spec.grain)
            } else {
                spec.face
            };

            data.extend_from_slice(&[color[0], color[1], color[2], 255]);
        }
    }

    data
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff_u32;

    for byte in data {
        crc ^= u32::from(*byte);

        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }

    !crc
}

fn adler32(data: &[u8]) -> u32 {
    let (mut low, mut high) = (1_u32, 0_u32);

    for byte in data {
        low = (low + u32::from(*byte)) % 65521;
        high = (high + low) % 65521;
    }

    (high << 16) | low
}

fn chunk(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
    let mut tagged = kind.to_vec();
    tagged.extend_from_slice(body);

    let mut out = (body.len() as u32).to_be_bytes().to_vec();
    out.extend_from_slice(&tagged);
    out.extend_from_slice(&crc32(&tagged).to_be_bytes());
    out
}

fn deflate_stored(raw: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01];
    let mut blocks = raw.chunks(65_535).peekable();

    if blocks.peek().is_none() {
        out.extend_from_slice(&[1, 0, 0, 0xff, 0xff]);
    }

    while let Some(block) = blocks.next() {
        let len = block.len() as u16;
        out.push(u8::from(blocks.peek().is_none()));
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(block);
    }

    out.extend_from_slice(&adler32(raw).to_be_bytes());
    out
}

pub fn png(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    let stride = width as usize * 4;
    let mut raw = Vec::with_capacity((stride + 1) * height as usize);

    for y in 0..height as usize {
        raw.push(0);
        raw.extend_from_slice(&rgba[y * stride..(y + 1) * stride]);
    }

    let mut header = width.to_be_bytes().to_vec();
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 6, 0, 0, 0]);

    let mut out = vec![137, 80, 78, 71, 13, 10, 26, 10];
    out.extend_from_slice(&chunk(b"IHDR", &header));
    out.extend_from_slice(&chunk(b"IDAT", &deflate_stored(&raw)));
    out.extend_from_slice(&chunk(b"IEND", &[]));
    out
}

pub fn flatten(frame: &[[u8; 4]]) -> Vec<u8> {
    frame.iter().flatten().copied().collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Block,
    Droplet,
    Sprout,
    Raise,
    Lower,
    Lens,
    Eraser,
    Undo,
    Play,
    Pause,
    Hourglass,
    Round,
    Square,
    Minus,
    Plus,
    Bucket,
    Peaks,
    Sprig,
    Spade,
    Paw,
    Chevron,
    Sponge,
    Wave,
    Snowflake,
    Seed,
    Flower,
    Cloud,
    Sun,
    Mound,
    Pit,
    Level,
    Trowel,
    Crack,
    Bolt,
    Flame,
    Flask,
    Bomb,
    Meteor,
    Volcano,
    Heart,
    Skull,
    Halo,
}

const BLOCK: [&str; SPRITE] = [
    "................",
    "................",
    "................",
    "....LLLLLLLL....",
    "...LLLLLLLLLL...",
    "..LLLLLLLLLLLL..",
    "..CCCCCCCCCCCC..",
    "..CCCCCCCCCCCC..",
    "..CCCCCCCCCCCC..",
    "..DDDDDDDDDDDD..",
    "..DDDDDDDDDDDD..",
    "...DDDDDDDDDD...",
    "................",
    "................",
    "................",
    "................",
];

const DROPLET: [&str; SPRITE] = [
    "................",
    "................",
    "......LLLL......",
    "....LLWWLLLL....",
    "...LLWWLLLLLL...",
    "..LLWWLLLLLLCC..",
    "..LLLLLLLLLLCC..",
    "..LLLLLLLLLLCC..",
    "..CLLLLLLLLCCC..",
    "..CCLLLLLLCCCC..",
    "...CCCCCCCCCC...",
    "....CCCCCCCC....",
    "......CCCC......",
    "................",
    "................",
    "................",
];

const SPROUT: [&str; SPRITE] = [
    "................",
    "................",
    ".....C....C.....",
    "....CC.CC.CC....",
    "...CC.CCC..CC...",
    "..C.CCCCCCC.C...",
    "...LLLLLLLLLL...",
    "..LLLLLLLLLLLL..",
    "..AAAAAAAAAAAA..",
    "..AAAAAAAAAAAA..",
    "..BBBBBBBBBBBB..",
    "...BBBBBBBBBB...",
    "................",
    "................",
    "................",
    "................",
];

const RAISE: [&str; SPRITE] = [
    "................",
    ".......CC.......",
    "......CCCC......",
    ".....CCCCCC.....",
    "....CC....CC....",
    ".......CC.......",
    ".......CC.......",
    "................",
    "...LLLLLLLLLL...",
    "..LLLLLLLLLLLL..",
    "..AAAAAAAAAAAA..",
    "..AAAAAAAAAAAA..",
    "..BBBBBBBBBBBB..",
    "...BBBBBBBBBB...",
    "................",
    "................",
];

const LOWER: [&str; SPRITE] = [
    "................",
    ".......CC.......",
    ".......CC.......",
    "....CC....CC....",
    ".....CCCCCC.....",
    "......CCCC......",
    ".......CC.......",
    "................",
    "...LLLLLLLLLL...",
    "..LLLLLLLLLLLL..",
    "..AAAAAAAAAAAA..",
    "..AAAAAAAAAAAA..",
    "..BBBBBBBBBBBB..",
    "...BBBBBBBBBB...",
    "................",
    "................",
];

const LENS: [&str; SPRITE] = [
    "................",
    "....EEEEEE......",
    "...EWWWWWWE.....",
    "..EWWCCCCWWE....",
    "..EWCCCCCCWE....",
    "..EWCCCCCCWE....",
    "..EWWCCCCWWE....",
    "...EWWWWWWE.....",
    "....EEEEEEE.....",
    ".........EEE....",
    "..........EEE...",
    "...........EEE..",
    "............EE..",
    "................",
    "................",
    "................",
];

const ERASER: [&str; SPRITE] = [
    "................",
    "...........EE...",
    "..........ECCE..",
    ".........ECCCCE.",
    "........ECCCCE..",
    ".......ECCCCE...",
    "......ECCCCE....",
    ".....EWWWWE.....",
    "....EWWWWE......",
    "...EWWWWE.......",
    "..EWWWWE........",
    "..EWWWE.........",
    "..EEEE..........",
    "................",
    "................",
    "................",
];

const UNDO: [&str; SPRITE] = [
    "................",
    "................",
    "................",
    "....CC..........",
    "...CCC..........",
    "..CCCCCCCCCC....",
    ".CCCCCCCCCCCC...",
    "CCCCCCCCCCCCCC..",
    ".CCCCC......CCC.",
    "..CCCC.......CC.",
    "...CCC.......CC.",
    "....CC.......CC.",
    ".............CC.",
    "..........CCCC..",
    "................",
    "................",
];

const PLAY: [&str; SPRITE] = [
    "................",
    "................",
    "....CC..........",
    "....CCCC........",
    "....CCCCCC......",
    "....CCCCCCCC....",
    "....CCCCCCCCCC..",
    "....CCCCCCCCCC..",
    "....CCCCCCCC....",
    "....CCCCCC......",
    "....CCCC........",
    "....CC..........",
    "................",
    "................",
    "................",
    "................",
];

const PAUSE: [&str; SPRITE] = [
    "................",
    "................",
    "...CCC...CCC....",
    "...CCC...CCC....",
    "...CCC...CCC....",
    "...CCC...CCC....",
    "...CCC...CCC....",
    "...CCC...CCC....",
    "...CCC...CCC....",
    "...CCC...CCC....",
    "...CCC...CCC....",
    "...CCC...CCC....",
    "................",
    "................",
    "................",
    "................",
];

const HOURGLASS: [&str; SPRITE] = [
    "................",
    "...AAAAAAAAAA...",
    "....CCCCCCCC....",
    "....CCCCCCCC....",
    ".....CCCCCC.....",
    "......CCCC......",
    ".......CC.......",
    ".......CC.......",
    "......CCCC......",
    ".....CCCCCC.....",
    "....CCCCCCCC....",
    "....CCCCCCCC....",
    "...AAAAAAAAAA...",
    "................",
    "................",
    "................",
];

const ROUND: [&str; SPRITE] = [
    "................",
    "................",
    ".....CCCCCC.....",
    "...CCCCCCCCCC...",
    "..CCC......CCC..",
    "..CC........CC..",
    ".CC..........CC.",
    ".CC..........CC.",
    ".CC..........CC.",
    ".CC..........CC.",
    "..CC........CC..",
    "..CCC......CCC..",
    "...CCCCCCCCCC...",
    ".....CCCCCC.....",
    "................",
    "................",
];

const SQUARE: [&str; SPRITE] = [
    "................",
    "................",
    "..CCCCCCCCCCCC..",
    "..CCCCCCCCCCCC..",
    "..CC........CC..",
    "..CC........CC..",
    "..CC........CC..",
    "..CC........CC..",
    "..CC........CC..",
    "..CC........CC..",
    "..CC........CC..",
    "..CC........CC..",
    "..CCCCCCCCCCCC..",
    "..CCCCCCCCCCCC..",
    "................",
    "................",
];

const MINUS: [&str; SPRITE] = [
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "..CCCCCCCCCCCC..",
    "..CCCCCCCCCCCC..",
    "..CCCCCCCCCCCC..",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
];

const PLUS: [&str; SPRITE] = [
    "................",
    "................",
    "................",
    ".......CC.......",
    ".......CC.......",
    ".......CC.......",
    "..CCCCCCCCCCCC..",
    "..CCCCCCCCCCCC..",
    "..CCCCCCCCCCCC..",
    ".......CC.......",
    ".......CC.......",
    ".......CC.......",
    "................",
    "................",
    "................",
    "................",
];

const BUCKET: [&str; SPRITE] = [
    "................",
    "................",
    "..EEEEEEEEEEEE..",
    "..EAAAAAAAAAAE..",
    "..EAAAAAAAAAAE..",
    "..ECCCCCCCCCCE..",
    "..ECCCCCCCCCCE..",
    "...ECCCCCCCCE...",
    "...ECCCCCCCCE...",
    "....ECCCCCCE....",
    "....ECCCCCCE....",
    ".....ECCCCE.....",
    ".....EEEEEE.....",
    "................",
    "................",
    "................",
];

const SPONGE: [&str; SPRITE] = [
    "................",
    "................",
    ".....LLLLLLLLL..",
    "....LLLLLLLLLD..",
    "...LLLLLLLLLDD..",
    "..CCCCCCCCCDDD..",
    "..CCDCCCCDCDDD..",
    "..CCCCCCCCCDDD..",
    "..CCCCDCCCCDDD..",
    "..CDCCCCCCCDDD..",
    "..CCCCCCDCCDDD..",
    "..CCCDCCCCCDD...",
    "..CCCCCCCCCD....",
    "................",
    "................",
    "................",
];

const WAVE: [&str; SPRITE] = [
    "................",
    "................",
    ".......LLLL.....",
    ".....LLCCCCL....",
    "....LCCCDDCCW...",
    "...LCCDD..WCW...",
    "..LCCD.....W....",
    "..LCCD..........",
    ".LCCCD......W...",
    ".LCCCCD....WLW..",
    ".CCCCCCDDWLLCCW.",
    "WCCCCCCCCCCCCCCW",
    ".DDDDDDDDDDDDDD.",
    "................",
    "................",
    "................",
];

const SNOWFLAKE: [&str; SPRITE] = [
    "................",
    ".......W........",
    ".....L.L.L......",
    "......LLL.......",
    "..L....L....L...",
    "...L...L...L....",
    "....L..L..L.....",
    ".WLLLLLCLLLLLW..",
    "....L..L..L.....",
    "...L...L...L....",
    "..L....L....L...",
    "......LLL.......",
    ".....L.L.L......",
    ".......W........",
    "................",
    "................",
];

const SEED: [&str; SPRITE] = [
    "................",
    ".......A........",
    "......AAA.A.....",
    ".....AA.BAA.....",
    "........B.......",
    "......EEEE......",
    ".......EE.......",
    ".....LCCCCD.....",
    "....LCCCCCCD....",
    "...LCCCCCCCCD...",
    "..LCCCCCCCCCCD..",
    "..LCCCCBBCCCCD..",
    "..CCCCBAABCCCD..",
    "..DCCCCBBCCCDD..",
    "...DDDDDDDDDD...",
    "................",
];

const FLOWER: [&str; SPRITE] = [
    "................",
    ".....LLLL.......",
    "...LLCCCCLL.....",
    "..LCCCWWCCCD....",
    "..CCCWWWWCCD....",
    "..CCCWWWWCCD....",
    "..DCCCWWCCDD....",
    "...DDCCCCDD.....",
    ".....DDAD.......",
    ".......A....AA..",
    "..AA...A...AAB..",
    "..BAA..A..AAB...",
    "...BAA.A.AB.....",
    ".....BAAAB......",
    ".......AB.......",
    "................",
];

const CLOUD: [&str; SPRITE] = [
    "................",
    "................",
    "......LLLL......",
    "....LLLLLLLL....",
    "...LLLLLLLLLLL..",
    ".LLLLLLLLLLLLLL.",
    "LLLLCCCCCCCCCCLL",
    "LCCCCCCCCCCCCCCD",
    ".DCCCCCCCCCCCCD.",
    "..DDDDDDDDDDDD..",
    "................",
    "...A....A....A..",
    "..AA...AA...AA..",
    "..AB...AB...AB..",
    "...B....B....B..",
    "................",
];

const SUN: [&str; SPRITE] = [
    "................",
    ".......AA.......",
    "..A....AA....A..",
    "...A........A...",
    "......LLLL......",
    ".....LCCCCD.....",
    "....LCCCCCCD....",
    "AA..LCCCCCCD..AA",
    "AA..CCCCCCCD..AA",
    "....DCCCCCDD....",
    ".....DCCCDD.....",
    "......DDDD......",
    "...A........A...",
    "..A....AA....A..",
    ".......AA.......",
    "................",
];

const MOUND: [&str; SPRITE] = [
    ".......WW.......",
    "......WWWW......",
    ".....WWWWWW.....",
    "....WWWWWWWW....",
    ".......WW.......",
    ".......WW.......",
    "................",
    "......AAAA......",
    ".....ALLCCB.....",
    "....ALLCCCCB....",
    "...ALCCCCCCDB...",
    "..ALCCCCCCCDDB..",
    ".ALCCCCCCCCDDDB.",
    ".BBBBBBBBBBBBBB.",
    "................",
    "................",
];

const PIT: [&str; SPRITE] = [
    ".......WW.......",
    ".......WW.......",
    ".......WW.......",
    "....WWWWWWWW....",
    ".....WWWWWW.....",
    "......WWWW......",
    ".......WW.......",
    "................",
    "..LLLLLLLLLLLL..",
    ".LLLLEEEEEELLLL.",
    ".CCCEEEEEEEECCC.",
    ".CCCCEEEEEECCCC.",
    ".DCCCCDDDDCCCCD.",
    ".DDDDDDDDDDDDDD.",
    "................",
    "................",
];

const LEVEL: [&str; SPRITE] = [
    "................",
    "................",
    "................",
    "..W..........W..",
    ".WW..........WW.",
    ".EEEEEEEEEEEEEE.",
    ".ELLLLLLLLLLLLE.",
    ".ECCCEEEEEECCCE.",
    ".ECCCEAWAAECCCE.",
    ".ECCCEBBBBECCCE.",
    ".ECCCEEEEEECCCE.",
    ".EDDDDDDDDDDDDE.",
    ".EEEEEEEEEEEEEE.",
    "................",
    "................",
    "................",
];

const TROWEL: [&str; SPRITE] = [
    "................",
    "............AB..",
    "...........AAB..",
    "..........AAB...",
    ".........AAB....",
    "........EB......",
    ".......EE.......",
    "....LLLLLLL.....",
    "...LLCCCCCCD....",
    "..LCCCCCCCCD....",
    ".LCCCCCCCCDD....",
    "LCCCCCCCCDD.....",
    "DDDDDDDDDD......",
    "................",
    "................",
    "................",
];

const CRACK: [&str; SPRITE] = [
    "................",
    "................",
    "..W..........W..",
    "................",
    "..LLLLLELLLLLL..",
    ".LLLLLLEELLLLLL.",
    ".CCCCCCCECCCCCC.",
    ".CCCCCCEECCCCCC.",
    ".CCCCCCECCCCCCC.",
    ".DDDDDDEEDDDDDD.",
    ".DDDDDDDEDDDDDD.",
    "..DDDDDDEEDDDD..",
    "................",
    ".W.W........W.W.",
    "................",
    "................",
];

const BOLT: [&str; SPRITE] = [
    "................",
    "........LLLL....",
    ".......LCCD.....",
    "......LCCD......",
    ".....LCCD.......",
    "....LCCD........",
    "...LCCCCCCCD....",
    "...DDDDLCCD.....",
    "......LCCD......",
    ".....LCCD.......",
    "....LCCD........",
    "...LCD..........",
    "..LD............",
    "..D.............",
    "................",
    "................",
];

const FLAME: [&str; SPRITE] = [
    "................",
    ".......L........",
    "......LC........",
    "......LCC.......",
    ".....LCCC...L...",
    "...L.LCCCD..LC..",
    "...LCLCCCCD.LC..",
    "..LCCCCAACCDCCD.",
    "..LCCCAAAACCCCD.",
    "..LCCAAWAAACCCD.",
    "..CCCAAWWAACCCD.",
    "..DCCAAAAAACCDD.",
    "...DCCAAAACCDD..",
    "....DDCCCCDD....",
    "......DDDD......",
    "................",
];

const FLASK: [&str; SPRITE] = [
    "................",
    "......EEEE......",
    "......EWAE......",
    "......EWAE......",
    "......EWAE......",
    ".....EWAAAE.....",
    "....EWAAAAAE....",
    "...EWLLLLLLLE...",
    "..EWLCCCCCCCCE..",
    "..ECCCLCCCCLCE..",
    "..ECCCCCCLCCCE..",
    "..ECCLCCCCCCDE..",
    "..EDCCCCCCCDDE..",
    "...EDDDDDDDDE...",
    "....EEEEEEEE....",
    "................",
];

const BOMB: [&str; SPRITE] = [
    "...........A.A..",
    "............W...",
    "...........AB.A.",
    "..........B.....",
    ".........B......",
    "......EEEE......",
    "....LLLLCCCC....",
    "...LLWLCCCCCD...",
    "..LLWLCCCCCCCD..",
    "..LLCCCCCCCCDD..",
    "..LCCCCCCCCCDD..",
    "..CCCCCCCCCDDD..",
    "...CCCCCCCDDD...",
    "....DDDDDDDD....",
    "......DDDD......",
    "................",
];

const METEOR: [&str; SPRITE] = [
    "................",
    ".............AA.",
    "...........AAA..",
    ".........AAAB...",
    "........AAAB....",
    "......LAAAB.....",
    ".....LLCAB......",
    "...LLLCCCB......",
    "..LLCCCCCCD.....",
    "..LCCDCCCCD.....",
    "..LCCCCCDCD.....",
    "..CCCDCCCDD.....",
    "...CCCCCDD......",
    "....DDDDD.......",
    "................",
    "................",
];

const VOLCANO: [&str; SPRITE] = [
    "................",
    "......W..W......",
    ".....W..W.......",
    "......AAAA......",
    ".....AAWWAA.....",
    "......LAAC......",
    ".....LCAACD.....",
    "....LCCCACCD....",
    "....LCCCACCDD...",
    "...LCCCCCACCD...",
    "...LCCCCCACCDD..",
    "..LCCCCCCCBCCDD.",
    "..CCCCCCCCCCDDD.",
    ".DDDDDDDDDDDDDD.",
    "................",
    "................",
];

const HEART: [&str; SPRITE] = [
    "................",
    "................",
    "...LLL...LLL....",
    "..LWWCC.LCCCD...",
    ".LWWCCCCCCCCCD..",
    ".LWCCCCCCCCCCD..",
    ".LCCCCCCCCCCCD..",
    "..CCCCCCCCCCD...",
    "...CCCCCCCCD....",
    "....CCCCCCD.....",
    ".....CCCCD......",
    "......CCD.......",
    ".......D........",
    "................",
    "................",
    "................",
];

const SKULL: [&str; SPRITE] = [
    "................",
    ".....LLLLLL.....",
    "...LLCCCCCCLL...",
    "..LCCCCCCCCCCD..",
    "..LCCCCCCCCCCD..",
    "..CCEEECCEEECD..",
    "..CCEEECCEEECD..",
    "..CCEEECCEEECD..",
    "..DCCCCEECCCDD..",
    "...DCCCCCCCCD...",
    "....CECECECD....",
    "....DCDCDCDD....",
    ".....DDDDDD.....",
    "................",
    "................",
    "................",
];

const HALO: [&str; SPRITE] = [
    "................",
    "....LLLLLLLL....",
    "..LLCCCCCCCCLL..",
    ".LC..........CD.",
    "..DDCCCCCCCCDD..",
    "....DDDDDDDD....",
    "................",
    "...A...AA...A...",
    "...A...AA...A...",
    "..AA..AAAA..AA..",
    "..AA..AAAA..AA..",
    ".AAA.AAWWAA.AAA.",
    ".AAA.AAWWAA.AAA.",
    ".BBB.BBBBBB.BBB.",
    "................",
    "................",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ink {
    pub base: [u8; 3],
    pub accent: [u8; 3],
}

pub const STEEL: [u8; 3] = [0xc2, 0xc8, 0xd2];
pub const CYAN: [u8; 3] = [0x0a, 0xc0, 0xeb];
pub const WOOD: [u8; 3] = [0x8a, 0x5c, 0x32];
pub const EARTH: [u8; 3] = [0x7a, 0x5a, 0x3c];
pub const AMBER: [u8; 3] = [0xf0, 0xb4, 0x3a];
pub const RUST: [u8; 3] = [0xd8, 0x52, 0x3c];
pub const BONE: [u8; 3] = [0xef, 0xf2, 0xf6];

impl Icon {
    pub fn template(self) -> &'static [&'static str; SPRITE] {
        match self {
            Self::Block => &BLOCK,
            Self::Droplet => &DROPLET,
            Self::Sprout => &SPROUT,
            Self::Raise => &RAISE,
            Self::Lower => &LOWER,
            Self::Lens => &LENS,
            Self::Eraser => &ERASER,
            Self::Undo => &UNDO,
            Self::Play => &PLAY,
            Self::Pause => &PAUSE,
            Self::Hourglass => &HOURGLASS,
            Self::Round => &ROUND,
            Self::Square => &SQUARE,
            Self::Minus => &MINUS,
            Self::Plus => &PLUS,
            Self::Bucket => &BUCKET,
            Self::Peaks => &PEAKS,
            Self::Sprig => &SPRIG,
            Self::Spade => &SPADE,
            Self::Paw => &PAW,
            Self::Chevron => &CHEVRON,
            Self::Sponge => &SPONGE,
            Self::Wave => &WAVE,
            Self::Snowflake => &SNOWFLAKE,
            Self::Seed => &SEED,
            Self::Flower => &FLOWER,
            Self::Cloud => &CLOUD,
            Self::Sun => &SUN,
            Self::Mound => &MOUND,
            Self::Pit => &PIT,
            Self::Level => &LEVEL,
            Self::Trowel => &TROWEL,
            Self::Crack => &CRACK,
            Self::Bolt => &BOLT,
            Self::Flame => &FLAME,
            Self::Flask => &FLASK,
            Self::Bomb => &BOMB,
            Self::Meteor => &METEOR,
            Self::Volcano => &VOLCANO,
            Self::Heart => &HEART,
            Self::Skull => &SKULL,
            Self::Halo => &HALO,
        }
    }

    pub fn takes_tint(self) -> bool {
        matches!(self, Self::Block | Self::Droplet | Self::Sprout)
    }

    pub fn ink(self) -> Ink {
        match self {
            Self::Block | Self::Droplet => Ink {
                base: EARTH,
                accent: EARTH,
            },
            Self::Sprout | Self::Raise | Self::Lower => Ink {
                base: AMBER,
                accent: EARTH,
            },
            Self::Lens => Ink {
                base: [0x7c, 0xc8, 0xf0],
                accent: STEEL,
            },
            Self::Eraser => Ink {
                base: RUST,
                accent: BONE,
            },
            Self::Undo => Ink {
                base: BONE,
                accent: BONE,
            },
            Self::Play | Self::Pause => Ink {
                base: CYAN,
                accent: CYAN,
            },
            Self::Round | Self::Square | Self::Minus | Self::Plus => Ink {
                base: STEEL,
                accent: STEEL,
            },
            Self::Hourglass => Ink {
                base: AMBER,
                accent: WOOD,
            },
            Self::Bucket => Ink {
                base: [0x55, 0xae, 0xf0],
                accent: STEEL,
            },
            Self::Peaks => Ink {
                base: [0x8e, 0x96, 0x9e],
                accent: [0x6a, 0x8e, 0x4a],
            },
            Self::Sprig => Ink {
                base: [0x6f, 0xb8, 0x4e],
                accent: WOOD,
            },
            Self::Spade => Ink {
                base: STEEL,
                accent: WOOD,
            },
            Self::Paw => Ink {
                base: [0xd8, 0xb0, 0x7c],
                accent: WOOD,
            },
            Self::Chevron => Ink {
                base: AMBER,
                accent: AMBER,
            },
            Self::Sponge => Ink {
                base: [0xf2, 0xc8, 0x3c],
                accent: [0xf2, 0xc8, 0x3c],
            },
            Self::Wave => Ink {
                base: [0x40, 0x9c, 0xe8],
                accent: [0x40, 0x9c, 0xe8],
            },
            Self::Snowflake => Ink {
                base: [0x8c, 0xd4, 0xf8],
                accent: [0x8c, 0xd4, 0xf8],
            },
            Self::Seed => Ink {
                base: [0xc8, 0x96, 0x58],
                accent: [0x6f, 0xb8, 0x4e],
            },
            Self::Flower => Ink {
                base: [0xf0, 0x6c, 0x9c],
                accent: [0x6f, 0xb8, 0x4e],
            },
            Self::Cloud => Ink {
                base: [0xc8, 0xd4, 0xe0],
                accent: [0x55, 0xae, 0xf0],
            },
            Self::Sun => Ink {
                base: [0xf0, 0xa8, 0x30],
                accent: [0xf8, 0xdc, 0x48],
            },
            Self::Mound => Ink {
                base: [0x8e, 0x96, 0x9e],
                accent: [0x6a, 0x9e, 0x4a],
            },
            Self::Pit => Ink {
                base: [0xc2, 0x7a, 0x40],
                accent: [0xc2, 0x7a, 0x40],
            },
            Self::Level => Ink {
                base: [0xf0, 0xb4, 0x3a],
                accent: [0x7c, 0xe0, 0x6a],
            },
            Self::Trowel => Ink {
                base: [0xc2, 0xc8, 0xd2],
                accent: [0x8a, 0x5c, 0x32],
            },
            Self::Crack => Ink {
                base: [0xc2, 0x7a, 0x40],
                accent: [0xc2, 0x7a, 0x40],
            },
            Self::Bolt => Ink {
                base: [0xf8, 0xd0, 0x30],
                accent: [0xf8, 0xd0, 0x30],
            },
            Self::Flame => Ink {
                base: [0xf0, 0x6a, 0x24],
                accent: [0xf8, 0xd0, 0x40],
            },
            Self::Flask => Ink {
                base: [0x8c, 0xe0, 0x3c],
                accent: [0xb8, 0xc4, 0xd0],
            },
            Self::Bomb => Ink {
                base: [0x50, 0x56, 0x64],
                accent: [0xf8, 0xb8, 0x30],
            },
            Self::Meteor => Ink {
                base: [0x9a, 0x6e, 0x52],
                accent: [0xf0, 0x7a, 0x2a],
            },
            Self::Volcano => Ink {
                base: [0x6e, 0x5e, 0x56],
                accent: [0xff, 0xa0, 0x10],
            },
            Self::Heart => Ink {
                base: [0xe8, 0x4a, 0x5a],
                accent: [0xe8, 0x4a, 0x5a],
            },
            Self::Skull => Ink {
                base: [0xde, 0xda, 0xcc],
                accent: [0x9c, 0x5c, 0xd8],
            },
            Self::Halo => Ink {
                base: [0xf8, 0xd0, 0x50],
                accent: [0xff, 0xf0, 0xa0],
            },
        }
    }
}

pub fn icon(kind: Icon, tint: Option<[u8; 3]>) -> Vec<[u8; 4]> {
    let ink = kind.ink();
    let base = match tint {
        Some(tint) if kind.takes_tint() => tint,
        _ => ink.base,
    };

    inked(
        kind.template(),
        Ink {
            base,
            accent: ink.accent,
        },
    )
}

pub fn inked(rows: &[&str; SPRITE], ink: Ink) -> Vec<[u8; 4]> {
    let dark = shade(ink.base, 0.62);
    let light = shade(ink.base, 1.28);
    let accent_dark = shade(ink.accent, 0.62);
    let mut out = vec![CLEAR; PIXELS];

    for (y, row) in rows.iter().enumerate() {
        for (x, glyph) in row.chars().enumerate() {
            out[y * SPRITE + x] = match glyph {
                'C' => opaque(ink.base),
                'D' => opaque(dark),
                'L' => opaque(light),
                'A' => opaque(ink.accent),
                'B' => opaque(accent_dark),
                'W' => opaque(BONE),
                'E' => opaque(EYE),
                _ => CLEAR,
            };
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const STANCES: [Stance; 5] = [
        Stance::Upright,
        Stance::Quadruped,
        Stance::Bird,
        Stance::Squat,
        Stance::Swimmer,
    ];

    fn skin() -> Skin {
        Skin {
            coat: [0xe6, 0xb2, 0x8a],
            accent: [0x40, 0x60, 0x90],
        }
    }

    #[test]
    fn every_template_is_a_square_of_the_sprite_size() {
        for stance in STANCES {
            let rows = stance.template();

            assert_eq!(rows.len(), SPRITE);

            for (y, row) in rows.iter().enumerate() {
                assert_eq!(row.chars().count(), SPRITE, "{stance:?} row {y}");
            }
        }
    }

    #[test]
    fn every_template_uses_only_the_glyphs_the_painter_knows() {
        for stance in STANCES {
            for row in stance.template() {
                for glyph in row.chars() {
                    assert!("CDAE.".contains(glyph), "{stance:?} has {glyph}");
                }
            }
        }
    }

    #[test]
    fn every_creature_has_a_face_and_a_body() {
        for stance in STANCES {
            let flat: String = stance.template().concat();

            assert!(flat.contains('E'), "{stance:?} has no eye");
            assert!(flat.contains('C'), "{stance:?} has no body");
            assert!(flat.contains('D'), "{stance:?} has no shading");
        }
    }

    #[test]
    fn a_sprite_is_one_rgba_per_pixel() {
        assert_eq!(sprite(Stance::Upright, Mark::Plain, skin()).len(), PIXELS);
    }

    #[test]
    fn the_background_of_a_sprite_is_transparent() {
        let drawn = sprite(Stance::Upright, Mark::Plain, skin());

        assert_eq!(drawn[0], CLEAR);
        assert_eq!(drawn[PIXELS - 1], CLEAR);
    }

    #[test]
    fn a_sprite_wears_the_colours_it_was_given() {
        let drawn = sprite(Stance::Upright, Mark::Plain, skin());
        let coat = opaque(skin().coat);
        let accent = opaque(skin().accent);

        assert!(drawn.contains(&coat));
        assert!(drawn.contains(&accent));
        assert!(drawn.contains(&opaque(EYE)));
    }

    #[test]
    fn two_coats_give_two_different_creatures() {
        let one = sprite(Stance::Quadruped, Mark::Plain, skin());
        let other = sprite(
            Stance::Quadruped,
            Mark::Plain,
            Skin {
                coat: [0x20, 0x80, 0x40],
                accent: [0x10, 0x10, 0x10],
            },
        );

        assert_ne!(one, other);
    }

    #[test]
    fn horns_add_pixels_the_plain_creature_does_not_have() {
        let plain = sprite(Stance::Upright, Mark::Plain, skin());
        let horned = sprite(Stance::Upright, Mark::Horns, skin());
        let grew = (0..PIXELS)
            .filter(|at| plain[*at] == CLEAR && horned[*at] != CLEAR)
            .count();

        assert_eq!(grew, 4);
    }

    #[test]
    fn a_mark_that_does_not_fit_a_stance_is_simply_not_drawn() {
        assert!(Mark::LongEars.pixels(Stance::Swimmer).is_empty());
        assert_eq!(
            sprite(Stance::Swimmer, Mark::LongEars, skin()),
            sprite(Stance::Swimmer, Mark::Plain, skin())
        );
    }

    #[test]
    fn an_atlas_lays_the_frames_out_in_one_row() {
        let frames = vec![
            sprite(Stance::Upright, Mark::Plain, skin()),
            sprite(Stance::Bird, Mark::Plain, skin()),
        ];
        let (data, width, height) = atlas(&frames);

        assert_eq!((width, height), (32, 16));
        assert_eq!(data.len(), 32 * 16 * 4);
    }

    const ICONS: [Icon; 42] = [
        Icon::Block,
        Icon::Droplet,
        Icon::Sprout,
        Icon::Raise,
        Icon::Lower,
        Icon::Lens,
        Icon::Eraser,
        Icon::Undo,
        Icon::Play,
        Icon::Pause,
        Icon::Hourglass,
        Icon::Round,
        Icon::Square,
        Icon::Minus,
        Icon::Plus,
        Icon::Bucket,
        Icon::Peaks,
        Icon::Sprig,
        Icon::Spade,
        Icon::Paw,
        Icon::Chevron,
        Icon::Sponge,
        Icon::Wave,
        Icon::Snowflake,
        Icon::Seed,
        Icon::Flower,
        Icon::Cloud,
        Icon::Sun,
        Icon::Mound,
        Icon::Pit,
        Icon::Level,
        Icon::Trowel,
        Icon::Crack,
        Icon::Bolt,
        Icon::Flame,
        Icon::Flask,
        Icon::Bomb,
        Icon::Meteor,
        Icon::Volcano,
        Icon::Heart,
        Icon::Skull,
        Icon::Halo,
    ];

    #[test]
    fn every_icon_is_a_square_of_the_sprite_size() {
        for kind in ICONS {
            let rows = kind.template();

            assert_eq!(rows.len(), SPRITE, "{kind:?}");

            for (y, row) in rows.iter().enumerate() {
                assert_eq!(row.chars().count(), SPRITE, "{kind:?} row {y}");
            }
        }
    }

    #[test]
    fn every_icon_uses_only_the_glyphs_the_painter_knows() {
        for kind in ICONS {
            for row in kind.template() {
                for glyph in row.chars() {
                    assert!("CDLABWE.".contains(glyph), "{kind:?} has {glyph}");
                }
            }
        }
    }

    #[test]
    fn every_icon_actually_draws_something() {
        for kind in ICONS {
            let drawn = icon(kind, None);
            let inked = drawn.iter().filter(|pixel| **pixel != CLEAR).count();

            assert!(inked > 20, "{kind:?} drew only {inked} pixels");
        }
    }

    #[test]
    fn a_tinted_icon_takes_the_colour_it_is_given() {
        let sand = [0xf7, 0xe8, 0x98];
        let drawn = icon(Icon::Block, Some(sand));

        assert!(drawn.contains(&opaque(sand)));
        assert_ne!(drawn, icon(Icon::Block, None));
    }

    #[test]
    fn an_icon_that_takes_no_tint_ignores_one() {
        assert!(!Icon::Undo.takes_tint());
        assert_eq!(icon(Icon::Undo, Some([0, 255, 0])), icon(Icon::Undo, None));
    }

    #[test]
    fn the_two_chevrons_point_opposite_ways() {
        assert_ne!(icon(Icon::Raise, None), icon(Icon::Lower, None));
        assert_ne!(icon(Icon::Round, None), icon(Icon::Square, None));
        assert_ne!(icon(Icon::Play, None), icon(Icon::Pause, None));
    }

    const CHROME_GLYPHS: &str = "EDMLHB.";

    fn well_formed(rows: &[&str], width: usize, height: usize) {
        assert_eq!(rows.len(), height);

        for (at, row) in rows.iter().enumerate() {
            assert_eq!(row.chars().count(), width, "row {at}");

            for glyph in row.chars() {
                assert!(CHROME_GLYPHS.contains(glyph), "row {at} has {glyph}");
            }
        }
    }

    fn luma(color: [u8; 3]) -> u32 {
        color.iter().map(|channel| u32::from(*channel)).sum()
    }

    fn tone(palette: Palette, glyph: char) -> [u8; 3] {
        palette
            .iter()
            .find(|(key, _)| *key == glyph)
            .map(|(_, color)| *color)
            .expect("glyph in palette")
    }

    #[test]
    fn every_palette_piece_is_a_clean_rectangle_its_palettes_can_paint() {
        let pieces: [(&str, &[&str], usize, usize, &[Palette]); 6] = [
            ("key", &KEY_PIECE, 24, 24, &[KEY_REST, KEY_LIT, KEY_ON]),
            ("led", &LED_PIECE, 6, 6, &[LED_LIT]),
            ("dot", &DOT_PIECE, 5, 5, &[DOT_OFF, DOT_ON]),
            ("plug", &PLUG_PIECE, 26, 50, &[PLUG_RED]),
            ("flag", &FLAG_PIECE, 40, 22, &[FLAG_REST, FLAG_ON]),
            ("groove", &GROOVE_PIECE, 4, 20, &[GROOVE_TONE]),
        ];

        for (name, rows, width, height, palettes) in pieces {
            assert_eq!(rows.len(), height, "{name} height");

            for (at, row) in rows.iter().enumerate() {
                assert_eq!(row.chars().count(), width, "{name} row {at}");

                for glyph in row.chars().filter(|glyph| *glyph != '.') {
                    for palette in palettes {
                        assert!(
                            palette.iter().any(|(key, _)| *key == glyph),
                            "{name} uses {glyph} that a palette cannot paint"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn paint_leaves_unmapped_glyphs_transparent() {
        let (data, width, height) = paint(&["F.", "?F"], KEY_REST);

        assert_eq!((width, height), (2, 2));
        assert_eq!(data[3], 255);
        assert_eq!(&data[4..8], &CLEAR);
        assert_eq!(&data[8..12], &CLEAR);
    }

    #[test]
    fn a_key_has_rounded_corners() {
        let (data, width, height) = paint(&KEY_PIECE, KEY_REST);
        let alpha = |x: u32, y: u32| data[((y * width + x) * 4 + 3) as usize];

        for (x, y) in [
            (0, 0),
            (width - 1, 0),
            (0, height - 1),
            (width - 1, height - 1),
        ] {
            assert_eq!(alpha(x, y), 0, "corner {x},{y} is filled");
        }
        assert_eq!(alpha(width / 2, 0), 255);
    }

    #[test]
    fn a_key_rim_catches_the_eye_against_its_face() {
        for palette in [KEY_REST, KEY_LIT, KEY_ON] {
            assert!(luma(tone(palette, 'R')) > luma(tone(palette, 'F')) + 60);
        }
    }

    #[test]
    fn a_hovered_key_is_brighter_than_a_resting_one() {
        assert!(luma(tone(KEY_LIT, 'R')) > luma(tone(KEY_REST, 'R')));
        assert!(luma(tone(KEY_LIT, 'F')) > luma(tone(KEY_REST, 'F')));
    }

    #[test]
    fn a_chosen_key_is_framed_in_red() {
        let [r, g, b] = tone(KEY_ON, 'R');

        assert!(r > g + 100 && r > b + 100, "{r},{g},{b}");
    }

    #[test]
    fn a_resting_key_reads_apart_from_the_panel_behind_it() {
        let apart = luma(FRAME.face).abs_diff(luma(tone(KEY_REST, 'F')));

        assert!(apart > 60, "only {apart} apart");
    }

    #[test]
    fn the_led_is_a_green_gem_with_a_glint() {
        let [r, g, b] = tone(LED_LIT, 'G');

        assert!(g > r + 100 && g > b + 100);
        assert!(luma(tone(LED_LIT, 'H')) > luma(tone(LED_LIT, 'G')));
    }

    #[test]
    fn a_lit_dot_outshines_an_unlit_one() {
        assert!(luma(tone(DOT_ON, 'F')) > luma(tone(DOT_OFF, 'F')) + 150);
    }

    #[test]
    fn the_undo_plug_is_lighter_above_its_waist_than_below() {
        assert!(luma(tone(PLUG_RED, 'U')) > luma(tone(PLUG_RED, 'W')));
        assert!(PLUG_PIECE[10].contains('U'));
        assert!(PLUG_PIECE[40].contains('W'));
    }

    #[test]
    fn a_chosen_flag_is_red_and_a_resting_one_is_not() {
        let [r, g, b] = tone(FLAG_ON, 'F');
        assert!(r > g + 100 && r > b + 100);

        let [r, g, b] = tone(FLAG_REST, 'F');
        assert!(r.abs_diff(g) < 20 && g.abs_diff(b) < 20);
    }

    #[test]
    fn a_silhouette_keeps_the_shape_in_one_tone() {
        let frame = icon(Icon::Peaks, None);
        let flat = silhouette(&frame, TAB_INK);

        assert_eq!(flat.len(), frame.len());
        for (before, after) in frame.iter().zip(&flat) {
            if before[3] == 0 {
                assert_eq!(*after, CLEAR);
            } else {
                assert_eq!(*after, opaque(TAB_INK));
            }
        }
    }

    #[test]
    fn the_time_buttons_are_drawn_in_cyan() {
        let drawn = icon(Icon::Pause, None);

        assert!(drawn.contains(&opaque(CYAN)));
        assert!(icon(Icon::Play, None).contains(&opaque(CYAN)));
    }

    #[test]
    fn every_chrome_piece_is_a_clean_rectangle_of_legal_glyphs() {
        well_formed(&RAIL_PIECE, 32, 24);
        well_formed(&CAP_PIECE, 14, 24);
    }

    #[test]
    fn the_stretching_middle_of_the_rail_tiles_without_a_seam() {
        let band = &RAIL_PIECE[10..17];

        assert!(
            band.windows(2).all(|pair| pair[0] == pair[1]),
            "rows differ"
        );

        for row in band {
            let middle: Vec<char> = row.chars().skip(10).take(12).collect();
            assert!(middle.windows(2).all(|pair| pair[0] == pair[1]), "{row}");
        }
    }

    #[test]
    fn a_tab_has_sloping_shoulders_that_widen_towards_the_panel() {
        let ink = |row: &str| row.chars().filter(|glyph| *glyph != '.').count();

        assert!(ink(FLAG_PIECE[0]) < ink(FLAG_PIECE[4]));
        assert!(ink(FLAG_PIECE[4]) < ink(FLAG_PIECE[8]));
        assert_eq!(ink(FLAG_PIECE[8]), 40);
    }

    #[test]
    fn the_foot_of_a_tab_is_bare_so_it_merges_into_the_panel() {
        for row in &FLAG_PIECE[19..22] {
            assert!(row.chars().all(|glyph| glyph == 'F'), "{row}");
        }
    }

    #[test]
    fn the_rail_and_the_end_cap_carry_studs() {
        assert!(RAIL_PIECE.iter().any(|row| row.contains('B')));
        assert!(CAP_PIECE.iter().any(|row| row.contains('B')));
    }

    #[test]
    fn the_end_cap_carries_a_weave_band_at_each_end() {
        assert_eq!(CAP_PIECE[5], CAP_PIECE[18]);
        assert_eq!(CAP_PIECE[6], CAP_PIECE[17]);
        assert_eq!(CAP_PIECE[7], CAP_PIECE[16]);
        assert!(CAP_PIECE[6].contains('H'));
    }

    #[test]
    fn the_stretching_middle_of_the_end_cap_tiles_without_a_seam() {
        let band = &CAP_PIECE[9..15];

        assert!(band.windows(2).all(|pair| pair[0] == pair[1]));

        for row in band {
            let middle: Vec<char> = row.chars().skip(5).take(4).collect();
            assert!(middle.windows(2).all(|pair| pair[0] == pair[1]), "{row}");
        }
    }

    #[test]
    fn a_chrome_piece_is_one_rgba_per_pixel() {
        let rows = ["EEEE", "ELME", "EDME", "EEEE"];
        let (data, width, height) = chrome(&rows, FRAME);

        assert_eq!((width, height), (4, 4));
        assert_eq!(data.len(), 4 * 4 * 4);
    }

    #[test]
    fn a_transparent_glyph_stays_transparent() {
        let rows = ["..", "MM"];
        let (data, _, _) = chrome(&rows, FRAME);

        assert_eq!(&data[0..4], &CLEAR);
        assert_eq!(data[8 + 3], 255);
    }

    #[test]
    fn the_six_metal_shades_run_from_the_seam_to_the_glint() {
        let rows = ["EDMLHB"];
        let (data, _, _) = chrome(&rows, FRAME);
        let value = |at: usize| {
            f32::from(data[at * 4]) + f32::from(data[at * 4 + 1]) + f32::from(data[at * 4 + 2])
        };

        assert!(value(0) < value(1), "seam darker than shade");
        assert!(value(1) < value(2), "shade darker than face");
        assert!(value(2) < value(3), "face darker than light");
        assert!(value(3) < value(4), "light darker than glint");
    }

    #[test]
    fn a_stud_is_the_accent_rather_than_the_face() {
        let (data, _, _) = chrome(&["B"], FRAME);

        assert_eq!(&data[0..3], &FRAME.stud);
    }

    #[test]
    fn the_same_piece_recoloured_gives_a_different_metal() {
        let rows = ["EMLH"];

        let ember = Metal {
            face: [0xc8, 0x53, 0x42],
            stud: BRASS,
        };

        assert_ne!(chrome(&rows, FRAME).0, chrome(&rows, ember).0);
    }

    #[test]
    fn a_plate_is_one_rgba_per_pixel() {
        assert_eq!(plate(24, 16, PANEL).len(), 24 * 16 * 4);
    }

    #[test]
    fn a_plate_is_ringed_by_its_edge_colour() {
        let (w, h) = (12_u32, 10_u32);
        let data = plate(w, h, PANEL);
        let at = |x: u32, y: u32| {
            let start = (y * w + x) as usize * 4;
            [data[start], data[start + 1], data[start + 2]]
        };

        for x in 0..w {
            assert_eq!(at(x, 0), PANEL.edge, "top {x}");
            assert_eq!(at(x, h - 1), PANEL.edge, "bottom {x}");
        }
        for y in 0..h {
            assert_eq!(at(0, y), PANEL.edge, "left {y}");
            assert_eq!(at(w - 1, y), PANEL.edge, "right {y}");
        }
    }

    #[test]
    fn a_raised_plate_is_lit_from_the_top_left() {
        let (w, h) = (12_u32, 10_u32);
        let data = plate(w, h, PANEL);
        let at = |x: u32, y: u32| {
            let start = (y * w + x) as usize * 4;
            f32::from(data[start]) + f32::from(data[start + 1]) + f32::from(data[start + 2])
        };

        assert!(
            at(5, 1) > at(5, h - 2),
            "top should be brighter than bottom"
        );
        assert!(
            at(1, 5) > at(w - 2, 5),
            "left should be brighter than right"
        );
    }

    #[test]
    fn an_inset_plate_is_lit_the_other_way_round() {
        let (w, h) = (12_u32, 10_u32);
        let data = plate(w, h, SLOT);
        let at = |x: u32, y: u32| {
            let start = (y * w + x) as usize * 4;
            f32::from(data[start]) + f32::from(data[start + 1]) + f32::from(data[start + 2])
        };

        assert!(at(5, 1) < at(5, h - 2), "a hole is dark at the top");
        assert!(at(1, 5) < at(w - 2, 5), "a hole is dark on the left");
    }

    #[test]
    fn the_face_of_a_plate_is_grainy_rather_than_flat() {
        let (w, h) = (24_u32, 24_u32);
        let data = plate(w, h, SLOT);
        let mut shades: Vec<[u8; 3]> = Vec::new();

        for y in 3..h - 3 {
            for x in 3..w - 3 {
                let at = (y * w + x) as usize * 4;
                shades.push([data[at], data[at + 1], data[at + 2]]);
            }
        }

        shades.sort_unstable();
        shades.dedup();

        assert!(shades.len() > 6, "only {} shades", shades.len());
    }

    #[test]
    fn a_plate_is_fully_opaque() {
        let data = plate(8, 8, TAB);

        assert!(data.chunks(4).all(|pixel| pixel[3] == 255));
    }

    #[test]
    fn the_active_tab_is_a_different_colour_from_a_resting_one() {
        assert_ne!(TAB.face, TAB_ON.face);
        assert_ne!(plate(10, 10, TAB), plate(10, 10, TAB_ON));
    }

    #[test]
    fn a_png_starts_with_the_signature_every_reader_looks_for() {
        let file = png(1, 1, &[255, 0, 0, 255]);

        assert_eq!(&file[..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
        assert_eq!(&file[12..16], b"IHDR");
        assert_eq!(&file[file.len() - 8..file.len() - 4], b"IEND");
    }

    #[test]
    fn a_png_header_carries_the_size_and_says_it_is_rgba() {
        let file = png(16, 9, &[0; 16 * 9 * 4]);

        assert_eq!(&file[16..20], &16_u32.to_be_bytes());
        assert_eq!(&file[20..24], &9_u32.to_be_bytes());
        assert_eq!(file[24], 8);
        assert_eq!(file[25], 6);
    }

    #[test]
    fn every_chunk_carries_a_checksum_that_matches_its_body() {
        let file = png(4, 4, &[7; 4 * 4 * 4]);
        let mut at = 8;

        while at < file.len() {
            let len = u32::from_be_bytes(file[at..at + 4].try_into().expect("length")) as usize;
            let tagged = &file[at + 4..at + 8 + len];
            let stated =
                u32::from_be_bytes(file[at + 8 + len..at + 12 + len].try_into().expect("crc"));

            assert_eq!(crc32(tagged), stated);
            at += 12 + len;
        }

        assert_eq!(at, file.len());
    }

    #[test]
    fn the_pixels_survive_the_trip_through_the_deflate_stream() {
        let rgba: Vec<u8> = (0..(8 * 3 * 4)).map(|at| (at % 251) as u8).collect();
        let file = png(8, 3, &rgba);
        let start = file
            .windows(4)
            .position(|window| window == b"IDAT")
            .expect("an idat")
            + 4;
        let stream = &file[start + 2..];
        let mut raw = Vec::new();
        let mut at = 0;

        loop {
            let last = stream[at] & 1 == 1;
            let len = u16::from_le_bytes([stream[at + 1], stream[at + 2]]) as usize;
            raw.extend_from_slice(&stream[at + 5..at + 5 + len]);
            at += 5 + len;

            if last {
                break;
            }
        }

        for y in 0..3 {
            let line = &raw[y * (8 * 4 + 1)..(y + 1) * (8 * 4 + 1)];

            assert_eq!(line[0], 0);
            assert_eq!(&line[1..], &rgba[y * 8 * 4..(y + 1) * 8 * 4]);
        }
    }

    #[test]
    fn a_frame_flattens_into_four_bytes_a_pixel() {
        let frame = sprite(Stance::Bird, Mark::Plain, skin());

        assert_eq!(flatten(&frame).len(), PIXELS * 4);
    }

    #[test]
    fn each_frame_keeps_its_own_slot_in_the_atlas() {
        let one = sprite(Stance::Upright, Mark::Plain, skin());
        let other = sprite(
            Stance::Upright,
            Mark::Plain,
            Skin {
                coat: [1, 2, 3],
                accent: [4, 5, 6],
            },
        );
        let (data, width, _) = atlas(&[one.clone(), other.clone()]);
        let row = 4_usize;

        for x in 0..SPRITE {
            let left = (row * width as usize + x) * 4;
            let right = (row * width as usize + SPRITE + x) * 4;

            assert_eq!(&data[left..left + 4], &one[row * SPRITE + x]);
            assert_eq!(&data[right..right + 4], &other[row * SPRITE + x]);
        }
    }
}
