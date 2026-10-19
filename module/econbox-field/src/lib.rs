//! The grids the generator works on: tile heights and alpha masks.

pub const MAX_HEIGHT: i32 = 255;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heights {
    pub width: u32,
    pub height: u32,
    pub values: Vec<u8>,
}

impl Heights {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            values: vec![0; (width * height) as usize],
        }
    }

    pub fn index(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    pub fn get(&self, x: u32, y: u32) -> i32 {
        i32::from(self.values[self.index(x, y)])
    }

    pub fn set(&mut self, x: u32, y: u32, value: i32) {
        let index = self.index(x, y);
        self.values[index] = value.clamp(0, MAX_HEIGHT) as u8;
    }

    pub fn add(&mut self, x: u32, y: u32, by: i32) {
        self.set(x, y, self.get(x, y) + by);
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mask {
    pub width: u32,
    pub height: u32,
    pub alpha: Vec<f32>,
}

impl Mask {
    pub fn from_fn(width: u32, height: u32, alpha: impl Fn(u32, u32) -> f32) -> Self {
        let mut values = Vec::with_capacity((width * height) as usize);

        for y in 0..height {
            for x in 0..width {
                values.push(alpha(x, y).clamp(0.0, 1.0));
            }
        }

        Self {
            width,
            height,
            alpha: values,
        }
    }

    pub fn at(&self, x: u32, y: u32) -> f32 {
        self.alpha[(y * self.width + x) as usize]
    }

    pub fn rotated(&self, degrees: i32) -> Self {
        let phi = std::f64::consts::PI / 180.0 * f64::from(degrees);
        let (sin, cos) = phi.sin_cos();
        let width = self.width as i32;
        let height = self.height as i32;
        let half_x = width / 2;
        let half_y = height / 2;
        let mut alpha = vec![0.0; self.alpha.len()];

        for y in 0..height {
            for x in 0..width {
                let dx = f64::from(x - half_x);
                let dy = f64::from(y - half_y);
                let from_x = (cos * dx + sin * dy + f64::from(half_x)) as i32;
                let from_y = (-sin * dx + cos * dy + f64::from(half_y)) as i32;

                if from_x > -1 && from_x < width && from_y > -1 && from_y < height {
                    alpha[(y * width + x) as usize] =
                        self.alpha[(from_y * width + from_x) as usize];
                }
            }
        }

        Self {
            width: self.width,
            height: self.height,
            alpha,
        }
    }

    pub fn scaled(&self, width: u32, height: u32) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        let ratio_x = 1.0 / (width as f32 / (self.width as f32 - 1.0));
        let ratio_y = 1.0 / (height as f32 / (self.height as f32 - 1.0));
        let last_x = self.width.saturating_sub(1);
        let last_y = self.height.saturating_sub(1);
        let mut alpha = Vec::with_capacity((width * height) as usize);

        for y in 0..height {
            let row = ((y as f32 * ratio_y).floor() as u32).min(last_y);
            let next_row = (row + 1).min(last_y);
            let along_y = y as f32 * ratio_y - row as f32;

            for x in 0..width {
                let column = ((x as f32 * ratio_x).floor() as u32).min(last_x);
                let next_column = (column + 1).min(last_x);
                let along_x = x as f32 * ratio_x - column as f32;
                let bottom = lerp(self.at(column, row), self.at(next_column, row), along_x);
                let top = lerp(
                    self.at(column, next_row),
                    self.at(next_column, next_row),
                    along_x,
                );

                alpha.push(lerp(bottom, top, along_y));
            }
        }

        Self {
            width,
            height,
            alpha,
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_height_never_leaves_zero_to_255() {
        let mut heights = Heights::new(4, 4);

        heights.add(1, 1, 300);
        assert_eq!(heights.get(1, 1), 255);

        heights.add(1, 1, -400);
        assert_eq!(heights.get(1, 1), 0);

        heights.set(2, 3, 128);
        assert_eq!(heights.get(2, 3), 128);
        assert_eq!(heights.get(3, 2), 0);
    }

    #[test]
    fn the_grid_knows_its_own_edges() {
        let heights = Heights::new(8, 4);

        assert!(heights.contains(7, 3));
        assert!(!heights.contains(8, 0));
        assert!(!heights.contains(0, -1));
    }

    #[test]
    fn a_mask_clamps_its_alpha() {
        let mask = Mask::from_fn(3, 3, |x, _| x as f32 - 0.5);

        assert_eq!(mask.at(0, 0), 0.0);
        assert_eq!(mask.at(1, 0), 0.5);
        assert_eq!(mask.at(2, 0), 1.0);
    }

    #[test]
    fn no_turn_leaves_the_mask_as_it_was() {
        let mask = Mask::from_fn(9, 9, |x, y| ((x * 3 + y) % 7) as f32 / 7.0);

        assert_eq!(mask.rotated(0), mask);
    }

    #[test]
    fn a_half_turn_mirrors_the_mask_through_its_centre() {
        let mask = Mask::from_fn(9, 9, |x, _| if x < 2 { 1.0 } else { 0.0 });
        let turned = mask.rotated(180);

        assert_eq!(turned.at(8, 4), 1.0);
        assert_eq!(turned.at(0, 4), 0.0);
    }

    #[test]
    fn scaling_keeps_corners_and_blends_between_them() {
        let mask = Mask::from_fn(2, 2, |x, _| x as f32);
        let wide = mask.scaled(8, 4);

        assert_eq!((wide.width, wide.height), (8, 4));
        assert_eq!(wide.at(0, 0), 0.0);
        assert!(wide.at(4, 1) > 0.4 && wide.at(4, 1) < 0.6);
        assert!(wide.at(7, 3) > 0.8);
    }
}
