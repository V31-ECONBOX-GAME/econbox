use crate::shapes::{EDGE_SIZE, Side};
use crate::template::PerlinReplaceContainer;
use crate::world::World;
use econbox_field::Mask;
use econbox_noise::{Randy, perlin};
use econbox_tile::Ground;

pub const RING_GROUNDS: [Ground; 4] = [
    Ground::SoilHigh,
    Ground::Hills,
    Ground::Mountains,
    Ground::Summit,
];

fn aspect(world: &World) -> (f32, f32) {
    let width = world.width as i32;
    let height = world.height as i32;

    if width > height {
        ((width / height) as f32, 1.0)
    } else {
        (1.0, (height / width) as f32)
    }
}

pub fn dist_tile(from: (i32, i32), to: (i32, i32)) -> f32 {
    let dx = from.0 - to.0;
    let dy = from.1 - to.1;

    ((dx * dx + dy * dy) as f32).sqrt()
}

pub fn apply_perlin_noise(
    world: &mut World,
    pos_x: f32,
    pos_y: f32,
    alpha_mod: f32,
    scale_mod: f32,
    subtract: bool,
) {
    let strength = 255.0 * alpha_mod;
    let (ratio_x, ratio_y) = aspect(world);
    let width = world.width as f32;
    let height = world.height as f32;

    for x in 0..world.width {
        for y in 0..world.height {
            let nx = (pos_x + x as f32) / width;
            let ny = (pos_y + y as f32) / height;
            let mut value =
                (perlin(nx * scale_mod * ratio_x, ny * scale_mod * ratio_y) * strength) as i32;

            if subtract {
                value = -value;
            }

            let index = world.index(x, y);
            world.add_height(index, value);
        }
    }
}

pub fn apply_random_shape(
    world: &mut World,
    randy: &mut Randy,
    shapes: &[Mask],
    dist_max: f32,
    strength_mod: f32,
    subtract: bool,
) {
    let shape = randy.pick(shapes);
    let width = (shape.width as f32 * randy.random_float(0.3, 2.0)) as i32;
    let height = (shape.height as f32 * randy.random_float(0.3, 2.0)) as i32;
    let turned = shape.rotated(randy.random_int(0, 360));
    let scaled = turned.scaled(width.max(1) as u32, height.max(1) as u32);
    let width = scaled.width as i32;
    let height = scaled.height as i32;
    let map_width = world.width as i32;
    let map_height = world.height as i32;

    let mut left = map_width / 2
        - width / 2
        - randy.random_float(-width as f32 * dist_max, width as f32 * dist_max) as i32;
    let mut bottom = map_height / 2
        - height / 2
        - randy.random_float(-height as f32 * dist_max, height as f32 * dist_max) as i32;

    if left < 0 {
        left = 0;
    }
    if bottom < 0 {
        bottom = 0;
    }
    if left + width > map_width {
        left = map_width - width;
    }
    if bottom + height > map_height {
        bottom = map_height - height;
    }

    let strength = 255.0 * strength_mod;

    for i in 0..width {
        for j in 0..height {
            let Some(index) = world.tile_at(left + i, bottom + j) else {
                continue;
            };
            let mut value = (scaled.at(i as u32, j as u32) * strength) as i32;

            if subtract {
                value = -value;
            }

            world.add_height(index, value);
        }
    }
}

pub fn apply_perlin_replace(
    world: &mut World,
    randy: &mut Randy,
    container: &PerlinReplaceContainer,
) {
    let pos_x = randy.random_int(0, 15_000) as f32;
    let pos_y = randy.random_int(0, 15_000) as f32;
    let scale = container.scale as f32;
    let (ratio_x, ratio_y) = aspect(world);
    let width = world.width as f32;
    let height = world.height as f32;

    for x in 0..world.width {
        for y in 0..world.height {
            let index = world.index(x, y);
            let nx = (pos_x + x as f32) / width;
            let ny = (pos_y + y as f32) / height;
            let value = (perlin(nx * scale * ratio_x, ny * scale * ratio_y) * 255.0) as i32;

            for option in container.options {
                if value > option.replace_height_value && world.main[index] == option.from {
                    world.main[index] = option.to;
                }
            }
        }
    }
}

