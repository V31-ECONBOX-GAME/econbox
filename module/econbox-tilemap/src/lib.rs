//! The tile grid, chunked the way WorldBox chunks it.

use bevy::prelude::{Message, Resource};
use econbox_tile::{Ground, Overlay, Tile};
use std::collections::BTreeMap;

pub const CHUNK: u32 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TilePos {
    pub x: u32,
    pub y: u32,
}

impl TilePos {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    pub fn chunk(self) -> (u32, u32) {
        (self.x / CHUNK, self.y / CHUNK)
    }
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct Painted {
    pub cells: Vec<TilePos>,
}

impl Painted {
    pub fn new(cells: Vec<TilePos>) -> Self {
        Self { cells }
    }
}

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct TileMap {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Tile>,
    pub heights: Vec<u8>,
}

impl TileMap {
    pub fn filled(width: u32, height: u32, tile: Tile) -> Self {
        let cells = (width * height) as usize;

        Self {
            width,
            height,
            tiles: vec![tile; cells],
            heights: vec![tile.ground.height_min().unwrap_or(0); cells],
        }
    }

    pub fn height_at(&self, x: u32, y: u32) -> u8 {
        self.heights[self.index(x, y)]
    }

    pub fn set_height(&mut self, x: u32, y: u32, height: u8) {
        let index = self.index(x, y);
        self.heights[index] = height;
    }

    pub fn index(&self, x: u32, y: u32) -> usize {
        (y.min(self.height - 1) * self.width + x.min(self.width - 1)) as usize
    }

    pub fn at(&self, x: u32, y: u32) -> Tile {
        self.tiles[self.index(x, y)]
    }

    pub fn get(&self, pos: TilePos) -> Option<Tile> {
        (pos.x < self.width && pos.y < self.height).then(|| self.at(pos.x, pos.y))
    }

    pub fn set(&mut self, x: u32, y: u32, tile: Tile) {
        let index = self.index(x, y);
        self.tiles[index] = tile;
    }

    pub fn cells(&self) -> usize {
        self.tiles.len()
    }

    pub fn chunks(&self) -> (u32, u32) {
        (self.width.div_ceil(CHUNK), self.height.div_ceil(CHUNK))
    }

    pub fn land_cells(&self) -> usize {
        self.tiles.iter().filter(|tile| tile.is_land()).count()
    }

    pub fn water_share(&self) -> f32 {
        1.0 - self.land_cells() as f32 / self.cells().max(1) as f32
    }

    pub fn ground_counts(&self) -> BTreeMap<Ground, usize> {
        let mut counts = BTreeMap::new();

        for tile in &self.tiles {
            *counts.entry(tile.ground).or_insert(0) += 1;
        }

        counts
    }

    pub fn overlay_counts(&self) -> BTreeMap<Overlay, usize> {
        let mut counts = BTreeMap::new();

        for tile in &self.tiles {
            *counts.entry(tile.overlay).or_insert(0) += 1;
        }

        counts
    }

    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let [r, g, b] = self.at(x, y).textured(x, y);
        [r, g, b, 255]
    }

    pub fn pixels(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.cells() * 4);

        for y in 0..self.height {
            for x in 0..self.width {
                data.extend_from_slice(&self.pixel(x, y));
            }
        }

        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_map_is_all_one_tile() {
        let map = TileMap::filled(64, 64, Tile::bare(Ground::DeepOcean));

        assert_eq!(map.cells(), 4096);
        assert_eq!(map.water_share(), 1.0);
        assert_eq!(map.land_cells(), 0);
    }

    #[test]
    fn a_worldbox_sized_map_splits_into_seven_by_seven_chunks() {
        let map = TileMap::filled(448, 448, Tile::default());

        assert_eq!(map.chunks(), (7, 7));
        assert_eq!(CHUNK, 64);
    }

    #[test]
    fn a_tile_knows_which_chunk_it_belongs_to() {
        assert_eq!(TilePos::new(0, 0).chunk(), (0, 0));
        assert_eq!(TilePos::new(63, 63).chunk(), (0, 0));
        assert_eq!(TilePos::new(64, 0).chunk(), (1, 0));
        assert_eq!(TilePos::new(447, 447).chunk(), (6, 6));
    }

    #[test]
    fn reading_outside_the_map_gives_nothing() {
        let map = TileMap::filled(16, 16, Tile::default());

        assert!(map.get(TilePos::new(15, 15)).is_some());
        assert!(map.get(TilePos::new(16, 0)).is_none());
        assert!(map.get(TilePos::new(0, 16)).is_none());
    }

    #[test]
    fn what_is_written_is_what_is_read_back() {
        let mut map = TileMap::filled(8, 8, Tile::default());
        let tile = Tile::new(Ground::SoilHigh, Overlay::Jungle);
        map.set(3, 5, tile);

        assert_eq!(map.at(3, 5), tile);
        assert_eq!(map.at(3, 4), Tile::default());
    }

    #[test]
    fn every_tile_keeps_a_worldbox_height() {
        let mut map = TileMap::filled(4, 4, Tile::bare(Ground::SoilHigh));

        assert_eq!(map.heights.len(), map.cells());
        assert_eq!(map.height_at(2, 2), 128);

        map.set_height(2, 2, 211);

        assert_eq!(map.height_at(2, 2), 211);
        assert_eq!(map.height_at(1, 2), 128);
    }

    #[test]
    fn the_pixels_carry_one_rgba_per_tile() {
        let map = TileMap::filled(4, 4, Tile::bare(Ground::Sand));

        assert_eq!(map.pixels().len(), 4 * 4 * 4);
    }

    #[test]
    fn one_pixel_is_the_textured_tile_plus_full_alpha() {
        let map = TileMap::filled(2, 2, Tile::bare(Ground::Sand));
        let [r, g, b] = Tile::bare(Ground::Sand).textured(1, 0);

        assert_eq!(map.pixel(1, 0), [r, g, b, 255]);
    }

    #[test]
    fn the_pixels_follow_the_textured_tile_colour() {
        let mut map = TileMap::filled(2, 1, Tile::bare(Ground::Sand));
        map.set(0, 0, Tile::bare(Ground::Mountains));
        let pixels = map.pixels();

        assert_eq!(&pixels[0..3], &Tile::bare(Ground::Mountains).textured(0, 0));
        assert_eq!(&pixels[4..7], &Tile::bare(Ground::Sand).textured(1, 0));
    }
}
