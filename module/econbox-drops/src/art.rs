use crate::kind::DropKind;
use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub const CELL: u32 = 11;

#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<[u8; 4]>,
}

fn paint(rows: &[&str], palette: &[(char, [u8; 3])]) -> Frame {
    let height = rows.len() as u32;
    let width = rows.iter().map(|row| row.len()).max().unwrap_or(0) as u32;
    let pixels = rows
        .iter()
        .flat_map(|row| {
            let mut line: Vec<[u8; 4]> = row
                .chars()
                .map(|cell| {
                    palette
                        .iter()
                        .find(|(key, _)| *key == cell)
                        .map_or([0; 4], |(_, [r, g, b])| [*r, *g, *b, 255])
                })
                .collect();
            line.resize(width as usize, [0; 4]);
            line
        })
        .collect();

    Frame {
        width,
        height,
        pixels,
    }
}

const RAIN: [(char, [u8; 3]); 2] = [('a', [0xb4, 0xe6, 0xff]), ('b', [0x46, 0x8c, 0xf0])];
const ACID: [(char, [u8; 3]); 2] = [('a', [0xa6, 0xf5, 0x6c]), ('b', [0x2a, 0xc0, 0x36])];
const SNOW: [(char, [u8; 3]); 2] = [('a', [0xff, 0xff, 0xff]), ('b', [0xd8, 0xee, 0xf8])];
const ASH: [(char, [u8; 3]); 2] = [('a', [0xcc, 0xb6, 0x86]), ('b', [0x9e, 0x8a, 0x62])];
const MAGIC: [(char, [u8; 3]); 2] = [('a', [0x72, 0xf2, 0xf4]), ('b', [0xc8, 0xff, 0xff])];
const RAGE: [(char, [u8; 3]); 2] = [('a', [0xf4, 0x52, 0x52]), ('b', [0xff, 0xb0, 0xa0])];
const FLAME: [(char, [u8; 3]); 3] = [
    ('a', [0xd8, 0x2a, 0x16]),
    ('b', [0xf8, 0x8c, 0x1a]),
    ('c', [0xff, 0xe2, 0x4c]),
];
const MOLTEN: [[(char, [u8; 3]); 3]; 4] = [
    [
        ('a', [0xff, 0xd2, 0x2e]),
        ('b', [0xf2, 0x7a, 0x12]),
        ('c', [0xc8, 0x2c, 0x10]),
    ],
    [
        ('a', [0xf2, 0x7a, 0x12]),
        ('b', [0xc8, 0x2c, 0x10]),
        ('c', [0xff, 0xd2, 0x2e]),
    ],
    [
        ('a', [0xff, 0xec, 0x6a]),
        ('b', [0xff, 0xa8, 0x22]),
        ('c', [0xe0, 0x46, 0x12]),
    ],
    [
        ('a', [0xff, 0xa8, 0x22]),
        ('b', [0xff, 0xec, 0x6a]),
        ('c', [0xc8, 0x2c, 0x10]),
    ],
];
const SEEDS: [[u8; 3]; 4] = [
    [0xb8, 0xee, 0xff],
    [0xff, 0xdc, 0x52],
    [0xff, 0xb4, 0x92],
    [0xb6, 0xf2, 0x84],
];

fn droplets(palette: &[(char, [u8; 3])]) -> Vec<Frame> {
    vec![
        paint(&[".a.", "aab", "abb", ".b."], palette),
        paint(&[".a.", "aab", ".b."], palette),
        paint(&["a"], palette),
        paint(&["b"], palette),
    ]
}

fn flakes(palette: &[(char, [u8; 3])]) -> Vec<Frame> {
    vec![
        paint(
            &[
                "...a...", ".b.a.b.", "..aba..", "aaabaaa", "..aba..", ".b.a.b.", "...a...",
            ],
            palette,
        ),
        paint(&["..a..", ".aba.", "abbba", ".aba.", "..a.."], palette),
        paint(&["a"], palette),
        paint(&["a.a", ".b.", "a.a"], palette),
    ]
}

