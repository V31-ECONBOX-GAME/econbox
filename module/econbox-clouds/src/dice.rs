#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dice(u32);

impl Dice {
    pub fn new(seed: u32) -> Self {
        let state = seed.wrapping_mul(0x9e37_79b9) ^ 0x7f4a_7c15;
        Self(if state == 0 { 1 } else { state })
    }

    pub fn roll(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }

    pub fn unit(&mut self) -> f32 {
        (self.roll() >> 8) as f32 / (1 << 24) as f32
    }

    pub fn range(&mut self, low: f32, high: f32) -> f32 {
        low + (high - low) * self.unit()
    }

    pub fn int(&mut self, low: i32, high: i32) -> i32 {
        if high <= low {
            return low;
        }

        low + (self.roll() % (high - low) as u32) as i32
    }

    pub fn chance(&mut self, odds: f32) -> bool {
        self.unit() < odds
    }

    pub fn coin(&mut self) -> bool {
        self.roll() & 1 == 1
    }

    pub fn pick<T: Copy>(&mut self, items: &[T]) -> Option<T> {
        if items.is_empty() {
            return None;
        }

        Some(items[self.roll() as usize % items.len()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integers_stay_inside_a_half_open_range() {
        let mut dice = Dice::new(9);

        for _ in 0..5000 {
            let roll = dice.int(-50, 30);
            assert!((-50..30).contains(&roll));
        }
    }

    #[test]
    fn units_stay_below_one() {
        let mut dice = Dice::new(1);

        for _ in 0..5000 {
            let unit = dice.unit();
            assert!((0.0..1.0).contains(&unit));
        }
    }

    #[test]
    fn the_same_seed_rolls_the_same_numbers() {
        let mut one = Dice::new(42);
        let mut two = Dice::new(42);

        for _ in 0..10 {
            assert_eq!(one.roll(), two.roll());
        }
    }
}
