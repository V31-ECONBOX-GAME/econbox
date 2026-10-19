use crate::dice::Dice;
use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub const SPRITE_PIXELS: f32 = 5.0;

pub const TONES: [[u8; 3]; 5] = [
    [250, 248, 236],
    [238, 243, 240],
    [218, 233, 240],
    [190, 215, 236],
    [160, 194, 230],
];

const LIGHT: [f32; 3] = [-0.45, -0.85, 0.5];
const SQUASH: f32 = 0.82;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Puff {
    Small,
    Wisp,
    Broad,
    Tall,
    Grand,
}

pub const PUFFS: [Puff; 5] = [
    Puff::Small,
    Puff::Wisp,
    Puff::Broad,
    Puff::Tall,
    Puff::Grand,
];

impl Puff {
    pub fn size(self) -> (u32, u32) {
        match self {
            Self::Small => (35, 15),
            Self::Wisp => (23, 11),
            Self::Broad => (93, 30),
            Self::Tall => (77, 32),
            Self::Grand => (135, 42),
        }
    }

    pub fn big(self) -> bool {
        matches!(self, Self::Broad | Self::Tall | Self::Grand)
    }

    pub fn tiles(self) -> (f32, f32) {
        let (width, height) = self.size();
        (width as f32, height as f32)
    }

