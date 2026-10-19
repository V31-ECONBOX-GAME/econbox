use crate::biome::{BIOME_HILL, BIOME_SAND, Biome, biome_of};
use econbox_field::Heights;
use econbox_tile::{Flora, Ground, Layer, Overlay, Tile};
use econbox_tilemap::TileMap;

pub const ZONE_SIZE: u32 = 8;
pub const CHUNK_SIZE: u32 = 16;

const NEIGHBOURS: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
const NEIGHBOURS_ALL: [(i32, i32); 8] = [
    (-1, 0),
    (1, 0),
    (0, -1),
    (0, 1),
    (-1, -1),
    (-1, 1),
    (1, -1),
    (1, 1),
];

#[derive(Debug, Clone, PartialEq)]
pub struct World {
    pub width: u32,
    pub height: u32,
    pub heights: Heights,
    pub main: Vec<Ground>,
    pub top: Vec<Option<Overlay>>,
    pub frozen: Vec<bool>,
    pub flora: Vec<Option<Flora>>,
}

impl World {
    pub fn new(width: u32, height: u32) -> Self {
        let cells = (width * height) as usize;

        Self {
            width,
            height,
            heights: Heights::new(width, height),
            main: vec![Ground::DeepOcean; cells],
            top: vec![None; cells],
            frozen: vec![false; cells],
            flora: vec![None; cells],
        }
    }

    pub fn cells(&self) -> usize {
        self.main.len()
    }

    pub fn index(&self, x: u32, y: u32) -> usize {
        (y * self.width + x) as usize
    }

    pub fn position(&self, index: usize) -> (u32, u32) {
        (index as u32 % self.width, index as u32 / self.width)
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height
    }

    pub fn tile_at(&self, x: i32, y: i32) -> Option<usize> {
        self.contains(x, y).then(|| self.index(x as u32, y as u32))
    }

    pub fn height_at(&self, index: usize) -> i32 {
        i32::from(self.heights.values[index])
    }

    pub fn set_height(&mut self, index: usize, value: i32) {
        self.heights.values[index] = value.clamp(0, econbox_field::MAX_HEIGHT) as u8;
    }

    pub fn add_height(&mut self, index: usize, by: i32) {
        self.set_height(index, self.height_at(index) + by);
    }