fn rings(palette: &[(char, [u8; 3])]) -> Vec<Frame> {
    vec![
        paint(&["aaa", "aba", "aaa"], palette),
        paint(&[".aaa.", "a...a", "a.b.a", "a...a", ".aaa."], palette),
        paint(&[".a.", "a.a", ".a."], palette),
        paint(&[".b.", "bab", ".b."], palette),
    ]
}

fn flame(sway: [i32; 11], tip: usize) -> Frame {
    let widths = [0, 1, 1, 2, 2, 3, 3, 4, 4, 4, 3];
    let mut rows = Vec::new();

    for (y, (&half, &shift)) in widths.iter().zip(sway.iter()).enumerate() {
        let centre = 5 + shift;
        let mut row = vec!['.'; CELL as usize];

        if y >= tip {
            let half = half as i32;
            for x in (centre - half)..=(centre + half) {
                if (0..CELL as i32).contains(&x) {
                    let edge = (x - centre).abs();
                    row[x as usize] = if edge == half {
                        'a'
                    } else if y >= 7 && edge <= 1 {
                        'c'
                    } else {
                        'b'
                    };
                }
            }
        }

        rows.push(row.into_iter().collect::<String>());
    }

    let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
    paint(&rows, &FLAME)
}

pub fn frames(kind: DropKind) -> Vec<Frame> {
    match kind {
        DropKind::Rain => droplets(&RAIN),
        DropKind::Acid => droplets(&ACID),
        DropKind::Snow => flakes(&SNOW),
        DropKind::Ash => flakes(&ASH),
        DropKind::MagicRain => rings(&MAGIC),
        DropKind::Rage => rings(&RAGE),
        DropKind::LifeSeed => SEEDS
            .iter()
            .map(|&seed| {
                let light = seed.map(|c| c.saturating_add(40));
                paint(&[".a.", "aba", ".a."], &[('a', seed), ('b', light)])
            })
            .collect(),
        DropKind::Lava => MOLTEN
            .iter()
            .map(|palette| paint(&[".a.", "abc", "bcb", ".c."], palette))
            .collect(),
        DropKind::Fire => vec![
            flame([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 1),
            flame([1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0], 2),
            flame([-1, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0], 1),
            flame([1, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0], 3),
        ],
    }
}

pub fn image(frame: &Frame) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: frame.width,
            height: frame.height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        frame.pixels.iter().flatten().copied().collect(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        mipmap_filter: ImageFilterMode::Nearest,
        ..Default::default()
    });

    image
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kind::DROPS;

    #[test]
    fn every_drop_has_four_frames_that_fit_a_worldbox_cell() {
        for kind in DROPS {
            let frames = frames(kind);
            assert_eq!(frames.len(), 4, "{kind:?}");

            for frame in &frames {
                assert!(frame.width <= CELL && frame.height <= CELL, "{kind:?}");
                assert_eq!(frame.pixels.len(), (frame.width * frame.height) as usize);
                assert!(frame.pixels.iter().any(|pixel| pixel[3] == 255));
            }
        }
    }

    #[test]
    fn rain_and_snow_keep_worldbox_frame_sizes() {
        let size = |kind| -> Vec<(u32, u32)> {
            frames(kind)
                .iter()
                .map(|frame| (frame.width, frame.height))
                .collect()
        };

        assert_eq!(size(DropKind::Rain), [(3, 4), (3, 3), (1, 1), (1, 1)]);
        assert_eq!(size(DropKind::Snow), [(7, 7), (5, 5), (1, 1), (3, 3)]);
        assert_eq!(size(DropKind::Lava), [(3, 4); 4]);
        assert_eq!(size(DropKind::Fire), [(11, 11); 4]);
    }

    #[test]
    fn a_flame_frame_burns_hottest_at_its_base() {
        let frame = &frames(DropKind::Fire)[0];
        let core = frame.pixels[(9 * frame.width + 5) as usize];
        let tip = frame.pixels[(frame.width + 5) as usize];

        assert_eq!([core[0], core[1], core[2]], [0xff, 0xe2, 0x4c]);
        assert_ne!(tip, core);
    }
}