pub fn ring_height(height: i32) -> Option<i32> {
    RING_GROUNDS.into_iter().find_map(|ground| {
        let min = i32::from(ground.height_min()?);

        ground
            .additional_height()
            .iter()
            .any(|extra| height == min - extra)
            .then_some(min)
    })
}

pub fn apply_ring_effect(world: &mut World) {
    for index in 0..world.cells() {
        if let Some(height) = ring_height(world.height_at(index)) {
            world.set_height(index, height);
        }
    }
}

pub fn add_edge_gradient_circle(world: &mut World) {
    let centre = ((world.width / 2) as i32, (world.height / 2) as i32);
    let half = (world.width / 2) as f32;
    let outer = half * 0.99;
    let inner = half * 0.85;
    let span = outer - inner;

    for index in 0..world.cells() {
        let (x, y) = world.position(index);
        let dist = dist_tile((x as i32, y as i32), centre);

        if dist > outer {
            world.set_height(index, 0);
        } else if dist < outer && dist > inner {
            let kept = (outer - dist) / span;
            world.set_height(index, (world.height_at(index) as f32 * kept) as i32);
        }
    }
}

pub fn add_edge_square(world: &mut World, edges: &EdgeMasks) {
    let columns = (world.width as f32 / EDGE_SIZE as f32) as i32 + 1;
    let rows = (world.height as f32 / EDGE_SIZE as f32) as i32 + 1;

    for row in 0..rows {
        fill_edge(world, 0, row, &edges.left);
    }
    for row in 0..rows {
        fill_edge(world, columns - 2, row, &edges.right);
    }
    for column in 0..columns {
        fill_edge(world, column, 0, &edges.down);
    }
    for column in 0..columns {
        fill_edge(world, column, rows - 2, &edges.up);
    }
}