    pub fn neighbours(&self, index: usize) -> impl Iterator<Item = usize> + '_ {
        self.around(index, &NEIGHBOURS)
    }

    pub fn neighbours_all(&self, index: usize) -> impl Iterator<Item = usize> + '_ {
        self.around(index, &NEIGHBOURS_ALL)
    }

    fn around<'a>(
        &'a self,
        index: usize,
        offsets: &'static [(i32, i32)],
    ) -> impl Iterator<Item = usize> + 'a {
        let (x, y) = self.position(index);

        offsets
            .iter()
            .filter_map(move |(dx, dy)| self.tile_at(x as i32 + dx, y as i32 + dy))
    }

    pub fn terraform(&mut self, index: usize, main: Ground) {
        self.main[index] = main;
        self.top[index] = None;
        self.frozen[index] = false;
    }

    pub fn set_top(&mut self, index: usize, top: Option<Overlay>) {
        self.top[index] = top;
    }

    pub fn is_soil(&self, index: usize) -> bool {
        self.top[index].is_none() && self.main[index].is_soil()
    }

    pub fn can_be_biome(&self, index: usize) -> bool {
        self.top[index].is_some() || self.main[index].is_soil()
    }

    pub fn is_ground(&self, index: usize) -> bool {
        self.main[index].layer() == Layer::Ground
    }

    pub fn is_block(&self, index: usize) -> bool {
        self.main[index].layer() == Layer::Block
    }

    pub fn is_liquid(&self, index: usize) -> bool {
        self.main[index].is_water() || self.main[index].is_lava()
    }

    pub fn biome(&self, index: usize) -> Option<&'static Biome> {
        match self.top[index] {
            Some(overlay) => biome_of(overlay),
            None => match self.main[index] {
                Ground::Sand => Some(&BIOME_SAND),
                Ground::Hills => Some(&BIOME_HILL),
                _ => None,
            },
        }
    }

    pub fn update_tile_types(&mut self) {
        for index in 0..self.cells() {
            self.main[index] = Ground::from_height(self.heights.values[index]);
        }
    }

    pub fn islands(&self) -> Islands {
        let mut island = vec![u32::MAX; self.cells()];
        let mut sizes = Vec::new();
        let mut wave = Vec::new();

        for start in 0..self.cells() {
            if island[start] != u32::MAX {
                continue;
            }

            let id = sizes.len() as u32;
            let layer = self.main[start].layer();
            let mut size = 0;
            island[start] = id;
            wave.push(start);

            while let Some(index) = wave.pop() {
                size += 1;

                for next in self.neighbours_all(index) {
                    if island[next] == u32::MAX
                        && self.main[next].layer() == layer
                        && !self.diagonal_blocked(index, next)
                    {
                        island[next] = id;
                        wave.push(next);
                    }
                }
            }

            sizes.push(size);
        }

        Islands { island, sizes }
    }

    fn diagonal_blocked(&self, from: usize, to: usize) -> bool {
        let (fx, fy) = self.position(from);
        let (tx, ty) = self.position(to);
        let dx = tx as i32 - fx as i32;
        let dy = ty as i32 - fy as i32;

        if dx.abs() != 1 || dy.abs() != 1 {
            return false;
        }

        let side = self.tile_at(fx as i32 + dx, fy as i32);
        let other = self.tile_at(fx as i32, fy as i32 + dy);

        side.is_none_or(|index| self.is_block(index))
            || other.is_none_or(|index| self.is_block(index))
    }

    pub fn into_map(self) -> TileMap {
        let mut map = TileMap::filled(self.width, self.height, Tile::default());

        for index in 0..self.cells() {
            let tile = Tile::new(self.main[index], self.top[index].unwrap_or(Overlay::Bare))
                .with_frozen(self.frozen[index]);

            map.tiles[index] = Tile {
                flora: self.flora[index],
                ..tile
            };
        }

        map.heights = self.heights.values;
        map
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Islands {
    pub island: Vec<u32>,
    pub sizes: Vec<u32>,
}

impl Islands {
    pub fn size_of(&self, index: usize) -> u32 {
        self.sizes[self.island[index] as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neighbours_come_in_the_worldbox_order_and_stop_at_the_border() {
        let world = World::new(4, 4);
        let centre = world.index(1, 1);
        let corner = world.index(0, 0);

        let around: Vec<_> = world
            .neighbours_all(centre)
            .map(|index| world.position(index))
            .collect();

        assert_eq!(
            around,
            [
                (0, 1),
                (2, 1),
                (1, 0),
                (1, 2),
                (0, 0),
                (0, 2),
                (2, 0),
                (2, 2)
            ]
        );
        assert_eq!(world.neighbours(corner).count(), 2);
        assert_eq!(world.neighbours_all(corner).count(), 3);
    }

    #[test]
    fn heights_turn_into_the_matching_ground() {
        let mut world = World::new(3, 1);
        world.set_height(0, 20);
        world.set_height(1, 120);
        world.set_height(2, 400);
        world.update_tile_types();

        assert_eq!(
            world.main,
            [Ground::DeepOcean, Ground::SoilLow, Ground::Summit]
        );
        assert_eq!(world.height_at(2), 255);
    }

    #[test]
    fn mountains_split_islands_and_close_diagonal_gaps() {
        let mut world = World::new(5, 5);

        for index in 0..world.cells() {
            world.main[index] = Ground::SoilLow;
        }
        for y in 0..5 {
            let index = world.index(2, y);
            world.main[index] = Ground::Mountains;
        }

        let islands = world.islands();

        assert_eq!(islands.size_of(world.index(0, 0)), 10);
        assert_eq!(islands.size_of(world.index(4, 4)), 10);
        assert_ne!(
            islands.island[world.index(0, 0)],
            islands.island[world.index(4, 0)]
        );
        assert_eq!(islands.size_of(world.index(2, 2)), 5);
    }

    #[test]
    fn a_diagonal_step_between_two_seas_joins_them() {
        let mut world = World::new(2, 2);
        world.main = vec![
            Ground::DeepOcean,
            Ground::SoilLow,
            Ground::SoilLow,
            Ground::DeepOcean,
        ];

        let islands = world.islands();

        assert_eq!(islands.size_of(0), 2);
        assert_eq!(islands.size_of(1), 2);
    }

    #[test]
    fn only_bare_soil_counts_as_soil_and_sand_and_hills_have_their_own_biome() {
        let mut world = World::new(3, 1);
        world.main = vec![Ground::SoilLow, Ground::Sand, Ground::Hills];

        assert!(world.is_soil(0));
        world.set_top(0, Some(Overlay::Grass));
        assert!(!world.is_soil(0));
        assert_eq!(world.biome(0).map(|biome| biome.id), Some("biome_grass"));
        assert_eq!(world.biome(1).map(|biome| biome.id), Some("biome_sand"));
        assert_eq!(world.biome(2).map(|biome| biome.id), Some("biome_hill"));
    }

    #[test]
    fn the_finished_world_keeps_its_heights_frozen_peaks_and_plants() {
        let mut world = World::new(2, 1);
        world.main = vec![Ground::Mountains, Ground::SoilHigh];
        world.frozen[0] = true;
        world.top[1] = Some(Overlay::Jungle);
        world.flora[1] = Some(Flora::Tree);
        world.set_height(0, 225);

        let map = world.into_map();

        assert!(map.at(0, 0).frozen);
        assert_eq!(map.at(1, 0).overlay, Overlay::Jungle);
        assert_eq!(map.at(1, 0).flora, Some(Flora::Tree));
        assert_eq!(map.height_at(0, 0), 225);
    }
}
