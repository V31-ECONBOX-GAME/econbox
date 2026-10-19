use crate::compose::{self, Area, Canvas};
use crate::look;
use crate::quality::{MapMode, Runup, SWITCH_GREY};
use bevy::asset::{AssetPath, RenderAssetUsages, embedded_path};
use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, Extent3d, TextureDimension, TextureFormat};
use bevy::shader::ShaderRef;
use bevy::sprite_render::Material2d;
use bevy::tasks::{ComputeTaskPool, TaskPool};
use econbox_camera::{TILE_SIZE, WorldCamera};
use econbox_clock::WorldClock;
use econbox_tilemap::{CHUNK, Painted, TileMap};
use std::collections::{HashMap, HashSet};

pub const TILES_LAYER: f32 = 0.0;
pub const MAP_LAYER: f32 = 0.1;
pub const FLASH_LAYER: f32 = 0.2;
pub const BLOCK: u32 = 8;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ChunkMaterial {
    #[uniform(0)]
    pub rim: Vec4,
    #[uniform(1)]
    pub ocean: Vec4,
    #[texture(2)]
    #[sampler(3)]
    pub tiles: Handle<Image>,
    #[texture(4)]
    #[sampler(5)]
    pub runup: Handle<Image>,
    #[uniform(6)]
    pub shadow: Vec4,
}

impl Material2d for ChunkMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("chunk.wgsl")).with_source("embedded"),
        )
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct MapLayer;

#[derive(Component, Debug, Clone, Copy)]
pub struct SwitchFlash;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileChunk {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone)]
pub struct ChunkView {
    pub entity: Entity,
    pub tiles: Handle<Image>,
    pub runup: Handle<Image>,
    pub material: Handle<ChunkMaterial>,
}

#[derive(Resource, Debug, Default)]
pub struct Drawn {
    pub width: u32,
    pub height: u32,
    pub minimap: Option<Handle<Image>>,
    pub quad: Option<Handle<Mesh>>,
    pub chunks: HashMap<(u32, u32), ChunkView>,
    pub dirty: HashSet<(u32, u32)>,
    pub epoch: u64,
}

pub fn nearest(image: &mut Image) {
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        mipmap_filter: ImageFilterMode::Nearest,
        ..default()
    });
}

fn image(width: u32, height: u32, format: TextureFormat, data: Vec<u8>) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        format,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    nearest(&mut image);
    image
}

pub fn minimap_image(map: &TileMap) -> Image {
    image(
        map.width,
        map.height,
        TextureFormat::Rgba8UnormSrgb,
        compose::minimap_pixels(map),
    )
}

pub fn world_size(map: &TileMap) -> Vec2 {
    Vec2::new(map.width as f32, map.height as f32) * TILE_SIZE
}

pub fn tile_centre(map: &TileMap, x: f32, y: f32) -> Vec2 {
    let half = world_size(map) * 0.5;
    Vec2::new(-half.x + x * TILE_SIZE, half.y - y * TILE_SIZE)
}

pub fn chunk_tiles(map: &TileMap, x: u32, y: u32) -> (u32, u32) {
    (
        CHUNK.min(map.width - x * CHUNK),
        CHUNK.min(map.height - y * CHUNK),
    )
}

pub fn chunk_area(map: &TileMap, x: u32, y: u32) -> Area {
    let (width, height) = chunk_tiles(map, x, y);
    Area::tiles(map, x * CHUNK, y * CHUNK, width, height)
}

pub fn block_area(map: &TileMap, x: u32, y: u32) -> Area {
    Area::tiles(map, x * BLOCK, y * BLOCK, BLOCK, BLOCK)
}

