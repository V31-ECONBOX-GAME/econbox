use crate::ambience::LightZones;
use crate::fire_art::{
    FLAME, FLAME_FRAMES, FLAME_PIVOT, SPARKLE, SPARKLE_FRAMES, flame_atlas, flame_set,
    sparkle_atlas,
};
use crate::quality::MapMode;
use crate::view::{Drawn, nearest, tile_centre, visible_tiles};
use bevy::asset::RenderAssetUsages;
use bevy::asset::embedded_path;
use bevy::asset::io::AssetSourceId;
use bevy::asset::{AssetPath, embedded_asset};
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, Extent3d,
    RenderPipelineDescriptor, SpecializedMeshPipelineError, TextureDimension, TextureFormat,
};
use bevy::shader::ShaderRef;
use bevy::sprite::Anchor;
use bevy::sprite_render::{
    AlphaMode2d, Material2d, Material2dKey, Material2dPipeline, Material2dPlugin,
};
use econbox_age::Dice;
use econbox_camera::{TILE_SIZE, WorldCamera};
use econbox_clock::{Frames, WorldClock};
use econbox_fire::{Behaviour, Fires};
use econbox_tilemap::{CHUNK, TileMap};
use std::collections::{BTreeMap, HashMap, HashSet};

pub const SCORCH_LAYER: f32 = 0.12;
pub const FIRE_MAP_LAYER: f32 = 0.13;
pub const FLAME_LAYER: f32 = 0.39;
pub const SPARKLE_LAYER: f32 = 2.0;
pub const BURNED_ALPHA: f32 = 0.5;
pub const FLASH_STEPS: f32 = 30.0;
pub const FLASH_ALPHA: f32 = 0.5;
pub const FIRE_COLOR: [u8; 3] = [0xff, 0x69, 0x30];
pub const FLAME_SCALE: f32 = 0.15;
pub const FLAME_FPS: f32 = 10.0;
pub const SPARKLE_SCALE: f32 = 0.25;
pub const SPARKLE_FRAME: f32 = 0.1;
pub const SPARKLE_LIMIT: usize = 15;
pub const SPARK_INTERVAL: f32 = 0.1;
pub const SPARK_RANDOM: f32 = 1.0;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GlowMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub marks: Handle<Image>,
}

