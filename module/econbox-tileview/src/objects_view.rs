use crate::look;
use crate::objects::{self, Placed, Sprite, WALL_FRAMES, WALL_SPEED};
use crate::quality::MapMode;
use crate::thing_art;
use crate::things::{self, Wind};
use crate::view::{self, Drawn, nearest, tile_centre};
use bevy::asset::{AssetPath, RenderAssetUsages, embedded_path};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, Extent3d, TextureDimension, TextureFormat};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d};
use econbox_camera::{TILE_SIZE, WorldCamera};
use econbox_clock::WorldClock;
use econbox_tile::WALLS;
use econbox_tilemap::{CHUNK, Painted, TileMap};
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

pub const OBJECTS_LAYER: f32 = 0.3;
pub const ATLAS_WIDTH: u32 = 4096;
pub const LEAVES: [f32; 4] = [1.81, 0.026, 0.002, 1.4];
pub const CELESTIAL: [f32; 4] = [0.7, 0.026, 0.002, 1.4];
pub const ANIMATION_WRAP: f32 = 100.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub frames: u32,
}

pub type Key = (&'static str, usize, bool);

pub struct Atlas {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub entries: HashMap<Key, Entry>,
}

fn sheets() -> Vec<(Key, Vec<crate::art::Pixmap>)> {
    let mut all = Vec::new();
    for kind in things::KINDS {
        for (index, variant) in kind.variants.iter().enumerate() {
            let frames = (0..u32::from(variant.frames.max(1)))
                .map(|frame| thing_art::frame(kind, index, frame, false))
                .collect();
            all.push(((kind.id, index, false), frames));
        }
        for (index, _) in kind.bare.iter().enumerate() {
            all.push((
                (kind.id, index, true),
                vec![thing_art::frame(kind, index, 0, true)],
            ));
        }
    }
    for wall in WALLS {
        let looked = look::wall(wall);
        let edge = looked.edge.unwrap_or(looked.color());
        let frames = (0..WALL_FRAMES)
            .map(|frame| thing_art::wall_frame(looked.color(), edge, frame))
            .collect();
        all.push(((looked.id, 0, false), frames));
    }
    all
}

pub fn atlas() -> &'static Atlas {
    static ATLAS: OnceLock<Atlas> = OnceLock::new();
    ATLAS.get_or_init(|| {
        let sheets = sheets();
        let mut entries = HashMap::new();
        let (mut x, mut y, mut row) = (0_u32, 0_u32, 0_u32);
        let mut placed = Vec::new();

        for (key, frames) in &sheets {
            let width = u32::from(frames[0].width);
            let height = u32::from(frames[0].height);
            let strip = width * frames.len() as u32;
            if x + strip > ATLAS_WIDTH {
                x = 0;
                y += row + 1;
                row = 0;
            }
            entries.insert(
                *key,
                Entry {
                    x,
                    y,
                    width,
                    height,
                    frames: frames.len() as u32,
                },
            );
            placed.push((x, y, key));
            x += strip + 1;
            row = row.max(height);
        }

        let height = y + row + 1;
        let mut pixels = vec![0_u8; (ATLAS_WIDTH * height * 4) as usize];
        for ((left, top, _), (_, frames)) in placed.iter().zip(&sheets) {
            for (index, frame) in frames.iter().enumerate() {
                let fx = left + index as u32 * u32::from(frame.width);
                for py in 0..u32::from(frame.height) {
                    for px in 0..u32::from(frame.width) {
                        let pixel = frame.at(px as u8, py as u8);
                        let at = (((top + py) * ATLAS_WIDTH + fx + px) * 4) as usize;
                        pixels[at..at + 4].copy_from_slice(&pixel);
                    }
                }
            }
        }

        Atlas {
            width: ATLAS_WIDTH,
            height,
            pixels,
            entries,
        }
    })
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ObjectMaterial {
    #[uniform(0)]
    pub wind: WindUniform,
    #[texture(1)]
    #[sampler(2)]
    pub atlas: Handle<Image>,
}

#[derive(bevy::render::render_resource::ShaderType, Debug, Clone, Copy, PartialEq)]
pub struct WindUniform {
    pub clock: Vec4,
    pub leaves: Vec4,
    pub celestial: Vec4,
    pub tint: Vec4,
}

impl Default for WindUniform {
    fn default() -> Self {
        Self {
            clock: Vec4::ZERO,
            leaves: Vec4::from_array(LEAVES),
            celestial: Vec4::from_array(CELESTIAL),
            tint: Vec4::ONE,
        }
    }
}

impl Material2d for ObjectMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("objects.wgsl")).with_source("embedded"),
        )
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq)]
pub struct ObjectClock {
    pub wind: f32,
    pub animation: f32,
}