fn fill_edge(world: &mut World, cell_x: i32, cell_y: i32, mask: &Mask) {
    for i in 0..mask.height {
        for j in 0..mask.width {
            let depth = (mask.at(j, i) * 255.0) as i32;
            let x = j as i32 + cell_x * EDGE_SIZE as i32;
            let y = i as i32 + cell_y * EDGE_SIZE as i32;

            if let Some(index) = world.tile_at(x, y) {
                world.add_height(index, -depth);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeMasks {
    pub left: Mask,
    pub right: Mask,
    pub up: Mask,
    pub down: Mask,
}

impl EdgeMasks {
    pub fn new() -> Self {
        Self {
            left: Side::Left.mask(),
            right: Side::Right.mask(),
            up: Side::Up.mask(),
            down: Side::Down.mask(),
        }
    }
}

impl Default for EdgeMasks {
    fn default() -> Self {
        Self::new()
    }
}

pub fn raycast(world: &World, from: (i32, i32), to: (i32, i32), reach: f32) -> Vec<usize> {
    let mut result = Vec::new();
    let target = world.tile_at(to.0, to.1);
    let dist = dist_tile(from, to) * reach;
    let mut last = None;

    if dist > 0.0 {
        let mut step = 0;

        while step as f32 <= dist {
            let t = (step as f32 / dist).clamp(0.0, 1.0);
            let x = from.0 as f32 + (to.0 - from.0) as f32 * t;
            let y = from.1 as f32 + (to.1 - from.1) as f32 * t;

            if let Some(index) = world.tile_at(x.floor() as i32, y.floor() as i32)
                && Some(index) != last
            {
                result.push(index);
                last = Some(index);
            }

            step += 1;
        }
    }

    if let Some(target) = target
        && last != Some(target)
    {
        result.push(target);
    }

    result
}

pub fn fill_tiles(
    world: &mut World,
    from: (i32, i32),
    to: (i32, i32),
    ground: Ground,
) -> Vec<usize> {
    let line = raycast(world, from, to, 1.0);

    for index in &line {
        world.terraform(*index, ground);
    }

    line
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::PerlinReplaceOption;

    #[test]
    fn the_ring_effect_snaps_heights_just_below_a_band_onto_it() {
        assert_eq!(ring_height(113), Some(128));
        assert_eq!(ring_height(111), Some(128));
        assert_eq!(ring_height(118), Some(128));
        assert_eq!(ring_height(119), None);
        assert_eq!(ring_height(110), None);
        assert_eq!(ring_height(197), Some(199));
        assert_eq!(ring_height(205), Some(199));
        assert_eq!(ring_height(208), Some(210));
        assert_eq!(ring_height(206), Some(210));
        assert_eq!(ring_height(228), Some(230));
        assert_eq!(ring_height(226), Some(230));
        assert_eq!(ring_height(150), None);
    }

    #[test]
    fn perlin_noise_raises_the_whole_map_and_subtracting_lowers_it_again() {
        let mut world = World::new(64, 64);
        apply_perlin_noise(&mut world, 10.0, 20.0, 1.0, 5.0, false);
        let raised: Vec<i32> = (0..world.cells()).map(|i| world.height_at(i)).collect();

        assert!(raised.iter().all(|height| (0..=255).contains(height)));
        assert!(raised.iter().any(|height| *height > 150));
        assert!(raised.iter().any(|height| *height < 100));

        apply_perlin_noise(&mut world, 10.0, 20.0, 1.0, 5.0, true);
        assert!((0..world.cells()).all(|i| world.height_at(i) <= 2));
    }

    #[test]
    fn the_round_edge_sinks_everything_outside_the_circle() {
        let mut world = World::new(64, 64);
        for index in 0..world.cells() {
            world.set_height(index, 200);
        }

        add_edge_gradient_circle(&mut world);

        assert_eq!(world.height_at(world.index(0, 0)), 0);
        assert_eq!(world.height_at(world.index(32, 32)), 200);
        assert_eq!(world.height_at(world.index(32, 0)), 0);
        assert_eq!(world.height_at(world.index(32, 6)), 200);
        let fading = world.height_at(world.index(32, 3));
        assert!(fading > 0 && fading < 200, "{fading}");
    }

    #[test]
    fn the_square_edge_sinks_every_border_of_the_map() {
        let mut world = World::new(128, 128);
        for index in 0..world.cells() {
            world.set_height(index, 255);
        }

        add_edge_square(&mut world, &EdgeMasks::new());

        for step in 0..128 {
            for (x, y) in [(step, 0), (0, step), (step, 127), (127, step)] {
                assert_eq!(world.height_at(world.index(x, y)), 0, "{x},{y}");
            }
        }
        assert_eq!(world.height_at(world.index(64, 64)), 255);
    }

    #[test]
    fn a_random_shape_carves_into_the_map_and_stays_on_it() {
        let mut world = World::new(128, 128);
        for index in 0..world.cells() {
            world.set_height(index, 200);
        }
        let mut randy = Randy::new(4);
        let shapes = crate::shapes::shape_masks();

        for _ in 0..5 {
            apply_random_shape(&mut world, &mut randy, &shapes, 2.0, 0.7, true);
        }

        assert!((0..world.cells()).any(|i| world.height_at(i) < 200));
        assert!((0..world.cells()).any(|i| world.height_at(i) == 200));
    }

    #[test]
    fn perlin_replace_only_touches_the_ground_it_names() {
        let mut world = World::new(32, 32);
        for index in 0..world.cells() {
            world.main[index] = if index % 2 == 0 {
                Ground::SoilHigh
            } else {
                Ground::Sand
            };
        }
        let container = PerlinReplaceContainer {
            scale: 15,
            options: &[PerlinReplaceOption {
                replace_height_value: 0,
                from: Ground::SoilHigh,
                to: Ground::SoilLow,
            }],
        };

        apply_perlin_replace(&mut world, &mut Randy::new(1), &container);

        assert!(world.main.iter().all(|g| *g != Ground::SoilHigh));
        assert_eq!(
            world.main.iter().filter(|g| **g == Ground::Sand).count(),
            512
        );
    }

    #[test]
    fn a_line_of_tiles_runs_from_end_to_end_without_gaps() {
        let world = World::new(16, 16);
        let line = raycast(&world, (0, 0), (15, 0), 1.0);

        assert_eq!(line.len(), 16);
        assert_eq!(line[0], world.index(0, 0));
        assert_eq!(*line.last().expect("end"), world.index(15, 0));

        let diagonal = raycast(&world, (0, 0), (15, 15), 1.0);
        assert_eq!(*diagonal.last().expect("end"), world.index(15, 15));
        assert!(diagonal.len() >= 16);
    }
}