pub fn write(target: &mut Image, origin: Area, canvas: &Canvas, channels: usize) {
    let width = target.width() as usize;
    let Some(data) = target.data.as_mut() else {
        return;
    };
    let dx = (canvas.area.x - origin.x) as usize;
    let dy = (canvas.area.y - origin.y) as usize;
    let row = canvas.area.width as usize;

    for y in 0..canvas.area.height as usize {
        let from = y * row;
        let to = ((dy + y) * width + dx) * channels;

        if channels == 4 {
            for (i, pixel) in canvas.color[from..from + row].iter().enumerate() {
                data[to + i * 4..to + i * 4 + 4].copy_from_slice(pixel);
            }
        } else {
            for i in 0..row {
                data[to + i * 2] = canvas.rim[from + i];
                data[to + i * 2 + 1] = canvas.shade[from + i];
            }
        }
    }
}

pub fn chunk_images(canvas: &Canvas) -> (Image, Image) {
    let Area { width, height, .. } = canvas.area;
    (
        image(width, height, TextureFormat::Rgba8Unorm, canvas.rgba()),
        image(
            width,
            height,
            TextureFormat::Rg8Unorm,
            canvas
                .rim
                .iter()
                .zip(&canvas.shade)
                .flat_map(|(&rim, &shade)| [rim, shade])
                .collect(),
        ),
    )
}

pub fn dirty_blocks(map: &TileMap, x: u32, y: u32) -> impl Iterator<Item = (u32, u32)> {
    let (wide, tall) = compose::shadow_reach();
    let x0 = x.saturating_sub(2.max(wide)) / BLOCK;
    let y0 = y.saturating_sub(3.max(tall)) / BLOCK;
    let x1 = (x + 2.max(wide)).min(map.width - 1) / BLOCK;
    let y1 = (y + 2).min(map.height - 1) / BLOCK;
    (y0..=y1).flat_map(move |by| (x0..=x1).map(move |bx| (bx, by)))
}

pub fn visible_tiles(map: &TileMap, centre: Vec2, size: Vec2) -> (u32, u32, u32, u32) {
    let half = world_size(map) * 0.5;
    let left = (centre.x - size.x * 0.5 + half.x) / TILE_SIZE;
    let right = (centre.x + size.x * 0.5 + half.x) / TILE_SIZE;
    let top = (half.y - (centre.y + size.y * 0.5)) / TILE_SIZE;
    let bottom = (half.y - (centre.y - size.y * 0.5)) / TILE_SIZE;
    let clamp =
        |value: f32, limit: u32| (value.floor().max(0.0) as u32).min(limit.saturating_sub(1));
    (
        clamp(left - 2.0, map.width),
        clamp(top - 2.0, map.height),
        clamp(right + 2.0, map.width),
        clamp(bottom + 2.0, map.height),
    )
}

