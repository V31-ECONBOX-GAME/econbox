use econbox_field::Mask;
use econbox_noise::perlin;

pub const SHAPE_SIZE: u32 = 60;
pub const EDGE_SIZE: u32 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Shape {
    Dome,
    Diamond,
    Halo,
    Crescent,
    BrokenRing,
    Cloud,
    Swirl,
}

pub const SHAPES: [Shape; 7] = [
    Shape::Dome,
    Shape::Diamond,
    Shape::Halo,
    Shape::Crescent,
    Shape::BrokenRing,
    Shape::Cloud,
    Shape::Swirl,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    Left,
    Right,
    Up,
    Down,
}

fn smoothstep(from: f32, to: f32, value: f32) -> f32 {
    let t = ((value - from) / (to - from)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn polar(x: u32, y: u32) -> (f32, f32, f32, f32) {
    let centre = (SHAPE_SIZE as f32 - 1.0) / 2.0;
    let dx = x as f32 - centre;
    let dy = y as f32 - centre;

    (dx, dy, dx.hypot(dy), dy.atan2(dx))
}

impl Shape {
    pub fn alpha(self, x: u32, y: u32) -> f32 {
        let (dx, dy, r, angle) = polar(x, y);
        let (fx, fy) = (x as f32, y as f32);

        match self {
            Self::Dome => 0.39 * (1.0 - r / 30.0).max(0.0),
            Self::Diamond => 0.36 * (1.0 - (dx.abs() + dy.abs()) / 30.0).max(0.0),
            Self::Halo => {
                0.28 * (-((r - 19.0) / 6.5).powi(2)).exp() * (1.0 - smoothstep(27.0, 30.0, r))
            }
            Self::Crescent => {
                let outer = 1.0 - smoothstep(21.0, 23.0, r);
                let inner = smoothstep(18.0, 20.0, (dx + 5.0).hypot(dy + 6.0));

                0.93 * outer * inner + 0.02 * (1.0 - smoothstep(26.0, 29.0, r))
            }
            Self::BrokenRing => {
                let band = smoothstep(14.0, 19.0, r) * (1.0 - smoothstep(24.0, 28.0, r));
                let gaps = smoothstep(0.3, 0.5, perlin(fx * 0.09 + 17.3, fy * 0.09 + 5.1));
                let core = 0.3
                    * (1.0 - smoothstep(12.0, 20.0, r))
                    * smoothstep(0.45, 0.65, perlin(fx * 0.2 + 40.7, fy * 0.2 + 3.3));

                band * gaps * 0.72 + core * 0.8 + 0.04 * (1.0 - smoothstep(27.0, 29.0, r))
            }
            Self::Cloud => {
                let disc = 1.0 - smoothstep(24.0, 29.5, r);
                let puff = smoothstep(0.25, 0.8, perlin(fx * 0.12 + 91.7, fy * 0.12 + 12.9));

                disc * (0.26 + 0.48 * puff)
            }
            Self::Swirl => {
                let disc = 1.0 - smoothstep(25.0, 30.0, r);
                let grain = perlin(fx * 0.15 + 61.1, fy * 0.15 + 33.7);
                let warp = perlin(fx * 0.08 + 7.9, fy * 0.08 + 71.3);
                let arm = 0.5 + 0.5 * (angle - r * 0.42 + (warp - 0.5) * 5.0).cos();

                disc * (0.3 + 0.6 * arm * arm) * (0.7 + 0.5 * grain)
            }
        }
    }

    pub fn mask(self) -> Mask {
        Mask::from_fn(SHAPE_SIZE, SHAPE_SIZE, |x, y| self.alpha(x, y))
    }
}

impl Side {
    pub fn alpha(self, x: u32, y: u32) -> f32 {
        let last = EDGE_SIZE - 1;
        let (depth, along) = match self {
            Self::Left => (x, y),
            Self::Right => (last - x, y),
            Self::Down => (y, x),
            Self::Up => (last - y, x),
        };
        let reach = 27.0 + 6.0 * (std::f32::consts::PI * along as f32 / EDGE_SIZE as f32).sin();

        1.0 - smoothstep(reach - 14.0, reach + 14.0, depth as f32)
    }

    pub fn mask(self) -> Mask {
        Mask::from_fn(EDGE_SIZE, EDGE_SIZE, |x, y| self.alpha(x, y))
    }
}

pub fn shape_masks() -> Vec<Mask> {
    SHAPES.into_iter().map(Shape::mask).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mean(mask: &Mask) -> f32 {
        mask.alpha.iter().sum::<f32>() / mask.alpha.len() as f32 * 255.0
    }

    #[test]
    fn each_shape_carries_about_as_much_weight_as_the_one_it_stands_for() {
        let expected = [24.9, 15.7, 28.3, 38.9, 48.4, 67.8, 84.6];

        for (shape, want) in SHAPES.into_iter().zip(expected) {
            let got = mean(&shape.mask());

            assert!((got - want).abs() < want * 0.2, "{shape:?} {got} vs {want}");
        }
    }

    #[test]
    fn every_shape_fades_out_before_the_border_of_its_square() {
        for shape in SHAPES {
            let mask = shape.mask();

            for step in 0..SHAPE_SIZE {
                for (x, y) in [
                    (step, 0),
                    (0, step),
                    (step, SHAPE_SIZE - 1),
                    (SHAPE_SIZE - 1, step),
                ] {
                    assert!(mask.at(x, y) < 0.05, "{shape:?} at {x},{y}");
                }
            }
        }
    }

    #[test]
    fn the_soft_shapes_are_weak_and_the_sharp_ones_reach_full_strength() {
        let peak = |shape: Shape| shape.mask().alpha.iter().copied().fold(0.0, f32::max);

        assert!(peak(Shape::Dome) < 0.4);
        assert!(peak(Shape::Halo) < 0.3);
        assert!(peak(Shape::Crescent) > 0.9);
        assert!(Shape::Halo.alpha(30, 30) < 0.05);
    }

    #[test]
    fn an_edge_mask_is_solid_at_its_side_and_empty_past_the_middle() {
        for side in [Side::Left, Side::Right, Side::Up, Side::Down] {
            let mask = side.mask();
            let (outer, inner) = match side {
                Side::Left => ((0, 32), (63, 32)),
                Side::Right => ((63, 32), (0, 32)),
                Side::Down => ((32, 0), (32, 63)),
                Side::Up => ((32, 63), (32, 0)),
            };

            assert!(mask.at(outer.0, outer.1) > 0.99, "{side:?}");
            assert!(mask.at(inner.0, inner.1) < 0.01, "{side:?}");
            assert!(
                (mean(&mask) - 117.0).abs() < 15.0,
                "{side:?} {}",
                mean(&mask)
            );
        }
    }

    #[test]
    fn the_edge_reaches_further_in_the_middle_of_its_stretch() {
        let mask = Side::Left.mask();

        assert!(mask.at(32, 32) > mask.at(32, 0));
    }
}
