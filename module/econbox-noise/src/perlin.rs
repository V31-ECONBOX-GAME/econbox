const PERMUTATION: [u8; 256] = [
    151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30, 69,
    142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219,
    203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68, 175,
    74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60, 211, 133, 230,
    220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1, 216, 80, 73, 209, 76,
    132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109, 198, 173,
    186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212, 207, 206,
    59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213, 119, 248, 152, 2, 44, 154, 163,
    70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39, 253, 19, 98, 108, 110, 79, 113, 224, 232,
    178, 185, 112, 104, 218, 246, 97, 228, 251, 34, 242, 193, 238, 210, 144, 12, 191, 179, 162,
    241, 81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31, 181, 199, 106, 157, 184, 84, 204,
    176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93, 222, 114, 67, 29, 24, 72, 243, 141,
    128, 195, 78, 66, 215, 61, 156, 180,
];

const LOWEST: f32 = -0.69;
const HIGHEST: f32 = 0.793;

pub fn perlin(x: f32, y: f32) -> f32 {
    (improved(x, y) - LOWEST) / (HIGHEST - LOWEST)
}

fn hash(index: usize) -> usize {
    usize::from(PERMUTATION[index & 255])
}

fn improved(x: f32, y: f32) -> f32 {
    let floor_x = x.floor();
    let floor_y = y.floor();
    let cell_x = (floor_x as i32 & 255) as usize;
    let cell_y = (floor_y as i32 & 255) as usize;
    let x = x - floor_x;
    let y = y - floor_y;
    let u = fade(x);
    let v = fade(y);

    let a = hash(cell_x) + cell_y;
    let b = hash(cell_x + 1) + cell_y;
    let aa = hash(a);
    let ab = hash(a + 1);
    let ba = hash(b);
    let bb = hash(b + 1);

    lerp(
        v,
        lerp(u, grad(hash(aa), x, y), grad(hash(ba), x - 1.0, y)),
        lerp(
            u,
            grad(hash(ab), x, y - 1.0),
            grad(hash(bb), x - 1.0, y - 1.0),
        ),
    )
}

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
}

fn grad(hash: usize, x: f32, y: f32) -> f32 {
    let h = hash & 15;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 {
        y
    } else if h == 12 || h == 14 {
        x
    } else {
        0.0
    };

    (if h & 1 == 0 { u } else { -u }) + (if h & 2 == 0 { v } else { -v })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_permutation_holds_every_byte_once() {
        let mut seen = [false; 256];

        for value in PERMUTATION {
            assert!(!seen[usize::from(value)], "{value} twice");
            seen[usize::from(value)] = true;
        }
    }

    #[test]
    fn whole_coordinates_give_the_value_unity_gives() {
        for (x, y) in [(0.0, 0.0), (3.0, 7.0), (120.0, 45.0)] {
            assert!((perlin(x, y) - 0.465_273_1).abs() < 1e-6, "{x},{y}");
        }
    }

    #[test]
    fn the_noise_stays_near_the_unit_range_and_spreads_over_it() {
        let mut low = f32::MAX;
        let mut high = f32::MIN;

        for step in 0..20_000 {
            let value = perlin(step as f32 * 0.137, step as f32 * 0.0711 + 3.3);
            low = low.min(value);
            high = high.max(value);
        }

        assert!(low > -0.2 && high < 1.1, "{low}..{high}");
        assert!(high - low > 0.7, "{low}..{high}");
    }

    #[test]
    fn the_noise_is_smooth() {
        for step in 0..2000 {
            let x = step as f32 * 0.05;
            let drift = (perlin(x, 1.3) - perlin(x + 0.001, 1.3)).abs();

            assert!(drift < 0.01, "{drift} at {x}");
        }
    }

    #[test]
    fn the_noise_tiles_every_256_cells() {
        assert_eq!(perlin(1.25, 2.5), perlin(257.25, 2.5));
        assert_eq!(perlin(1.25, 2.5), perlin(1.25, 258.5));
    }
}