impl ObjectClock {
    pub fn advance(&mut self, seconds: f32) {
        self.wind += seconds;
        self.animation = (self.animation + seconds) % ANIMATION_WRAP;
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectChunk {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone)]
pub struct Built {
    pub entity: Entity,
    pub mesh: Handle<Mesh>,
    pub tween: f32,
}

#[derive(Resource, Debug, Default)]
pub struct ObjectsDrawn {
    pub epoch: u64,
    pub atlas: Option<Handle<Image>>,
    pub material: Option<Handle<ObjectMaterial>>,
    pub chunks: HashMap<(u32, u32), Built>,
    pub dirty: HashSet<(u32, u32)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quad {
    pub corners: [Vec2; 4],
    pub uvs: [Vec2; 4],
    pub color: Vec4,
    pub tangent: Vec4,
}

pub fn quad(map: &TileMap, placed: &Placed, tween: f32, entry: &Entry) -> Quad {
    let variant = placed.variant();
    let size = TILE_SIZE / 4.0 * tween;
    let anchor = tile_centre(map, placed.x as f32 + 0.5, placed.y as f32 + 0.5);
    let w = entry.width as f32;
    let h = entry.height as f32;
    let left = if placed.flip {
        anchor.x - (w - variant.pivot_x) * size
    } else {
        anchor.x - variant.pivot_x * size
    };
    let bottom = anchor.y - variant.pivot_y * size;
    let right = left + w * size;
    let top = bottom + h * size;
    let (u0, u1) = if placed.flip {
        (w + 1.0, 1.0)
    } else {
        (1.0, w + 1.0)
    };

    let (mode, fps, offset) = match placed.sprite {
        Sprite::Kind { kind, .. } => (
            objects::wind_mode(Some(kind)),
            kind.speed,
            ((placed.seed % 1000) * 100) as f32,
        ),
        Sprite::Wall(wall) => {
            let animated = look::wall(wall).id == "wall_light";
            (
                0,
                if animated { WALL_SPEED } else { 0.0 },
                (placed.seed % WALL_FRAMES) as f32,
            )
        }
    };
    let phase = (placed.x as f32 + 0.5) + ((map.height - 1 - placed.y) as f32 + 0.5);

    Quad {
        corners: [
            Vec2::new(left, bottom),
            Vec2::new(right, bottom),
            Vec2::new(right, top),
            Vec2::new(left, top),
        ],
        uvs: [
            Vec2::new(u0, 1.0),
            Vec2::new(u1, 1.0),
            Vec2::new(u1, h + 1.0),
            Vec2::new(u0, h + 1.0),
        ],
        color: Vec4::new(entry.x as f32, entry.y as f32, w, h),
        tangent: Vec4::new(phase, (mode + entry.frames * 4) as f32, fps, offset),
    }
}

pub fn entry_for(placed: &Placed) -> Option<Entry> {
    let key: Key = match placed.sprite {
        Sprite::Kind {
            kind,
            variant,
            bare,
        } => (kind.id, variant, bare),
        Sprite::Wall(wall) => (look::wall(wall).id, 0, false),
    };
    atlas().entries.get(&key).copied()
}

pub fn chunk_quads(map: &TileMap, cx: u32, cy: u32, tween: f32) -> Vec<Quad> {
    let (width, height) = view::chunk_tiles(map, cx, cy);
    let mut placed: Vec<Placed> = Vec::new();
    for y in cy * CHUNK..cy * CHUNK + height {
        for x in cx * CHUNK..cx * CHUNK + width {
            let tile = map.at(x, y);
            if tile.flora.is_none() && tile.ore.is_none() && tile.wall.is_none() {
                continue;
            }
            placed.extend(objects::objects(tile, x, y));
        }
    }
    placed.sort_by_key(|p| (p.y, p.x));
    placed
        .iter()
        .filter_map(|p| entry_for(p).map(|entry| quad(map, p, tween, &entry)))
        .collect()
}

pub fn mesh_of(quads: &[Quad]) -> Mesh {
    let mut positions = Vec::with_capacity(quads.len() * 4);
    let mut uvs = Vec::with_capacity(quads.len() * 4);
    let mut colors = Vec::with_capacity(quads.len() * 4);
    let mut tangents = Vec::with_capacity(quads.len() * 4);
    let mut indices = Vec::with_capacity(quads.len() * 6);

    for (index, quad) in quads.iter().enumerate() {
        let base = (index * 4) as u32;
        for corner in 0..4 {
            positions.push([quad.corners[corner].x, quad.corners[corner].y, 0.0]);
            uvs.push(quad.uvs[corner].to_array());
            colors.push(quad.color.to_array());
            tangents.push(quad.tangent.to_array());
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_attribute(Mesh::ATTRIBUTE_TANGENT, tangents)
    .with_inserted_indices(Indices::U32(indices))
}

pub fn atlas_image() -> Image {
    let atlas = atlas();
    let mut image = Image::new(
        Extent3d {
            width: atlas.width,
            height: atlas.height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        atlas.pixels.clone(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    nearest(&mut image);
    image
}

pub fn mark(
    mut painted: MessageReader<Painted>,
    mut drawn: ResMut<ObjectsDrawn>,
    map: Option<Res<TileMap>>,
) {
    let Some(map) = map else {
        painted.clear();
        return;
    };
    for message in painted.read() {
        for pos in &message.cells {
            if pos.x < map.width && pos.y < map.height {
                drawn.dirty.insert(pos.chunk());
            }
        }
    }
}

pub fn tick(time: Res<Time>, clock: Option<Res<WorldClock>>, mut objects: ResMut<ObjectClock>) {
    let rate = clock.map(|clock| clock.rate()).unwrap_or(1.0);
    objects.advance(time.delta_secs() * rate);
}

pub fn draw(
    mut commands: Commands,
    mut drawn: ResMut<ObjectsDrawn>,
    tiles: Res<Drawn>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ObjectMaterial>>,
    camera: Option<Single<(&Transform, &Projection), With<WorldCamera>>>,
    mode: Res<MapMode>,
    clock: Res<ObjectClock>,
    mut chunks: Query<&mut Visibility, With<ObjectChunk>>,
    map: Option<Res<TileMap>>,
) {
    if drawn.epoch != tiles.epoch {
        for (_, built) in drawn.chunks.drain() {
            commands.entity(built.entity).despawn();
            meshes.remove(&built.mesh);
        }
        drawn.dirty.clear();
        drawn.epoch = tiles.epoch;
    }

    let tween = mode.tween_buildings();
    let shown = if tween > 0.0 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut visibility in &mut chunks {
        visibility.set_if_neq(shown);
    }

    let material = match drawn.material.clone() {
        Some(material) => material,
        None => {
            let atlas = images.add(atlas_image());
            let material = materials.add(ObjectMaterial {
                wind: WindUniform::default(),
                atlas: atlas.clone(),
            });
            drawn.atlas = Some(atlas);
            drawn.material = Some(material.clone());
            material
        }
    };

    if let Some(mut material) = materials.get_mut(&material) {
        material.wind.clock = Vec4::new(clock.wind, clock.animation, 0.0, 0.0);
    }

    if tween <= 0.0 {
        return;
    }

    let (Some(map), Some(camera)) = (map, camera) else {
        return;
    };
    let (transform, projection) = camera.into_inner();
    let Projection::Orthographic(projection) = projection else {
        return;
    };
    let (x0, y0, x1, y1) = view::visible_tiles(
        &map,
        transform.translation.truncate(),
        projection.area.size(),
    );
    let (cx0, cy0) = ((x0 / CHUNK).saturating_sub(1), y0 / CHUNK);
    let (cx1, cy1) = (
        (x1 / CHUNK + 1).min((map.width - 1) / CHUNK),
        (y1 / CHUNK + 1).min((map.height - 1) / CHUNK),
    );

    for cy in cy0..=cy1 {
        for cx in cx0..=cx1 {
            let key = (cx, cy);
            let stale = match drawn.chunks.get(&key) {
                None => true,
                Some(built) => built.tween != tween || drawn.dirty.contains(&key),
            };
            if !stale {
                continue;
            }
            drawn.dirty.remove(&key);
            let mesh = mesh_of(&chunk_quads(&map, cx, cy, tween));
            match drawn.chunks.get_mut(&key) {
                Some(built) => {
                    if let Some(mut slot) = meshes.get_mut(&built.mesh) {
                        *slot = mesh;
                    }
                    built.tween = tween;
                }
                None => {
                    let handle = meshes.add(mesh);
                    let entity = commands
                        .spawn((
                            ObjectChunk { x: cx, y: cy },
                            Mesh2d(handle.clone()),
                            MeshMaterial2d(material.clone()),
                            Transform::from_xyz(
                                0.0,
                                0.0,
                                OBJECTS_LAYER + cy as f32 * 0.001 + cx as f32 * 0.00001,
                            ),
                        ))
                        .id();
                    drawn.chunks.insert(
                        key,
                        Built {
                            entity,
                            mesh: handle,
                            tween,
                        },
                    );
                }
            }
        }
    }
}

pub fn wind_of(kind: &things::Kind) -> [f32; 4] {
    match kind.wind {
        Wind::Celestial => CELESTIAL,
        _ => LEAVES,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use econbox_tile::{Flora, Ground, Overlay, Tile};

    #[test]
    fn every_sprite_and_frame_fits_in_one_atlas() {
        let atlas = atlas();

        assert!(atlas.height <= 8192, "{}", atlas.height);
        assert!(atlas.entries.len() >= things::KINDS.len());
        for (key, entry) in &atlas.entries {
            assert!(
                entry.x + entry.width * entry.frames <= atlas.width,
                "{key:?}"
            );
        }
        assert_eq!(
            atlas.entries[&("celestial_tree", 0, false)].frames,
            u32::from(things::CELESTIAL_TREE.variants[0].frames)
        );
        assert_eq!(atlas.entries[&("wall_light", 0, false)].frames, WALL_FRAMES);
    }

    #[test]
    fn a_tree_quad_stands_on_its_tile_centre_with_its_pivot() {
        let mut map = TileMap::filled(8, 8, Tile::new(Ground::SoilLow, Overlay::Grass));
        map.set(3, 4, map.at(3, 4).with_flora(Some(Flora::Tree)));
        let placed = objects::objects(map.at(3, 4), 3, 4)[0];
        let entry = entry_for(&placed).unwrap();
        let quad = quad(&map, &placed, 1.0, &entry);
        let anchor = tile_centre(&map, 3.5, 4.5);
        let variant = placed.variant();
        let px = TILE_SIZE / 4.0;

        assert!((quad.corners[0].y - (anchor.y - variant.pivot_y * px)).abs() < 1e-4);
        assert!((quad.corners[2].y - quad.corners[0].y - entry.height as f32 * px).abs() < 1e-4);
        assert!((quad.corners[1].x - quad.corners[0].x - entry.width as f32 * px).abs() < 1e-4);
        assert_eq!(quad.color.z, entry.width as f32);
    }

    #[test]
    fn objects_shrink_to_nothing_with_the_buildings_tween() {
        let mut map = TileMap::filled(8, 8, Tile::new(Ground::SoilLow, Overlay::Grass));
        map.set(3, 4, map.at(3, 4).with_flora(Some(Flora::Bush)));
        let placed = objects::objects(map.at(3, 4), 3, 4)[0];
        let entry = entry_for(&placed).unwrap();
        let quad = quad(&map, &placed, 0.0, &entry);

        assert!(quad.corners.iter().all(|c| *c == quad.corners[0]));
    }

    #[test]
    fn a_chunk_mesh_has_four_corners_per_object_drawn_top_down() {
        let mut map = TileMap::filled(64, 64, Tile::new(Ground::SoilLow, Overlay::Grass));
        for (x, y) in [(10, 20), (5, 3), (40, 3)] {
            map.set(x, y, map.at(x, y).with_flora(Some(Flora::Tree)));
        }
        let quads = chunk_quads(&map, 0, 0, 1.0);

        assert_eq!(quads.len(), 3);
        assert!(quads[0].corners[0].y >= quads[2].corners[0].y);
        let mesh = mesh_of(&quads);
        assert_eq!(mesh.count_vertices(), 12);
    }

    #[test]
    fn the_wind_numbers_are_worldbox_tree_material_values() {
        assert_eq!(wind_of(&things::TREE_GREEN_1), [1.81, 0.026, 0.002, 1.4]);
        assert_eq!(wind_of(&things::CELESTIAL_TREE)[0], 0.7);
    }

    #[test]
    fn the_animation_clock_wraps_at_one_hundred() {
        let mut clock = ObjectClock::default();
        clock.advance(150.0);

        assert_eq!(clock.wind, 150.0);
        assert!((clock.animation - 50.0).abs() < 1e-4);
    }
}