impl Material2d for GlowMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("glow.wgsl"))
                .with_source(AssetSourceId::from("embedded")),
        )
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }

    fn specialize(
        _pipeline: &Material2dPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = descriptor.fragment.as_mut() {
            for target in fragment.targets.iter_mut().flatten() {
                target.blend = Some(BlendState {
                    color: BlendComponent {
                        src_factor: BlendFactor::SrcAlpha,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                    alpha: BlendComponent {
                        src_factor: BlendFactor::Zero,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                });
            }
        }
        Ok(())
    }
}

pub fn burned_alpha(stage: u8) -> f32 {
    if stage == 0 {
        0.0
    } else {
        f32::from(stage - 1) / f32::from(econbox_fire::BURNED_STAGES) * BURNED_ALPHA
    }
}

pub fn flash_alpha(state: u8) -> f32 {
    f32::from(state) / FLASH_STEPS * FLASH_ALPHA
}

fn to_linear(value: f32) -> f32 {
    Color::srgb(value, value, value).to_linear().red
}

fn to_srgb(value: f32) -> f32 {
    Color::linear_rgb(value, value, value).to_srgba().red
}

pub fn linear_black(alpha: f32) -> f32 {
    1.0 - to_linear(1.0 - alpha)
}

pub fn scorch_pixel(stage: u8, flash: u8) -> [u8; 4] {
    let black = linear_black(burned_alpha(stage));
    let white = flash_alpha(flash);
    let alpha = white + black * (1.0 - white);
    if alpha <= 0.0 {
        return [0; 4];
    }
    let level = (to_srgb(white / alpha) * 255.0).round() as u8;
    [level, level, level, (alpha * 255.0).round() as u8]
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScorchChunk {
    pub x: u32,
    pub y: u32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlowChunk;

#[derive(Debug, Clone)]
pub struct ScorchView {
    pub scorch: Entity,
    pub scorch_image: Handle<Image>,
    pub glow: Entity,
    pub glow_image: Handle<Image>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ScorchLayer {
    pub epoch: u64,
    pub chunks: HashMap<(u32, u32), ScorchView>,
    pub flashes: BTreeMap<u32, u8>,
    pub quad: Option<Handle<Mesh>>,
}

fn chunk_image(width: u32, height: u32) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    nearest(&mut image);
    image
}

fn chunk_size(map: &TileMap, cx: u32, cy: u32) -> (u32, u32) {
    (
        CHUNK.min(map.width - cx * CHUNK),
        CHUNK.min(map.height - cy * CHUNK),
    )
}

pub fn scorch(
    mut commands: Commands,
    fires: Option<Res<Fires>>,
    map: Option<Res<TileMap>>,
    drawn: Res<Drawn>,
    mode: Res<MapMode>,
    mut layer: ResMut<ScorchLayer>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GlowMaterial>>,
    mut glows: Query<&mut Visibility, With<GlowChunk>>,
) {
    let (Some(fires), Some(map)) = (fires, map) else {
        return;
    };
    if fires.width != map.width || fires.height != map.height || map.width == 0 {
        return;
    }

    if layer.epoch != drawn.epoch {
        for (_, view) in layer.chunks.drain() {
            commands.entity(view.scorch).despawn();
            commands.entity(view.glow).despawn();
        }
        layer.flashes.clear();
        layer.epoch = drawn.epoch;
    }

    let mut dirty: HashSet<(u32, u32)> = HashSet::new();
    let chunk_of = |index: u32| {
        let x = index % map.width;
        let y = index / map.width;
        (x / CHUNK, y / CHUNK)
    };

    for &index in &fires.changed {
        dirty.insert(chunk_of(index));
    }
    for &(index, value) in &fires.flashes {
        let state = layer.flashes.entry(index).or_default();
        *state = (*state).max(value);
    }
    for &index in layer.flashes.keys() {
        dirty.insert(chunk_of(index));
    }

    let quad = layer
        .quad
        .get_or_insert_with(|| meshes.add(Rectangle::new(1.0, 1.0)))
        .clone();

    for (cx, cy) in dirty {
        let (width, height) = chunk_size(&map, cx, cy);
        let ScorchLayer {
            chunks, flashes, ..
        } = &mut *layer;
        let view = chunks.entry((cx, cy)).or_insert_with(|| {
            let scorch_image = images.add(chunk_image(width, height));
            let glow_image = images.add(chunk_image(width, height));
            let centre = tile_centre(
                &map,
                (cx * CHUNK) as f32 + width as f32 * 0.5,
                (cy * CHUNK) as f32 + height as f32 * 0.5,
            );
            let size = Vec2::new(width as f32, height as f32) * TILE_SIZE;
            let scorch = commands
                .spawn((
                    ScorchChunk { x: cx, y: cy },
                    Sprite {
                        image: scorch_image.clone(),
                        custom_size: Some(size),
                        ..default()
                    },
                    Transform::from_xyz(centre.x, centre.y, SCORCH_LAYER),
                ))
                .id();
            let material = materials.add(GlowMaterial {
                marks: glow_image.clone(),
            });
            let glow = commands
                .spawn((
                    GlowChunk,
                    Mesh2d(quad.clone()),
                    MeshMaterial2d(material),
                    Transform::from_xyz(centre.x, centre.y, FIRE_MAP_LAYER)
                        .with_scale(size.extend(1.0)),
                    Visibility::Hidden,
                ))
                .id();
            ScorchView {
                scorch,
                scorch_image,
                glow,
                glow_image,
            }
        });

        if let Some(mut image) = images.get_mut(&view.scorch_image)
            && let Some(data) = image.data.as_mut()
        {
            for row in 0..height {
                for column in 0..width {
                    let x = cx * CHUNK + column;
                    let y = cy * CHUNK + row;
                    let index = y * map.width + x;
                    let pixel = scorch_pixel(
                        fires.burned_stage(x, y),
                        flashes.get(&index).copied().unwrap_or_default(),
                    );
                    let at = ((row * width + column) * 4) as usize;
                    data[at..at + 4].copy_from_slice(&pixel);
                }
            }
        }
        if let Some(mut image) = images.get_mut(&view.glow_image)
            && let Some(data) = image.data.as_mut()
        {
            for row in 0..height {
                for column in 0..width {
                    let x = cx * CHUNK + column;
                    let y = cy * CHUNK + row;
                    let [r, g, b] = FIRE_COLOR;
                    let alpha = if fires.on_fire(x, y) { 255 } else { 0 };
                    let at = ((row * width + column) * 4) as usize;
                    data[at..at + 4].copy_from_slice(&[r, g, b, alpha]);
                }
            }
        }
    }

    layer.flashes.retain(|_, state| {
        if *state == 0 {
            return false;
        }
        *state -= 1;
        true
    });

    let shown = if mode.low_res && fires.has_fires() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut visibility in &mut glows {
        visibility.set_if_neq(shown);
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Flame;

#[derive(Resource, Debug, Clone, Default)]
pub struct Flames {
    pub atlas: Option<Handle<Image>>,
    pub pool: Vec<Entity>,
}

pub fn seed(x: u32, y: u32) -> u32 {
    crate::objects::hash(x, y, 71) % 10_000
}

pub fn flame_frame(time: f32, seed: u32) -> u32 {
    ((time * FLAME_FPS) as u32).wrapping_add(seed.wrapping_mul(100)) % FLAME_FRAMES
}

pub fn flames(
    mut commands: Commands,
    fires: Option<Res<Fires>>,
    map: Option<Res<TileMap>>,
    mode: Res<MapMode>,
    time: Res<Time>,
    camera: Option<Single<(&Transform, &Projection), With<WorldCamera>>>,
    mut state: ResMut<Flames>,
    mut images: ResMut<Assets<Image>>,
    mut sprites: Query<
        (&mut Sprite, &mut Transform, &mut Visibility),
        (With<Flame>, Without<WorldCamera>),
    >,
) {
    let mut spots: Vec<(u32, u32)> = Vec::new();
    if let (Some(fires), Some(map), Some(camera)) = (fires, map.as_ref(), camera)
        && mode.tiles_on()
        && fires.width == map.width
        && fires.height == map.height
        && let (transform, Projection::Orthographic(projection)) = camera.into_inner()
    {
        let (x0, y0, x1, y1) = visible_tiles(
            map,
            transform.translation.truncate(),
            projection.area.size(),
        );
        let inside = |index: u32| {
            let x = index % map.width;
            let y = index / map.width;
            (x >= x0 && x <= x1 && y >= y0 && y <= y1).then_some((x, y))
        };
        spots.extend(fires.burning.keys().filter_map(|&index| inside(index)));
        spots.extend(
            fires
                .objects
                .keys()
                .filter(|index| !fires.burning.contains_key(index))
                .filter_map(|&index| inside(index)),
        );
    }

    let atlas = state
        .atlas
        .get_or_insert_with(|| {
            let (width, height, data) = flame_atlas();
            let mut image = Image::new(
                Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                data,
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::RENDER_WORLD,
            );
            nearest(&mut image);
            images.add(image)
        })
        .clone();

    while state.pool.len() < spots.len() {
        let entity = commands
            .spawn((
                Flame,
                Sprite {
                    image: atlas.clone(),
                    custom_size: Some(Vec2::splat(FLAME as f32 * FLAME_SCALE * TILE_SIZE)),
                    rect: Some(Rect::new(0.0, 0.0, FLAME as f32, FLAME as f32)),
                    ..default()
                },
                Anchor(Vec2::new(0.0, FLAME_PIVOT - 0.5)),
                Transform::from_xyz(0.0, 0.0, FLAME_LAYER),
                Visibility::Hidden,
            ))
            .id();
        state.pool.push(entity);
    }

    let Some(map) = map else {
        return;
    };
    let now = time.elapsed_secs();
    for (slot, &entity) in state.pool.iter().enumerate() {
        let Ok((mut sprite, mut transform, mut visibility)) = sprites.get_mut(entity) else {
            continue;
        };
        let Some(&(x, y)) = spots.get(slot) else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        let seed = seed(x, y);
        let set = flame_set(seed);
        let frame = flame_frame(now, seed);
        let left = (frame * FLAME) as f32;
        let top = (set * FLAME) as f32;
        sprite.rect = Some(Rect::new(
            left,
            top,
            left + FLAME as f32,
            top + FLAME as f32,
        ));
        let at = tile_centre(&map, x as f32 + 0.5, y as f32 + 0.5);
        transform.translation = at.extend(FLAME_LAYER + 0.000_01 * y as f32);
        visibility.set_if_neq(Visibility::Inherited);
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Sparkle {
    pub age: f32,
}

#[derive(Resource, Debug, Clone)]
pub struct Sparkles {
    pub atlas: Option<Handle<Image>>,
    pub behaviour: Behaviour,
    pub dice: Dice,
}

impl Default for Sparkles {
    fn default() -> Self {
        Self {
            atlas: None,
            behaviour: Behaviour::new(SPARK_INTERVAL, SPARK_RANDOM),
            dice: Dice::new(0x5a4c),
        }
    }
}

pub fn sparkle_spot(x: u32, y: u32) -> Option<(f32, f32)> {
    (y > 0).then(|| (x as f32 + 1.0, (y - 1) as f32 + 0.5))
}

pub fn sparkles(
    mut commands: Commands,
    frames: Res<Frames>,
    clock: Option<Res<WorldClock>>,
    mode: Res<MapMode>,
    map: Option<Res<TileMap>>,
    drawn: Res<Drawn>,
    mut zones: ResMut<LightZones>,
    mut state: ResMut<Sparkles>,
    mut images: ResMut<Assets<Image>>,
    mut live: Query<(Entity, &mut Sparkle, &mut Sprite)>,
) {
    let clock = clock.map(|clock| *clock).unwrap_or_default();
    let elapsed = if clock.running() {
        clock.step() * frames.due as f32
    } else {
        0.0
    };

    let mut count = 0;
    for (entity, mut sparkle, mut sprite) in &mut live {
        sparkle.age += elapsed;
        let frame = (sparkle.age / SPARKLE_FRAME) as u32;
        if frame >= SPARKLE_FRAMES {
            commands.entity(entity).despawn();
            continue;
        }
        let left = (frame * SPARKLE) as f32;
        sprite.rect = Some(Rect::new(left, 0.0, left + SPARKLE as f32, SPARKLE as f32));
        count += 1;
    }

    let Some(map) = map else {
        return;
    };
    if !mode.tiles_on() || !clock.running() || map.width == 0 {
        return;
    }

    let atlas = state
        .atlas
        .get_or_insert_with(|| {
            let (width, height, data) = sparkle_atlas();
            let mut image = Image::new(
                Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                data,
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::RENDER_WORLD,
            );
            nearest(&mut image);
            images.add(image)
        })
        .clone();

    for _ in 0..frames.due {
        let Sparkles {
            behaviour, dice, ..
        } = &mut *state;
        if !behaviour.due(clock.step(), dice) {
            continue;
        }
        zones.ensure(&map, drawn.epoch);
        let spots = zones.sparkles(&map);
        let Some((x, y)) = state.dice.pick(&spots) else {
            continue;
        };
        let Some((sx, sy)) = sparkle_spot(x, y) else {
            continue;
        };
        if count >= SPARKLE_LIMIT {
            continue;
        }
        count += 1;
        let at = tile_centre(&map, sx, sy);
        commands.spawn((
            Sparkle { age: 0.0 },
            Sprite {
                image: atlas.clone(),
                custom_size: Some(Vec2::splat(SPARKLE as f32 * SPARKLE_SCALE * TILE_SIZE)),
                rect: Some(Rect::new(0.0, 0.0, SPARKLE as f32, SPARKLE as f32)),
                ..default()
            },
            Transform::from_xyz(at.x, at.y, SPARKLE_LAYER),
        ));
    }
}

pub struct FireViewPlugin;

impl Plugin for FireViewPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "glow.wgsl");
        app.add_plugins(Material2dPlugin::<GlowMaterial>::default())
            .init_resource::<ScorchLayer>()
            .init_resource::<Flames>()
            .init_resource::<Sparkles>()
            .init_resource::<Frames>()
            .add_systems(PostUpdate, (scorch, flames, sparkles));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scorch_darkens_in_fifteen_steps_up_to_half() {
        assert_eq!(burned_alpha(0), 0.0);
        assert_eq!(burned_alpha(1), 0.0);
        assert!((burned_alpha(15) - 14.0 / 15.0 * 0.5).abs() < 1e-6);
        assert_eq!(scorch_pixel(0, 0), [0; 4]);
        let dark = scorch_pixel(15, 0);
        assert_eq!(dark[0], 0);
        assert!(dark[3] > (0.4667 * 255.0) as u8);
    }

    #[test]
    fn a_black_overlay_keeps_its_gamma_strength_in_linear_blending() {
        let gamma = 0.6_f32;
        let alpha = 0.3;
        let want = to_linear(gamma * (1.0 - alpha));
        let got = to_linear(gamma) * (1.0 - linear_black(alpha));
        assert!((want - got).abs() < 0.01, "{want} {got}");
    }

    #[test]
    fn a_flash_of_ten_is_a_sixth_white() {
        assert!((flash_alpha(10) - 1.0 / 6.0).abs() < 1e-6);
        let pixel = scorch_pixel(0, 10);
        assert_eq!(&pixel[..3], &[255, 255, 255]);
        assert_eq!(pixel[3], 43);
    }

    #[test]
    fn flames_cycle_through_six_frames_at_ten_a_second() {
        assert_eq!(flame_frame(0.0, 0), 0);
        assert_eq!(flame_frame(0.1, 0), 1);
        assert_eq!(flame_frame(0.65, 0), 0);
        assert_eq!(flame_frame(0.0, 1), 100 % 6);
    }

    #[test]
    fn a_sparkle_rises_over_the_tile_above_the_mineral() {
        assert_eq!(sparkle_spot(4, 0), None);
        assert_eq!(sparkle_spot(4, 7), Some((5.0, 6.5)));
    }
}