    fn seed(self) -> u32 {
        match self {
            Self::Broad => 1,
            Self::Tall => 2,
            Self::Grand => 3,
            Self::Small => 4,
            Self::Wisp => 5,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Lobe {
    x: f32,
    y: f32,
    radius: f32,
    row: usize,
}

struct Row {
    level: f32,
    small: f32,
    large: f32,
    left: f32,
    right: f32,
    spacing: f32,
    jitter: f32,
}

const ROWS: [Row; 3] = [
    Row {
        level: 0.28,
        small: 0.30,
        large: 0.36,
        left: 0.10,
        right: 0.90,
        spacing: 1.5,
        jitter: 0.10,
    },
    Row {
        level: 0.42,
        small: 0.22,
        large: 0.28,
        left: 0.05,
        right: 0.95,
        spacing: 1.1,
        jitter: 0.05,
    },
    Row {
        level: 0.62,
        small: 0.14,
        large: 0.20,
        left: 0.10,
        right: 0.90,
        spacing: 1.0,
        jitter: 0.04,
    },
];

const STRAYS: [f32; 2] = [0.06, 0.94];

fn dome(t: f32) -> f32 {
    (1.0 - (2.0 * t - 1.0).powi(2)).max(0.0)
}

fn lobes(width: f32, height: f32, dice: &mut Dice) -> Vec<Lobe> {
    let least = (height * 0.2).max(1.6);
    let mut out = Vec::new();

    for (index, row) in ROWS.iter().enumerate() {
        let span = (row.right - row.left) * width;
        let typical = (height * (row.small + row.large) * 0.5 * 0.75).max(least);
        let count = ((span / (typical * row.spacing)) as usize).max(2);

        for i in 0..count {
            let t = row.left
                + (row.right - row.left) * (i as f32 + dice.range(0.15, 0.85)) / count as f32;
            let swell = dome(t).sqrt();
            let radius = (height * dice.range(row.small, row.large) * (0.35 + 0.65 * swell))
                .max(least * (0.55 + 0.45 * swell));
            let sag = (1.0 - swell) * 0.4 * (1.0 - index as f32 * 0.4);
            let y = height * (row.level + sag) + dice.range(-row.jitter, row.jitter) * height;

            out.push(Lobe {
                x: t * width,
                y,
                radius,
                row: index,
            });
        }
    }

    for side in STRAYS {
        if dice.chance(0.8) {
            let radius = (height * dice.range(0.08, 0.13)).max(least * 0.8);
            let x = side * width + dice.range(-0.02, 0.02) * width;

            out.push(Lobe {
                x,
                y: height * dice.range(0.55, 0.7),
                radius,
                row: ROWS.len() - 1,
            });
        }
    }

    out
}

fn tone(lit: f32) -> [u8; 3] {
    let index = match lit {
        lit if lit > 0.66 => 0,
        lit if lit > 0.46 => 1,
        lit if lit > 0.26 => 2,
        lit if lit > 0.06 => 3,
        _ => 4,
    };

    TONES[index]
}

pub fn paint(puff: Puff) -> Vec<[u8; 4]> {
    let (width, height) = puff.size();
    let (w, h) = (width as f32, height as f32);
    let mut dice = Dice::new(puff.seed());
    let lobes = lobes(w, h, &mut dice);
    let norm = LIGHT.iter().map(|c| c * c).sum::<f32>().sqrt();
    let light = LIGHT.map(|c| c / norm);
    let front = ROWS.len() - 1;
    let mut pixels = vec![[0; 4]; (width * height) as usize];

    for y in 0..height {
        for x in 0..width {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            let seen = lobes
                .iter()
                .filter_map(|lobe| {
                    let nx = (px - lobe.x) / lobe.radius;
                    let ny = (py - lobe.y) / (lobe.radius * SQUASH);
                    let d2 = nx * nx + ny * ny;

                    (d2 < 1.0).then(|| {
                        let nz = (1.0 - d2).sqrt();
                        let depth = lobe.row as f32 * 10.0 + nz * 2.0 + lobe.y / h;
                        (depth, nx, ny, nz, lobe.row)
                    })
                })
                .max_by(|a, b| a.0.total_cmp(&b.0));

            let Some((_, nx, ny, nz, row)) = seen else {
                continue;
            };

            let lit = nx * light[0] + ny * light[1] + nz * light[2] - py / h * 0.5;
            let alpha = if row == front && ny > 0.75 { 170 } else { 255 };
            let [r, g, b] = tone(lit);

            pixels[(y * width + x) as usize] = [r, g, b, alpha];
        }
    }

    pixels
}

pub fn image(puff: Puff) -> Image {
    let (width, height) = puff.size();
    let data = paint(puff).into_iter().flatten().collect();
    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
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

    fn opaque(puff: Puff) -> Vec<bool> {
        paint(puff).iter().map(|pixel| pixel[3] > 0).collect()
    }

    #[test]
    fn the_sprites_keep_worldbox_proportions_in_five_pixel_art_cells() {
        let worldbox = [(175, 75), (115, 55), (465, 150), (385, 160), (675, 210)];

        for (puff, (width, height)) in PUFFS.into_iter().zip(worldbox) {
            let (w, h) = puff.size();
            assert_eq!(
                (w as f32 * SPRITE_PIXELS, h as f32 * SPRITE_PIXELS),
                (width as f32, height as f32)
            );
        }
    }

    #[test]
    fn every_cloud_is_a_solid_body_wider_than_tall() {
        for puff in PUFFS {
            let (width, height) = puff.size();
            let cover = opaque(puff).iter().filter(|&&on| on).count() as f32;
            let share = cover / (width * height) as f32;

            assert!(share > 0.4 && share < 0.85, "{puff:?} covers {share}");
            assert!(width > height);
        }
    }

    #[test]
    fn the_ends_taper_instead_of_ending_in_a_wall() {
        for puff in PUFFS {
            let (width, height) = puff.size();
            let cells = opaque(puff);
            let column = |x: u32| {
                (0..height)
                    .filter(|&y| cells[(y * width + x) as usize])
                    .count()
            };
            let middle = column(width / 2);

            assert!(column(0) * 2 <= middle, "{puff:?} left edge");
            assert!(column(width - 1) * 2 <= middle, "{puff:?} right edge");
        }
    }

    #[test]
    fn the_top_is_lit_and_the_underside_shaded() {
        for puff in PUFFS {
            let (width, height) = puff.size();
            let pixels = paint(puff);
            let shade = |range: std::ops::Range<u32>| {
                let lit: Vec<u32> = range
                    .flat_map(|y| (0..width).map(move |x| (x, y)))
                    .map(|(x, y)| pixels[(y * width + x) as usize])
                    .filter(|pixel| pixel[3] > 0)
                    .map(|pixel| pixel[0] as u32)
                    .collect();
                lit.iter().sum::<u32>() as f32 / lit.len().max(1) as f32
            };

            assert!(shade(0..height / 2) > shade(height / 2..height), "{puff:?}");
        }
    }

    #[test]
    fn the_underside_thins_out() {
        for puff in PUFFS {
            let (width, height) = puff.size();
            let thin: Vec<u32> = paint(puff)
                .iter()
                .enumerate()
                .filter(|(_, pixel)| pixel[3] > 0 && pixel[3] < 255)
                .map(|(index, _)| index as u32 / width)
                .collect();

            assert!(!thin.is_empty(), "{puff:?}");
            assert!(thin.iter().all(|&y| y >= height / 2), "{puff:?}");
        }
    }
}
