const NET_BIG: i32 = i32::MAX;
const NET_SEED: i32 = 161_803_398;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XorShift {
    state: u32,
}

impl XorShift {
    pub fn new(seed: u32) -> Self {
        let mut random = Self { state: seed };
        random.next_state();
        random
    }

    pub fn state(self) -> u32 {
        self.state
    }

    fn next_state(&mut self) -> u32 {
        let current = self.state;
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 5;
        current
    }

    pub fn next_bool(&mut self) -> bool {
        self.next_state() & 1 == 1
    }

    pub fn next_int(&mut self, min: i32, max: i32) -> i32 {
        if max <= min {
            return min;
        }

        let range = max.wrapping_sub(min) as u32;
        ((u64::from(self.next_state()) * u64::from(range)) >> 32) as i32 + min
    }

    pub fn next_float(&mut self) -> f32 {
        f32::from_bits(0x3f80_0000 | (self.next_state() >> 9)) - 1.0
    }

    pub fn next_float_in(&mut self, min: f32, max: f32) -> f32 {
        self.next_float() * (max - min) + min
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetRandom {
    next: usize,
    next_p: usize,
    seeds: [i32; 56],
}

impl NetRandom {
    pub fn new(seed: i32) -> Self {
        let mut seeds = [0_i32; 56];
        let subtraction = if seed == i32::MIN {
            i32::MAX
        } else {
            seed.abs()
        };
        let mut mj = NET_SEED.wrapping_sub(subtraction);
        seeds[55] = mj;
        let mut mk = 1_i32;

        for i in 1..55 {
            let ii = (21 * i) % 55;
            seeds[ii] = mk;
            mk = mj.wrapping_sub(mk);
            if mk < 0 {
                mk = mk.wrapping_add(NET_BIG);
            }
            mj = seeds[ii];
        }

        for _ in 1..5 {
            for i in 1..56 {
                seeds[i] = seeds[i].wrapping_sub(seeds[1 + (i + 30) % 55]);
                if seeds[i] < 0 {
                    seeds[i] = seeds[i].wrapping_add(NET_BIG);
                }
            }
        }

        Self {
            next: 0,
            next_p: 21,
            seeds,
        }
    }

    fn internal_sample(&mut self) -> i32 {
        let next = if self.next + 1 >= 56 {
            1
        } else {
            self.next + 1
        };
        let next_p = if self.next_p + 1 >= 56 {
            1
        } else {
            self.next_p + 1
        };
        let mut value = self.seeds[next].wrapping_sub(self.seeds[next_p]);

        if value == NET_BIG {
            value -= 1;
        }
        if value < 0 {
            value = value.wrapping_add(NET_BIG);
        }

        self.seeds[next] = value;
        self.next = next;
        self.next_p = next_p;
        value
    }

    pub fn sample(&mut self) -> f64 {
        f64::from(self.internal_sample()) * (1.0 / f64::from(NET_BIG))
    }

    pub fn next_in(&mut self, min: i32, max: i32) -> i32 {
        if max <= min {
            return min;
        }

        let range = i64::from(max) - i64::from(min);
        (self.sample() * range as f64) as i64 as i32 + min
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Randy {
    rand: XorShift,
    rnd: NetRandom,
}

impl Randy {
    pub fn new(seed: i32) -> Self {
        let seed = if seed == 0 { 1 } else { seed };
        let mut rand = XorShift::new(seed as u32);
        rand.next_bool();

        Self {
            rand,
            rnd: NetRandom::new(seed),
        }
    }

    pub fn random_int(&mut self, min: i32, max: i32) -> i32 {
        self.rand.next_int(min, max)
    }

    pub fn random_bool(&mut self) -> bool {
        self.rand.next_bool()
    }

    pub fn random(&mut self) -> f32 {
        self.rand.next_float()
    }

    pub fn random_float(&mut self, min: f32, max: f32) -> f32 {
        self.rand.next_float_in(min, max)
    }

    pub fn random_chance(&mut self, chance: f32) -> bool {
        chance >= self.random()
    }

    pub fn index(&mut self, len: usize) -> usize {
        self.rnd.next_in(0, len as i32) as usize
    }

    pub fn index_from(&mut self, from: usize, len: usize) -> usize {
        self.rnd.next_in(from as i32, len as i32) as usize
    }

    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.index(items.len())]
    }

    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        if items.len() < 2 {
            return;
        }

        for i in 0..items.len() {
            let j = self.index_from(i, items.len());
            items.swap(i, j);
        }
    }

    pub fn shuffle_last<T>(&mut self, items: &mut [T]) {
        if items.len() < 2 {
            return;
        }

        let j = self.index(items.len());
        let last = items.len() - 1;
        items.swap(last, j);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_xorshift_steps_like_unity_mathematics() {
        let mut random = XorShift::new(1);

        assert_eq!(random.state(), 270_369);
        assert_eq!(random.next_int(0, i32::MAX), 135_184);
        assert_eq!(random.state(), 67_634_689);
    }

    #[test]
    fn whole_numbers_stay_in_their_half_open_range() {
        let mut randy = Randy::new(99);

        for _ in 0..10_000 {
            let value = randy.random_int(-5, 7);
            assert!((-5..7).contains(&value), "{value}");
        }
        assert_eq!(randy.random_int(4, 4), 4);
    }

    #[test]
    fn floats_stay_in_their_half_open_range_and_cover_it() {
        let mut randy = Randy::new(7);
        let mut low = f32::MAX;
        let mut high = f32::MIN;

        for _ in 0..10_000 {
            let value = randy.random_float(0.3, 2.0);
            low = low.min(value);
            high = high.max(value);
        }

        assert!(low >= 0.3 && high < 2.0);
        assert!(low < 0.35 && high > 1.95, "{low}..{high}");
    }

    #[test]
    fn the_net_random_matches_the_dotnet_sequence() {
        let mut random = NetRandom::new(123);

        assert_eq!(random.next_in(0, 100), 98);
        assert_eq!(random.next_in(0, 100), 90);
        assert_eq!(random.next_in(0, 100), 74);
    }

    #[test]
    fn the_same_seed_draws_the_same_numbers() {
        let mut a = Randy::new(5);
        let mut b = Randy::new(5);

        for _ in 0..100 {
            assert_eq!(a.random_int(0, 1000), b.random_int(0, 1000));
            assert_eq!(a.index(77), b.index(77));
        }
    }

    #[test]
    fn a_zero_seed_behaves_like_seed_one() {
        assert_eq!(Randy::new(0), Randy::new(1));
    }

    #[test]
    fn a_shuffle_keeps_every_item() {
        let mut randy = Randy::new(3);
        let mut items: Vec<u32> = (0..50).collect();
        randy.shuffle(&mut items);
        let mut sorted = items.clone();
        sorted.sort_unstable();

        assert_ne!(items, sorted);
        assert_eq!(sorted, (0..50).collect::<Vec<_>>());
    }
}
