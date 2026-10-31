pub const FLAME: u32 = 19;
pub const FLAME_FRAMES: u32 = 6;
pub const FLAME_SETS: u32 = 3;
pub const FLAME_PIVOT: f32 = 0.15;
pub const SPARKLE: u32 = 3;
pub const SPARKLE_FRAMES: u32 = 5;

const EMBER: [u8; 4] = [0xf2, 0x48, 0x3a, 0xff];
const BLAZE: [u8; 4] = [0xfb, 0xa8, 0x32, 0xff];
const CORE: [u8; 4] = [0xfe, 0xf4, 0x62, 0xff];

#[derive(Debug, Clone, Copy, PartialEq)]
struct Shape {
    height: f32,
    width: f32,
    sway: f32,
    lean: f32,
}

const SHAPES: [Shape; FLAME_SETS as usize] = [
    Shape {
        height: 11.0,
        width: 4.2,
        sway: 1.6,
        lean: 0.0,
    },
    Shape {
        height: 11.5,
        width: 3.2,
        sway: 1.2,
        lean: 0.4,
    },
    Shape {
        height: 13.0,
        width: 6.0,
        sway: 1.8,
        lean: -0.3,
    },
];

fn hash(a: u32, b: u32, c: u32) -> u32 {
    let mut h =
        a.wrapping_mul(0x9e37_79b1) ^ b.wrapping_mul(0x85eb_ca77) ^ c.wrapping_mul(0xc2b2_ae3d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^ (h >> 12)
}

pub fn flame(set: u32, frame: u32) -> Vec<[u8; 4]> {
    let shape = SHAPES[(set % FLAME_SETS) as usize];
    let mut pixels = vec![[0_u8; 4]; (FLAME * FLAME) as usize];
    let base = FLAME as f32 - FLAME as f32 * FLAME_PIVOT;
    let phase = frame as f32 / FLAME_FRAMES as f32 * std::f32::consts::TAU;
    let tall = shape.height + (phase * 2.0).sin() * 1.2;

    for row in 0..FLAME {
        let up = base - row as f32 - 0.5;
        if up < -1.0 || up > tall {
            continue;
        }
        let t = (up / tall).clamp(0.0, 1.0);
        let centre =
            FLAME as f32 * 0.5 + shape.sway * t * (phase + t * 3.2).sin() + shape.lean * t * 3.0;
        let half = shape.width * (1.0 - t).powf(0.7) * (0.75 + 0.25 * (t * 3.0).min(1.0));
        for column in 0..FLAME {
            let off = (column as f32 + 0.5 - centre).abs();
            if off > half {
                continue;
            }
            let reach = off / half.max(0.01);
            let heat = (1.0 - t) * (1.0 - reach);
            let color = if heat > 0.42 {
                CORE
            } else if heat > 0.18 {
                BLAZE
            } else {
                EMBER
            };
            if t > 0.75 && hash(column, row, frame + set * 17).is_multiple_of(3) {
                continue;
            }
            pixels[(row * FLAME + column) as usize] = color;
        }
    }

    let spark = hash(set, frame, 91);
    if spark.is_multiple_of(2) {
        let x = (FLAME / 2) as i32 + (spark >> 3) as i32 % 7 - 3;
        let y = (base - tall - 1.5 - ((spark >> 9) % 3) as f32) as i32;
        if (0..FLAME as i32).contains(&x) && (0..FLAME as i32).contains(&y) {
            pixels[(y as u32 * FLAME + x as u32) as usize] = EMBER;
        }
    }
    pixels
}

pub fn flame_atlas() -> (u32, u32, Vec<u8>) {
    let width = FLAME * FLAME_FRAMES;
    let height = FLAME * FLAME_SETS;
    let mut data = vec![0_u8; (width * height * 4) as usize];
    for set in 0..FLAME_SETS {
        for frame in 0..FLAME_FRAMES {
            let pixels = flame(set, frame);
            for row in 0..FLAME {
                for column in 0..FLAME {
                    let pixel = pixels[(row * FLAME + column) as usize];
                    let x = frame * FLAME + column;
                    let y = set * FLAME + row;
                    let at = ((y * width + x) * 4) as usize;
                    data[at..at + 4].copy_from_slice(&pixel);
                }
            }
        }
    }
    (width, height, data)
}

pub fn sparkle(frame: u32) -> [[u8; 3]; 3] {
    match frame {
        0 => [[0, 0, 0], [0, 255, 0], [0, 0, 0]],
        1 => [[0, 255, 0], [255, 255, 255], [0, 255, 0]],
        2 => [[0, 150, 0], [150, 200, 150], [0, 150, 0]],
        3 => [[70, 0, 70], [0, 90, 0], [70, 0, 70]],
        _ => [[0, 0, 0], [0, 40, 0], [0, 0, 0]],
    }
}

pub fn sparkle_atlas() -> (u32, u32, Vec<u8>) {
    let width = SPARKLE * SPARKLE_FRAMES;
    let mut data = vec![0_u8; (width * SPARKLE * 4) as usize];
    for frame in 0..SPARKLE_FRAMES {
        let alpha = sparkle(frame);
        for (row, line) in alpha.iter().enumerate() {
            for (column, &a) in line.iter().enumerate() {
                let x = frame * SPARKLE + column as u32;
                let at = ((row as u32 * width + x) * 4) as usize;
                data[at..at + 4].copy_from_slice(&[255, 255, 255, a]);
            }
        }
    }
    (width, SPARKLE, data)
}

pub fn flame_set(seed: u32) -> u32 {
    if seed.is_multiple_of(6) {
        2
    } else if seed.is_multiple_of(3) {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_flame_frame_burns_from_its_foot_with_three_heats() {
        for set in 0..FLAME_SETS {
            for frame in 0..FLAME_FRAMES {
                let pixels = flame(set, frame);
                let lit: Vec<usize> = (0..pixels.len()).filter(|&i| pixels[i][3] > 0).collect();
                assert!(lit.len() > 20, "{set} {frame}");
                let lowest = lit
                    .iter()
                    .map(|i| i / FLAME as usize)
                    .max()
                    .unwrap_or_default();
                assert!((15..=17).contains(&lowest), "{set} {frame} {lowest}");
                assert!(pixels.contains(&CORE) && pixels.contains(&EMBER));
            }
        }
    }

    #[test]
    fn the_atlases_hold_every_frame() {
        let (w, h, data) = flame_atlas();
        assert_eq!((w, h), (FLAME * 6, FLAME * 3));
        assert_eq!(data.len(), (w * h * 4) as usize);
        let (w, h, data) = sparkle_atlas();
        assert_eq!((w, h), (15, 3));
        assert_eq!(data[(4 * 4) + 3], 255);
    }

    #[test]
    fn one_tile_in_six_uses_the_third_flame_and_one_in_six_the_second() {
        let sets: Vec<u32> = (0..6000).map(flame_set).collect();
        let count = |set| sets.iter().filter(|&&s| s == set).count();
        assert_eq!(count(2), 1000);
        assert_eq!(count(1), 1000);
        assert_eq!(count(0), 4000);
    }
}
