use crate::art::{self, Pixmap};
use crate::look::{self, Layer, Look};
use econbox_tile::{Flora, Ore, Tile};
use econbox_tilemap::TileMap;
use std::cmp::Reverse;
use std::collections::HashMap;

pub const PX: u32 = 4;

pub const EDGE_OCEAN: [u8; 3] = [0x2d, 0x61, 0xaf];
pub const EDGE_NO_OCEAN: [u8; 3] = [0x49, 0x49, 0x49];
pub const EDGE_HILLS: [u8; 3] = [0x31, 0x33, 0x33];
pub const EDGE_MOUNTAIN: [u8; 3] = [0x2c, 0x30, 0x32];
pub const EDGE_ALPHA: u8 = 26;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Solid([u8; 3]),
    Shade,
}

fn up(map: &TileMap, x: u32, y: u32) -> Option<&'static Look> {
    (y > 0).then(|| look::of(map.at(x, y - 1)))
}

fn down(map: &TileMap, x: u32, y: u32) -> Option<&'static Look> {
    (y + 1 < map.height).then(|| look::of(map.at(x, y + 1)))
}

pub fn edge(map: &TileMap, x: u32, y: u32) -> Option<Edge> {
    let here = look::of(map.at(x, y));

    if !here.check_edge || here.wall {
        return None;
    }

    if let Some(below) = down(map, x, y)
        && !below.wall
    {
        if here.edge_hills && !below.rocks {
            return Some(Edge::Solid(EDGE_HILLS));
        }
        if here.edge_mountains && below.edge_hills {
            return Some(Edge::Solid(EDGE_MOUNTAIN));
        }
        if here.rocks && !below.rocks {
            return Some(Edge::Solid(EDGE_MOUNTAIN));
        }
        if !below.ocean && !here.ocean && !below.is(here) && below.height_min < here.height_min {
            return Some(Edge::Shade);
        }
    }

    let above = up(map, x, y)?;

    if above.wall {
        return above.edge.map(Edge::Solid);
    }
    if here.ocean && above.ocean && above.height_min > here.height_min {
        return Some(Edge::Shade);
    }
    if above.ground && !above.fills_with_ocean {
        if here.layer == Layer::Ocean {
            return Some(Edge::Solid(EDGE_OCEAN));
        }
        if here.fills_with_ocean && !here.explodable {
            return Some(Edge::Solid(EDGE_NO_OCEAN));
        }
    }

    None
}

pub fn shade(rgb: [u8; 3]) -> [u8; 3] {
    let keep = u32::from(255 - EDGE_ALPHA);
    rgb.map(|channel| ((u32::from(channel) * keep + 127) / 255) as u8)
}

pub fn minimap(map: &TileMap, x: u32, y: u32) -> [u8; 4] {
    let base = map.at(x, y).color();
    let [r, g, b] = match edge(map, x, y) {
        None => base,
        Some(Edge::Solid(rgb)) => rgb,
        Some(Edge::Shade) => shade(base),
    };
    [r, g, b, 255]
}

pub fn minimap_pixels(map: &TileMap) -> Vec<u8> {
    let mut data = Vec::with_capacity(map.cells() * 4);

    for y in 0..map.height {
        for x in 0..map.width {
            data.extend_from_slice(&minimap(map, x, y));
        }
    }

    data
}

pub fn pick(x: u32, y: u32, count: u8) -> u8 {
    let mut h = x.wrapping_mul(0x27d4_eb2d) ^ y.wrapping_mul(0x1656_67b1);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    (h % u32::from(count.max(1))) as u8
}

pub fn variant(map: &TileMap, x: u32, y: u32) -> u8 {
    let here = look::of(map.at(x, y));

    if let Some(frame) = here.edge_frame
        && let Some(above) = up(map, x, y)
        && !above.is(here)
    {
        return frame;
    }

    pick(x, y, here.variants)
}