pub fn spawn_layers(mut commands: Commands) {
    commands.spawn((
        MapLayer,
        Sprite::default(),
        Transform::from_xyz(0.0, 0.0, MAP_LAYER),
    ));
    commands.spawn((
        SwitchFlash,
        Sprite {
            color: Color::srgba(SWITCH_GREY, SWITCH_GREY, SWITCH_GREY, 0.0),
            custom_size: Some(Vec2::splat(10_000.0 * TILE_SIZE)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, FLASH_LAYER),
        Visibility::Hidden,
    ));
}

pub fn refresh(
    mut commands: Commands,
    mut asked: MessageReader<crate::Regenerate>,
    mut drawn: ResMut<Drawn>,
    mut images: ResMut<Assets<Image>>,
    mut layers: Query<&mut Sprite, With<MapLayer>>,
    map: Option<Res<TileMap>>,
) {
    let regenerated = asked.read().count() > 0;
    let Some(map) = map else { return };

    let fresh = drawn.minimap.is_none()
        || regenerated
        || map.is_added()
        || drawn.width != map.width
        || drawn.height != map.height;

    if !fresh {
        return;
    }

    for (_, chunk) in drawn.chunks.drain() {
        commands.entity(chunk.entity).despawn();
        images.remove(&chunk.tiles);
        images.remove(&chunk.runup);
    }
    drawn.dirty.clear();
    drawn.epoch += 1;
    drawn.width = map.width;
    drawn.height = map.height;

    let pixels = minimap_image(&map);
    let handle = match drawn.minimap.clone() {
        Some(handle) if images.get(&handle).is_some() => {
            if let Some(mut image) = images.get_mut(&handle) {
                *image = pixels;
            }
            handle
        }
        _ => images.add(pixels),
    };
    drawn.minimap = Some(handle.clone());

    for mut sprite in &mut layers {
        sprite.image = handle.clone();
        sprite.custom_size = Some(world_size(&map));
    }
}

pub fn repaint(
    mut painted: MessageReader<Painted>,
    mut drawn: ResMut<Drawn>,
    mut images: ResMut<Assets<Image>>,
    map: Option<Res<TileMap>>,
) {
    let cells: Vec<_> = painted
        .read()
        .flat_map(|message| message.cells.iter().copied())
        .collect();

    if cells.is_empty() {
        return;
    }

    let Some(map) = map else { return };

    if let Some(handle) = drawn.minimap.clone()
        && let Some(mut image) = images.get_mut(&handle)
        && let Some(data) = image.data.as_mut()
    {
        for pos in &cells {
            if pos.x >= map.width || pos.y >= map.height {
                continue;
            }

            for y in pos.y.saturating_sub(1)..=(pos.y + 1).min(map.height - 1) {
                let at = (y * map.width + pos.x) as usize * 4;
                if at + 4 <= data.len() {
                    data[at..at + 4].copy_from_slice(&compose::minimap(&map, pos.x, y));
                }
            }
        }
    }

    for pos in cells {
        if pos.x >= map.width || pos.y >= map.height {
            continue;
        }

        for (bx, by) in dirty_blocks(&map, pos.x, pos.y) {
            let chunk = (bx * BLOCK / CHUNK, by * BLOCK / CHUNK);
            if drawn.chunks.contains_key(&chunk) {
                drawn.dirty.insert((bx, by));
            }
        }
    }
}

pub fn switch(
    camera: Option<Single<&Projection, With<WorldCamera>>>,
    time: Res<Time>,
    options: Res<crate::sleek::ScreenOptions>,
    mut mode: ResMut<MapMode>,
    mut layers: Query<(&mut Sprite, &mut Visibility), (With<MapLayer>, Without<SwitchFlash>)>,
    mut flashes: Query<(&mut Sprite, &mut Visibility), (With<SwitchFlash>, Without<MapLayer>)>,
    mut chunks: Query<&mut Visibility, (With<TileChunk>, Without<MapLayer>, Without<SwitchFlash>)>,
) {
    if let Some(camera) = camera
        && let Projection::Orthographic(projection) = camera.into_inner()
    {
        let area = projection.area.size();
        if area.y > 0.0 {
            mode.set_zoom(area.y * 0.5 / TILE_SIZE, area.x, area.y);
        }
    }

    mode.advance(time.delta_secs());
    if !options.transition {
        mode.snap();
    }

    for (mut sprite, mut visibility) in &mut layers {
        sprite.color = Color::srgba(1.0, 1.0, 1.0, mode.map_alpha());
        visibility.set_if_neq(if mode.map_on() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }

    for (mut sprite, mut visibility) in &mut flashes {
        sprite.color = Color::srgba(SWITCH_GREY, SWITCH_GREY, SWITCH_GREY, mode.flash);
        visibility.set_if_neq(if mode.flash > 0.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }

    let shown = if mode.tiles_on() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut visibility in &mut chunks {
        visibility.set_if_neq(shown);
    }
}

fn rim_color(runup: &Runup) -> Vec4 {
    let [r, g, b] = look::BORDER_WATER_RUNUP.color();
    let tint = look::RIM_TINT;
    let channel = |c: u8, t: u8| f32::from(c) * f32::from(t) / (255.0 * 255.0);
    Vec4::new(
        channel(r, tint[0]),
        channel(g, tint[1]),
        channel(b, tint[2]),
        runup.alpha,
    )
}

fn ocean_color(clear: &ClearColor) -> Vec4 {
    let srgba = clear.0.to_srgba();
    Vec4::new(srgba.red, srgba.green, srgba.blue, 1.0)
}

pub fn draw(
    mut commands: Commands,
    mut drawn: ResMut<Drawn>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
    camera: Option<Single<(&Transform, &Projection), With<WorldCamera>>>,
    mode: Res<MapMode>,
    options: Res<crate::sleek::ScreenOptions>,
    runup: Res<Runup>,
    clear: Option<Res<ClearColor>>,
    map: Option<Res<TileMap>>,
) {
    if !mode.tiles_on() {
        return;
    }

    let (Some(map), Some(camera)) = (map, camera) else {
        return;
    };
    let (transform, projection) = camera.into_inner();
    let Projection::Orthographic(projection) = projection else {
        return;
    };

    let (x0, y0, x1, y1) = visible_tiles(
        &map,
        transform.translation.truncate(),
        projection.area.size(),
    );
    let ocean = clear.map(|clear| ocean_color(&clear)).unwrap_or(Vec4::ONE);
    let rim = rim_color(&runup);
    let shadow = Vec4::new(
        if options.shadows && mode.shadows_on() {
            mode.shadow_alpha()
        } else {
            0.0
        },
        0.0,
        0.0,
        0.0,
    );

    let wanted: Vec<(u32, u32)> = (y0 / CHUNK..=y1 / CHUNK)
        .flat_map(|cy| (x0 / CHUNK..=x1 / CHUNK).map(move |cx| (cx, cy)))
        .collect();
    let missing: Vec<(u32, u32)> = wanted
        .iter()
        .copied()
        .filter(|key| !drawn.chunks.contains_key(key))
        .collect();

    if !missing.is_empty() {
        let canvases = {
            let map = &*map;
            ComputeTaskPool::get_or_init(TaskPool::new).scope(|scope| {
                for &(cx, cy) in &missing {
                    scope.spawn(async move {
                        ((cx, cy), compose::compose(map, chunk_area(map, cx, cy)))
                    });
                }
            })
        };

        let quad = drawn
            .quad
            .get_or_insert_with(|| meshes.add(Rectangle::new(1.0, 1.0)))
            .clone();

        for ((cx, cy), canvas) in canvases {
            let (tiles, runup_counts) = chunk_images(&canvas);
            let tiles = images.add(tiles);
            let runup_counts = images.add(runup_counts);
            let material = materials.add(ChunkMaterial {
                rim,
                ocean,
                tiles: tiles.clone(),
                runup: runup_counts.clone(),
                shadow,
            });
            let (width, height) = chunk_tiles(&map, cx, cy);
            let centre = tile_centre(
                &map,
                (cx * CHUNK) as f32 + width as f32 * 0.5,
                (cy * CHUNK) as f32 + height as f32 * 0.5,
            );
            let entity = commands
                .spawn((
                    TileChunk { x: cx, y: cy },
                    Mesh2d(quad.clone()),
                    MeshMaterial2d(material.clone()),
                    Transform::from_xyz(centre.x, centre.y, TILES_LAYER).with_scale(Vec3::new(
                        width as f32 * TILE_SIZE,
                        height as f32 * TILE_SIZE,
                        1.0,
                    )),
                ))
                .id();
            drawn.chunks.insert(
                (cx, cy),
                ChunkView {
                    entity,
                    tiles,
                    runup: runup_counts,
                    material,
                },
            );
        }
    }

    let (bx0, by0, bx1, by1) = (x0 / BLOCK, y0 / BLOCK, x1 / BLOCK, y1 / BLOCK);
    let due: Vec<(u32, u32)> = drawn
        .dirty
        .iter()
        .copied()
        .filter(|&(bx, by)| bx >= bx0 && bx <= bx1 && by >= by0 && by <= by1)
        .collect();

    if !due.is_empty() {
        let canvases = {
            let map = &*map;
            ComputeTaskPool::get_or_init(TaskPool::new).scope(|scope| {
                for &(bx, by) in &due {
                    scope.spawn(async move {
                        ((bx, by), compose::compose(map, block_area(map, bx, by)))
                    });
                }
            })
        };

        for ((bx, by), canvas) in canvases {
            drawn.dirty.remove(&(bx, by));
            let key = (bx * BLOCK / CHUNK, by * BLOCK / CHUNK);
            let Some(chunk) = drawn.chunks.get(&key) else {
                continue;
            };
            let origin = chunk_area(&map, key.0, key.1);

            if let Some(mut image) = images.get_mut(&chunk.tiles) {
                write(&mut image, origin, &canvas, 4);
            }
            if let Some(mut image) = images.get_mut(&chunk.runup) {
                write(&mut image, origin, &canvas, 2);
            }
        }
    }

    for key in wanted {
        if let Some(chunk) = drawn.chunks.get(&key)
            && let Some(material) = materials.get(&chunk.material)
            && (material.rim != rim || material.ocean != ocean || material.shadow != shadow)
            && let Some(mut material) = materials.get_mut(&chunk.material)
        {
            material.rim = rim;
            material.ocean = ocean;
            material.shadow = shadow;
        }
    }
}

pub fn pulse(time: Res<Time>, clock: Option<Res<WorldClock>>, mut runup: ResMut<Runup>) {
    if clock.is_some_and(|clock| clock.paused) {
        return;
    }

    runup.update(time.delta_secs());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compose::PX;
    use econbox_tile::{Ground, Tile};
    use econbox_tilemap::TilePos;

    fn painter(map: TileMap) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<Painted>()
            .insert_resource(Assets::<Image>::default())
            .add_systems(Update, repaint);

        let minimap = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(minimap_image(&map));
        let mut drawn = Drawn {
            width: map.width,
            height: map.height,
            minimap: Some(minimap),
            ..Drawn::default()
        };
        drawn.chunks.insert(
            (0, 0),
            ChunkView {
                entity: Entity::PLACEHOLDER,
                tiles: Handle::default(),
                runup: Handle::default(),
                material: Handle::default(),
            },
        );
        app.insert_resource(drawn).insert_resource(map);
        app
    }

    fn pixel(app: &App, x: u32, y: u32) -> [u8; 4] {
        let drawn = app.world().resource::<Drawn>();
        let images = app.world().resource::<Assets<Image>>();
        let image = images
            .get(drawn.minimap.as_ref().expect("minimap"))
            .expect("image");
        let data = image.data.as_ref().expect("pixels");
        let at = ((y * drawn.width + x) * 4) as usize;
        [data[at], data[at + 1], data[at + 2], data[at + 3]]
    }

    fn stroke(app: &mut App, x: u32, y: u32, tile: Tile) {
        app.world_mut().resource_mut::<TileMap>().set(x, y, tile);
        app.world_mut()
            .write_message(Painted::new(vec![TilePos::new(x, y)]));
        app.update();
    }

    #[test]
    fn a_stroke_and_its_undo_both_reach_the_map_and_queue_the_tiles() {
        let mut app = painter(TileMap::filled(64, 64, Tile::bare(Ground::SoilHigh)));
        let before = pixel(&app, 10, 10);
        let above = pixel(&app, 10, 9);

        stroke(&mut app, 10, 10, Tile::bare(Ground::SoilLow));
        let [r, g, b] = Ground::SoilLow.color();
        let [sr, sg, sb] = compose::shade(Ground::SoilHigh.color());
        assert_eq!(pixel(&app, 10, 10), [r, g, b, 255]);
        assert_eq!(pixel(&app, 10, 9), [sr, sg, sb, 255]);
        {
            let drawn = app.world().resource::<Drawn>();
            assert!(drawn.dirty.contains(&(1, 1)));
            assert!(drawn.dirty.contains(&(1, 0)));
        }

        app.world_mut().resource_mut::<Drawn>().dirty.clear();
        stroke(&mut app, 10, 10, Tile::bare(Ground::SoilHigh));
        assert_eq!(pixel(&app, 10, 10), before);
        assert_eq!(pixel(&app, 10, 9), above);
        assert!(app.world().resource::<Drawn>().dirty.contains(&(1, 1)));
    }

    #[test]
    fn strokes_outside_a_drawn_chunk_queue_nothing() {
        let mut app = painter(TileMap::filled(192, 64, Tile::bare(Ground::SoilLow)));
        stroke(&mut app, 150, 10, Tile::bare(Ground::Sand));

        assert!(app.world().resource::<Drawn>().dirty.is_empty());
        assert_ne!(pixel(&app, 150, 10), pixel(&app, 20, 10));
    }

    #[test]
    fn a_chunk_image_is_four_pixels_per_tile() {
        let map = TileMap::filled(100, 70, Tile::bare(Ground::Sand));
        let canvas = compose::compose(&map, chunk_area(&map, 1, 1));
        let (tiles, rims) = chunk_images(&canvas);

        assert_eq!((tiles.width(), tiles.height()), (36 * PX, 6 * PX));
        assert_eq!((rims.width(), rims.height()), (36 * PX, 6 * PX));
        assert_eq!(tiles.texture_descriptor.format, TextureFormat::Rgba8Unorm);
    }

    #[test]
    fn a_redrawn_block_lands_at_its_own_offset_and_leaves_the_rest_alone() {
        let mut map = TileMap::filled(64, 64, Tile::bare(Ground::Sand));
        let origin = chunk_area(&map, 0, 0);
        let (mut tiles, _) = chunk_images(&compose::compose(&map, origin));
        let before = tiles.data.clone().expect("pixels");

        map.set(20, 12, Tile::bare(Ground::Mountains));
        let block = compose::compose(&map, block_area(&map, 2, 1));
        write(&mut tiles, origin, &block, 4);
        let after = tiles.data.expect("pixels");
        let whole = compose::compose(&map, origin).rgba();

        let at = |x: u32, y: u32| ((y * 256 + x) * 4) as usize;
        assert_eq!(
            after[at(81, 49)..at(81, 49) + 4],
            whole[at(81, 49)..at(81, 49) + 4]
        );
        assert_ne!(
            after[at(81, 49)..at(81, 49) + 4],
            before[at(81, 49)..at(81, 49) + 4]
        );
        assert_eq!(after[0..4], before[0..4]);
        assert_eq!(
            after[at(200, 200)..at(200, 200) + 4],
            before[at(200, 200)..at(200, 200) + 4]
        );
    }

    #[test]
    fn painting_one_tile_dirties_the_blocks_its_neighbours_reach() {
        let map = TileMap::filled(64, 64, Tile::default());
        let blocks: Vec<_> = dirty_blocks(&map, 8, 8).collect();

        assert!(blocks.contains(&(0, 0)));
        assert!(blocks.contains(&(1, 1)));
        let (wide, _) = compose::shadow_reach();
        let reach: Vec<_> = dirty_blocks(&map, 12, 12).collect();
        assert!(reach.contains(&(1, 1)));
        assert!(reach.contains(&(((12 + wide).min(63) / BLOCK), 1)));
        assert!(!reach.contains(&(1, 2)));
    }

    #[test]
    fn the_view_covers_the_tiles_under_the_camera_plus_a_margin() {
        let map = TileMap::filled(448, 448, Tile::default());
        let (x0, y0, x1, y1) = visible_tiles(&map, Vec2::ZERO, Vec2::splat(40.0 * TILE_SIZE));

        assert_eq!((x0, y0, x1, y1), (202, 202, 246, 246));
        assert_eq!(
            visible_tiles(&map, Vec2::ZERO, Vec2::splat(1e6)),
            (0, 0, 447, 447)
        );
    }

    #[test]
    fn the_minimap_has_one_pixel_per_tile() {
        let map = TileMap::filled(64, 48, Tile::bare(Ground::Sand));
        let image = minimap_image(&map);

        assert_eq!((image.width(), image.height()), (64, 48));
    }

    #[test]
    fn a_tile_centre_follows_the_cursor_convention() {
        let map = TileMap::filled(16, 16, Tile::default());

        assert_eq!(
            tile_centre(&map, 0.5, 0.5),
            Vec2::new(-7.5, 7.5) * TILE_SIZE
        );
    }
}