pub fn water_around(map: &TileMap, x: u32, y: u32) -> bool {
    if x == 0 || y == 0 || x + 1 >= map.width || y + 1 >= map.height {
        return true;
    }

    [(x, y + 1), (x, y - 1), (x - 1, y), (x + 1, y)]
        .into_iter()
        .any(|(nx, ny)| look::of(map.at(nx, ny)).liquid)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Border {
    Pit,
    Water,
}

pub fn border(map: &TileMap, x: u32, y: u32) -> Option<Border> {
    let main = look::main(map.at(x, y));

    if !(main.ground || main.block) || main.fills_with_ocean {
        return None;
    }

    if y + 1 < map.height && look::main(map.at(x, y + 1)).fills_with_ocean {
        return Some(Border::Pit);
    }

    water_around(map, x, y).then_some(Border::Water)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Area {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Area {
    pub fn tiles(map: &TileMap, x: u32, y: u32, width: u32, height: u32) -> Self {
        let x1 = (x + width).min(map.width);
        let y1 = (y + height).min(map.height);
        Self {
            x: x * PX,
            y: y * PX,
            width: x1.saturating_sub(x) * PX,
            height: y1.saturating_sub(y) * PX,
        }
    }

    fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x as i32
            && y >= self.y as i32
            && x < (self.x + self.width) as i32
            && y < (self.y + self.height) as i32
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Canvas {
    pub area: Area,
    pub color: Vec<[u8; 4]>,
    pub rim: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ink {
    Tile,
    Rim,
    Object,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Thing {
    Wall(&'static str),
    Flora(Flora, [u8; 3], u8),
    Ore(Ore, u8),
}

enum Art {
    Sheet(&'static Pixmap),
    Thing(Thing),
}

struct Stamp {
    pass: u8,
    z: u8,
    tx: u32,
    ty: u32,
    ink: Ink,
    art: Art,
}

impl Canvas {
    fn new(area: Area) -> Self {
        let size = (area.width * area.height) as usize;
        Self {
            area,
            color: vec![art::CLEAR; size],
            rim: vec![0; size],
        }
    }

    pub fn at(&self, x: u32, y: u32) -> [u8; 4] {
        self.color[self.index(x, y)]
    }

    pub fn rim_at(&self, x: u32, y: u32) -> u8 {
        self.rim[self.index(x, y)]
    }

    fn index(&self, x: u32, y: u32) -> usize {
        ((y - self.area.y) * self.area.width + (x - self.area.x)) as usize
    }

    fn stamp(&mut self, sprite: &Pixmap, tx: u32, ty: u32, ink: Ink) {
        let origin_x = (tx * PX) as i32 - i32::from(sprite.left);
        let origin_y = (ty * PX) as i32 - i32::from(sprite.top);

        for sy in 0..sprite.height {
            for sx in 0..sprite.width {
                let pixel = sprite.at(sx, sy);
                if pixel[3] == 0 {
                    continue;
                }

                let x = origin_x + i32::from(sx);
                let y = origin_y + i32::from(sy);
                if !self.area.contains(x, y) {
                    continue;
                }

                let at = self.index(x as u32, y as u32);
                match ink {
                    Ink::Tile => {
                        self.color[at] = pixel;
                        self.rim[at] = 0;
                    }
                    Ink::Rim => self.rim[at] = self.rim[at].saturating_add(1),
                    Ink::Object => {
                        self.color[at] = over(pixel, self.color[at]);
                        if pixel[3] == 255 {
                            self.rim[at] = 0;
                        }
                    }
                }
            }
        }
    }

    pub fn rgba(&self) -> Vec<u8> {
        self.color.iter().flatten().copied().collect()
    }
}

pub fn over(top: [u8; 4], under: [u8; 4]) -> [u8; 4] {
    if top[3] == 255 || under[3] == 0 {
        return top;
    }

    let a = f32::from(top[3]) / 255.0;
    let b = f32::from(under[3]) / 255.0 * (1.0 - a);
    let alpha = a + b;
    let channel =
        |i: usize| ((f32::from(top[i]) * a + f32::from(under[i]) * b) / alpha).round() as u8;
    [
        channel(0),
        channel(1),
        channel(2),
        (alpha * 255.0).round() as u8,
    ]
}

fn things(tile: Tile, tx: u32, ty: u32) -> Vec<Thing> {
    let mut found = Vec::new();

    if let Some(wall) = tile.wall {
        found.push(Thing::Wall(look::wall(wall).id));
    }
    if let Some(ore) = tile.ore {
        found.push(Thing::Ore(ore, pick(tx, ty, 3)));
    }
    if let Some(flora) = tile.flora {
        found.push(Thing::Flora(flora, tile.cover(), pick(ty, tx, 3)));
    }

    found
}

fn render_thing(thing: Thing) -> Pixmap {
    match thing {
        Thing::Wall(id) => art::wall_block(
            econbox_tile::WALLS
                .into_iter()
                .map(look::wall)
                .find(|look| look.id == id)
                .unwrap_or(&look::WALL_ORDER),
        ),
        Thing::Flora(flora, cover, variant) => art::flora(flora, cover, variant),
        Thing::Ore(ore, variant) => art::deposit(ore, variant),
    }
}

fn stamps(map: &TileMap, tx: u32, ty: u32, out: &mut Vec<Stamp>) {
    let tile = map.at(tx, ty);
    let here = look::of(tile);

    if here.z > 0
        && let Some(sprite) = art::sprites(here).get(usize::from(variant(map, tx, ty)))
    {
        out.push(Stamp {
            pass: 0,
            z: here.z,
            tx,
            ty,
            ink: Ink::Tile,
            art: Art::Sheet(sprite),
        });
    }

    match border(map, tx, ty) {
        Some(Border::Pit) => out.push(Stamp {
            pass: 0,
            z: look::BORDER_PIT.z,
            tx,
            ty,
            ink: Ink::Tile,
            art: Art::Sheet(&art::sprites(&look::BORDER_PIT)[0]),
        }),
        Some(Border::Water) => {
            out.push(Stamp {
                pass: 0,
                z: look::BORDER_WATER.z,
                tx,
                ty,
                ink: Ink::Tile,
                art: Art::Sheet(&art::sprites(&look::BORDER_WATER)[0]),
            });
            out.push(Stamp {
                pass: 0,
                z: look::BORDER_WATER_RUNUP.z,
                tx,
                ty,
                ink: Ink::Rim,
                art: Art::Sheet(&art::sprites(&look::BORDER_WATER_RUNUP)[0]),
            });
        }
        None => {}
    }

    for thing in things(tile, tx, ty) {
        out.push(Stamp {
            pass: 1,
            z: 0,
            tx,
            ty,
            ink: Ink::Object,
            art: Art::Thing(thing),
        });
    }
}

pub const REACH: u32 = 2;

pub fn compose(map: &TileMap, area: Area) -> Canvas {
    let mut canvas = Canvas::new(area);

    if area.width == 0 || area.height == 0 {
        return canvas;
    }

    let tx0 = (area.x / PX).saturating_sub(REACH);
    let ty0 = (area.y / PX).saturating_sub(REACH);
    let tx1 = ((area.x + area.width - 1) / PX + REACH).min(map.width - 1);
    let ty1 = ((area.y + area.height - 1) / PX + REACH).min(map.height - 1);

    let mut list = Vec::new();
    for ty in ty0..=ty1 {
        for tx in tx0..=tx1 {
            stamps(map, tx, ty, &mut list);
        }
    }

    list.sort_by_key(|stamp| (stamp.pass, stamp.z, stamp.ty, Reverse(stamp.tx)));

    let mut made: HashMap<Thing, Pixmap> = HashMap::new();
    for stamp in &list {
        match stamp.art {
            Art::Sheet(sprite) => canvas.stamp(sprite, stamp.tx, stamp.ty, stamp.ink),
            Art::Thing(thing) => {
                let sprite = made
                    .entry(thing)
                    .or_insert_with(|| render_thing(thing))
                    .clone();
                canvas.stamp(&sprite, stamp.tx, stamp.ty, stamp.ink);
            }
        }
    }

    canvas
}

#[cfg(test)]
mod tests {
    use super::*;
    use econbox_tile::{Ground, Overlay, Wall};

    fn strip(column: &[Tile]) -> TileMap {
        let mut map = TileMap::filled(1, column.len() as u32, column[0]);
        for (y, tile) in column.iter().enumerate() {
            map.set(0, y as u32, *tile);
        }
        map
    }

    fn bare(ground: Ground) -> Tile {
        Tile::bare(ground)
    }

    #[test]
    fn hills_over_soft_ground_wear_the_hill_edge() {
        let map = strip(&[bare(Ground::Hills), bare(Ground::SoilHigh)]);

        assert_eq!(edge(&map, 0, 0), Some(Edge::Solid(EDGE_HILLS)));
    }

    #[test]
    fn mountains_over_hills_and_rock_over_soil_wear_the_mountain_edge() {
        let ridge = strip(&[bare(Ground::Mountains), bare(Ground::Hills)]);
        let cliff = strip(&[bare(Ground::Summit), bare(Ground::Sand)]);

        assert_eq!(edge(&ridge, 0, 0), Some(Edge::Solid(EDGE_MOUNTAIN)));
        assert_eq!(edge(&cliff, 0, 0), Some(Edge::Solid(EDGE_MOUNTAIN)));
    }

    #[test]
    fn a_summit_over_mountains_is_only_a_shade_darker() {
        let map = strip(&[bare(Ground::Summit), bare(Ground::Mountains)]);

        assert_eq!(edge(&map, 0, 0), Some(Edge::Shade));
    }

    #[test]
    fn higher_ground_over_lower_ground_casts_a_ten_percent_shade() {
        let steps = strip(&[
            bare(Ground::SoilHigh),
            bare(Ground::SoilLow),
            bare(Ground::Sand),
        ]);
        let meadow = strip(&[
            Tile::new(Ground::SoilHigh, Overlay::Grass),
            Tile::new(Ground::SoilLow, Overlay::Grass),
        ]);
        let flat = strip(&[bare(Ground::SoilLow), bare(Ground::SoilLow)]);

        assert_eq!(edge(&steps, 0, 0), Some(Edge::Shade));
        assert_eq!(edge(&steps, 0, 1), Some(Edge::Shade));
        assert_eq!(edge(&meadow, 0, 0), Some(Edge::Shade));
        assert_eq!(edge(&flat, 0, 0), None);
    }

    #[test]
    fn enchanted_high_ground_keeps_the_low_height_and_casts_no_shade() {
        let map = strip(&[
            Tile::new(Ground::SoilHigh, Overlay::Enchanted),
            Tile::new(Ground::SoilLow, Overlay::Enchanted),
        ]);

        assert_eq!(edge(&map, 0, 0), None);
    }

    #[test]
    fn the_sea_right_under_the_land_shows_the_dark_coast_line() {
        let map = strip(&[bare(Ground::Sand), bare(Ground::ShallowWaters)]);

        assert_eq!(edge(&map, 0, 1), Some(Edge::Solid(EDGE_OCEAN)));
        assert_eq!(edge(&map, 0, 0), None);
    }

    #[test]
    fn mountains_are_no_ground_so_the_sea_under_them_has_no_coast_line() {
        let map = strip(&[bare(Ground::Mountains), bare(Ground::ShallowWaters)]);

        assert_eq!(edge(&map, 0, 1), None);
    }

    #[test]
    fn deeper_water_under_shallower_water_is_shaded() {
        let map = strip(&[
            bare(Ground::ShallowWaters),
            bare(Ground::CloseOcean),
            bare(Ground::DeepOcean),
        ]);

        assert_eq!(edge(&map, 0, 1), Some(Edge::Shade));
        assert_eq!(edge(&map, 0, 2), Some(Edge::Shade));
        assert_eq!(edge(&map, 0, 0), None);
    }

    #[test]
    fn a_wall_paints_its_edge_colour_on_the_tile_below_and_none_on_itself() {
        let walled = Tile::new(Ground::SoilLow, Overlay::Grass).with_wall(Some(Wall::Iron));
        let map = strip(&[walled, Tile::new(Ground::SoilLow, Overlay::Grass)]);

        assert_eq!(edge(&map, 0, 1), Some(Edge::Solid([0x2e, 0x36, 0x3e])));
        assert_eq!(edge(&map, 0, 0), None);
    }

    #[test]
    fn the_minimap_pixel_is_the_tile_colour_under_its_edge() {
        let map = strip(&[
            bare(Ground::SoilHigh),
            bare(Ground::SoilLow),
            bare(Ground::SoilLow),
        ]);
        let [r, g, b] = Ground::SoilHigh.color();

        assert_eq!(minimap(&map, 0, 2), {
            let [r, g, b] = Ground::SoilLow.color();
            [r, g, b, 255]
        });
        assert_eq!(minimap(&map, 0, 0), {
            let [r, g, b] = shade([r, g, b]);
            [r, g, b, 255]
        });
        assert_eq!(shade([255, 255, 255]), [229, 229, 229]);
    }

    #[test]
    fn mountains_take_the_edge_frame_under_a_different_tile() {
        let map = strip(&[
            bare(Ground::SoilLow),
            bare(Ground::Mountains),
            bare(Ground::Mountains),
        ]);

        assert_eq!(variant(&map, 0, 1), 2);
        assert_eq!(variant(&map, 0, 2), pick(0, 2, 3));
    }

    #[test]
    fn land_next_to_water_or_lava_or_the_world_edge_gets_the_coast_border() {
        let mut map = TileMap::filled(5, 5, bare(Ground::SoilLow));

        assert_eq!(border(&map, 2, 2), None);
        assert_eq!(border(&map, 0, 2), Some(Border::Water));

        map.set(3, 2, bare(Ground::Lava3));
        assert_eq!(border(&map, 2, 2), Some(Border::Water));

        map.set(3, 2, bare(Ground::ShallowWaters));
        assert_eq!(border(&map, 2, 2), Some(Border::Water));
        assert_eq!(border(&map, 3, 2), None);
    }

    #[test]
    fn lava_takes_no_coast_border_of_its_own() {
        let map = TileMap::filled(3, 3, bare(Ground::Lava2));

        assert_eq!(border(&map, 1, 1), None);
    }

    fn sea_with_island() -> TileMap {
        let mut map = TileMap::filled(8, 8, bare(Ground::CloseOcean));
        for y in 3..5 {
            for x in 3..5 {
                map.set(x, y, bare(Ground::Sand));
            }
        }
        map
    }

    #[test]
    fn the_canvas_is_four_pixels_per_tile() {
        let map = sea_with_island();
        let canvas = compose(&map, Area::tiles(&map, 0, 0, 8, 8));

        assert_eq!((canvas.area.width, canvas.area.height), (32, 32));
        assert_eq!(canvas.color.len(), 32 * 32);
    }

    #[test]
    fn the_island_carries_the_coast_outline_and_a_rim_on_the_water() {
        let map = sea_with_island();
        let canvas = compose(&map, Area::tiles(&map, 0, 0, 8, 8));
        let outline = art::sprites(&look::BORDER_WATER)[0].at(5, 5);
        let sand = art::sprites(&look::SAND)[usize::from(pick(3, 3, 3))].at(3, 3);

        assert_eq!(canvas.at(13, 13), sand);
        assert_eq!(canvas.rim_at(13, 13), 0);

        assert_eq!(canvas.at(10, 14), outline);
        assert!(canvas.rim_at(10, 14) > 0);
        assert!(canvas.rim_at(9, 14) > 0);
        assert_eq!(canvas.rim_at(4, 14), 0);
    }

    #[test]
    fn the_sand_bleeds_one_pixel_into_the_sea_on_every_side() {
        let map = sea_with_island();
        let canvas = compose(&map, Area::tiles(&map, 0, 0, 8, 8));
        let sand = look::SAND.color();
        let is_sand = |pixel: [u8; 4]| (0..3).all(|i| pixel[i].abs_diff(sand[i]) < 35);

        assert!(is_sand(canvas.at(11, 13)));
        assert!(is_sand(canvas.at(20, 13)));
        assert!(is_sand(canvas.at(13, 11)));
        assert!(is_sand(canvas.at(13, 20)));
        assert!(!is_sand(canvas.at(11, 11)));
    }

    #[test]
    fn a_higher_layer_wins_where_two_tiles_overlap() {
        let mut map = TileMap::filled(4, 1, bare(Ground::Sand));
        map.set(2, 0, Tile::new(Ground::SoilLow, Overlay::Grass));
        let canvas = compose(&map, Area::tiles(&map, 0, 0, 4, 1));
        let grass = Overlay::Grass.color(false).unwrap();
        let near = |pixel: [u8; 4]| (0..3).all(|i| pixel[i].abs_diff(grass[i]) < 40);

        assert!(near(canvas.at(7, 2)), "{:?}", canvas.at(7, 2));
        assert!(near(canvas.at(12, 2)), "{:?}", canvas.at(12, 2));
    }

    #[test]
    fn deep_ocean_is_left_clear_for_the_sea_behind_it() {
        let map = TileMap::filled(4, 4, bare(Ground::DeepOcean));
        let canvas = compose(&map, Area::tiles(&map, 0, 0, 4, 4));

        assert!(canvas.color.iter().all(|pixel| pixel[3] == 0));
        assert!(canvas.rim.iter().all(|&rim| rim == 0));
    }

    #[test]
    fn a_part_of_the_canvas_matches_the_whole() {
        let mut map = sea_with_island();
        map.set(5, 5, bare(Ground::Mountains));
        map.set(
            4,
            3,
            Tile::new(Ground::SoilLow, Overlay::Grass).with_flora(Some(Flora::Tree)),
        );
        let whole = compose(&map, Area::tiles(&map, 0, 0, 8, 8));
        let part = compose(&map, Area::tiles(&map, 2, 2, 4, 4));

        for y in 8..24 {
            for x in 8..24 {
                assert_eq!(part.at(x, y), whole.at(x, y), "{x},{y}");
                assert_eq!(part.rim_at(x, y), whole.rim_at(x, y), "{x},{y}");
            }
        }
    }

    #[test]
    fn objects_sit_on_top_and_hide_the_rim() {
        let mut map = sea_with_island();
        map.set(3, 3, bare(Ground::Sand).with_wall(Some(Wall::Light)));
        let canvas = compose(&map, Area::tiles(&map, 0, 0, 8, 8));
        let block = art::wall_block(&look::WALL_LIGHT);

        assert_eq!(canvas.at(13, 15), block.at(3, 7));
        assert_eq!(canvas.at(13, 9), block.at(3, 1));
    }

    #[test]
    fn transparent_ink_blends_over_what_is_under_it() {
        assert_eq!(over([0, 0, 0, 255], [9, 9, 9, 255]), [0, 0, 0, 255]);
        assert_eq!(over([255, 0, 0, 0], [9, 9, 9, 255])[3], 255);
        let half = over([200, 200, 200, 128], [0, 0, 0, 255]);
        assert!(half[0] > 95 && half[0] < 105 && half[3] == 255, "{half:?}");
    }
}
