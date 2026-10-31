use crate::look::{self, Layer};
use crate::objects;
use crate::quality::{MapMode, Night};
use crate::sleek::ScreenEffects;
use crate::view::{Drawn, tile_centre, visible_tiles};
use bevy::asset::{embedded_asset, load_embedded_asset};
use bevy::core_pipeline::FullscreenShader;
use bevy::core_pipeline::schedule::{Core2d, Core2dSystems};
use bevy::core_pipeline::tonemapping::tonemapping;
use bevy::mesh::VertexBufferLayout;
use bevy::prelude::*;
use bevy::render::camera::ExtractedCamera;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::render_resource::binding_types::{texture_2d, uniform_buffer};
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderContext, RenderDevice, ViewQuery};
use bevy::render::texture::{CachedTexture, TextureCache};
use bevy::render::view::ViewTarget;
use bevy::render::{Render, RenderApp, RenderStartup, RenderSystems};
use bevy::shader::Shader;
use econbox_age::{AgeSettings, Ages, Overlay};
use econbox_camera::{TILE_SIZE, WorldCamera};
use econbox_fire::Fires;
use econbox_tile::Wall;
use econbox_tilemap::{Painted, TileMap};

pub const ZONE: u32 = 8;
pub const LIGHT_DISC: f32 = 25.0;
pub const LIGHT_SHOW: f32 = 0.6;
pub const LIGHT_DIVISOR: u32 = 3;
pub const LAVA_ZONE_LIGHTS: usize = 5;
pub const LAVA_LIGHT: f32 = 0.2;
pub const LAVA_ZONE_LIGHT: f32 = 1.0;
pub const WALL_LIGHT: f32 = 0.3;
pub const FIRE_LIGHT: f32 = 0.2;
pub const BURNING_LIGHT: f32 = 0.2;
pub const LIGHT_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Blend {
    Multiply,
    Additive,
    Alpha,
}

impl Blend {
    fn code(self) -> f32 {
        match self {
            Self::Multiply => 0.0,
            Self::Additive => 1.0,
            Self::Alpha => 2.0,
        }
    }
}

pub const OVERLAY_LOOKS: [(Overlay, Blend, [f32; 3]); 9] = [
    (
        Overlay::Night,
        Blend::Multiply,
        [0.130_032, 0.122_553, 0.320_755],
    ),
    (Overlay::Chaos, Blend::Multiply, [1.0, 0.0, 0.0]),
    (Overlay::Moon, Blend::Multiply, [0.0, 0.839_623, 0.831_720]),
    (Overlay::Magic, Blend::Additive, [1.0, 0.259_434, 0.766_955]),
    (Overlay::Sun, Blend::Additive, [1.0, 0.937_158, 0.513_726]),
    (
        Overlay::RainDarkness,
        Blend::Multiply,
        [0.122_989, 0.103_729, 0.198_113],
    ),
    (Overlay::Winter, Blend::Alpha, [0.0, 0.0, 0.0]),
    (
        Overlay::Ash,
        Blend::Alpha,
        [0.283_019, 0.241_278, 0.193_574],
    ),
    (
        Overlay::Rain,
        Blend::Alpha,
        [0.193_797, 0.346_591, 0.632_075],
    ),
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightBlob {
    pub pos: Vec2,
    pub radius: f32,
}

impl LightBlob {
    pub fn at_tile(map: &TileMap, x: f32, y: f32, radius: f32) -> Self {
        Self {
            pos: tile_centre(map, x + 0.5, y + 0.5),
            radius,
        }
    }

    pub fn half_size(&self) -> f32 {
        LIGHT_DISC * self.radius * TILE_SIZE
    }
}

#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct LightBlobs(pub Vec<LightBlob>);

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ZoneLights {
    pub lava: Vec<(u32, u32)>,
    pub walls: Vec<(u32, u32)>,
    pub things: Vec<(f32, f32, f32)>,
    pub sparkles: Vec<(u32, u32)>,
    pub centre: Option<(u32, u32)>,
}

impl ZoneLights {
    pub fn scan(map: &TileMap, zx: u32, zy: u32) -> Self {
        let mut zone = Self::default();
        for local_y in 0..ZONE {
            for local_x in 0..ZONE {
                let x = zx * ZONE + local_x;
                let up = zy * ZONE + local_y;
                if x >= map.width || up >= map.height {
                    continue;
                }
                let y = map.height - 1 - up;
                if local_x == ZONE / 2 && local_y == ZONE / 2 {
                    zone.centre = Some((x, y));
                }
                let tile = map.at(x, y);
                if look::of(tile).layer == Layer::Lava {
                    zone.lava.push((x, y));
                }
                if tile.wall == Some(Wall::Light) {
                    zone.walls.push((x, y));
                }
                if tile.flora.is_none() && tile.ore.is_none() {
                    continue;
                }
                for placed in objects::objects(tile, x, y) {
                    let objects::Sprite::Kind { kind, .. } = placed.sprite else {
                        continue;
                    };
                    if let Some((size, offset_y)) = kind.light {
                        zone.things.push((x as f32, y as f32 - offset_y, size));
                    }
                    if kind.sparkle {
                        zone.sparkles.push((x, y));
                    }
                }
            }
        }
        zone
    }

    pub fn lava_blobs(&self, map: &TileMap, out: &mut Vec<LightBlob>) {
        if self.lava.is_empty() {
            return;
        }
        if self.lava.len() < LAVA_ZONE_LIGHTS {
            out.extend(
                self.lava
                    .iter()
                    .map(|&(x, y)| LightBlob::at_tile(map, x as f32, y as f32, LAVA_LIGHT)),
            );
        } else if let Some((x, y)) = self.centre {
            out.push(LightBlob::at_tile(map, x as f32, y as f32, LAVA_ZONE_LIGHT));
        }
    }

    pub fn wall_blobs(&self, map: &TileMap, out: &mut Vec<LightBlob>) {
        out.extend(
            self.walls
                .iter()
                .map(|&(x, y)| LightBlob::at_tile(map, x as f32, y as f32, WALL_LIGHT)),
        );
    }

    pub fn thing_blobs(&self, map: &TileMap, out: &mut Vec<LightBlob>) {
        out.extend(
            self.things
                .iter()
                .map(|&(x, y, size)| LightBlob::at_tile(map, x, y, size)),
        );
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct LightZones {
    pub epoch: u64,
    pub columns: u32,
    pub rows: u32,
    pub zones: Vec<Option<ZoneLights>>,
    pub sparkle_cache: Option<Vec<(u32, u32)>>,
}

impl LightZones {
    pub fn reset(&mut self, map: &TileMap, epoch: u64) {
        self.epoch = epoch;
        self.columns = map.width.div_ceil(ZONE);
        self.rows = map.height.div_ceil(ZONE);
        self.zones = vec![None; (self.columns * self.rows) as usize];
        self.sparkle_cache = None;
    }

    pub fn ensure(&mut self, map: &TileMap, epoch: u64) {
        if self.epoch != epoch
            || self.zones.is_empty()
            || self.columns != map.width.div_ceil(ZONE)
            || self.rows != map.height.div_ceil(ZONE)
        {
            self.reset(map, epoch);
        }
    }

    pub fn sparkles(&mut self, map: &TileMap) -> Vec<(u32, u32)> {
        if let Some(cached) = &self.sparkle_cache {
            return cached.clone();
        }
        let mut found = Vec::new();
        for zy in 0..self.rows {
            for zx in 0..self.columns {
                found.extend(self.zone(map, zx, zy).sparkles.iter().copied());
            }
        }
        self.sparkle_cache = Some(found.clone());
        found
    }

    pub fn zone_of(&self, map: &TileMap, x: u32, y: u32) -> (u32, u32) {
        (x / ZONE, (map.height - 1 - y) / ZONE)
    }

    pub fn forget(&mut self, map: &TileMap, x: u32, y: u32) {
        let (zx, zy) = self.zone_of(map, x, y);
        if let Some(slot) = self.zones.get_mut((zy * self.columns + zx) as usize) {
            *slot = None;
            self.sparkle_cache = None;
        }
    }

    pub fn zone(&mut self, map: &TileMap, zx: u32, zy: u32) -> &ZoneLights {
        let slot = &mut self.zones[(zy * self.columns + zx) as usize];
        slot.get_or_insert_with(|| ZoneLights::scan(map, zx, zy))
    }

    pub fn span(&self, map: &TileMap, tiles: (u32, u32, u32, u32), margin: u32) -> [u32; 4] {
        let (x0, y0, x1, y1) = tiles;
        let (zx0, zy1) = self.zone_of(map, x0, y0);
        let (zx1, zy0) = self.zone_of(map, x1, y1);
        [
            zx0.saturating_sub(margin),
            zy0.saturating_sub(margin),
            (zx1 + margin).min(self.columns - 1),
            (zy1 + margin).min(self.rows - 1),
        ]
    }
}

#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct Ambience {
    pub light: Vec4,
    pub overlays: [Vec4; 9],
    pub blobs: Vec<Vec4>,
}

impl Ambience {
    pub fn active(&self) -> bool {
        self.light.w > 0.0 || self.overlays.iter().any(|overlay| overlay.w > 0.0)
    }
}

impl ExtractResource<RenderApp> for Ambience {
    type Source = Ambience;

    fn extract_resource(source: &Self::Source) -> Self {
        source.clone()
    }
}

pub fn overlay_colors(ages: &Ages) -> [Vec4; 9] {
    OVERLAY_LOOKS.map(|(overlay, _, [r, g, b])| Vec4::new(r, g, b, ages.layer(overlay).shown()))
}

pub fn light_color(ages: &Ages) -> Vec4 {
    let [r, g, b] = ages.asset().light_color.map(|c| f32::from(c) / 255.0);
    Vec4::new(r, g, b, ages.night_mod() * LIGHT_SHOW)
}

pub fn blob_quad(blob: &LightBlob, centre: Vec2, area: Rect) -> Option<Vec4> {
    let half = area.half_size();
    if half.x <= 0.0 || half.y <= 0.0 {
        return None;
    }
    let offset = blob.pos - centre - area.center();
    let ndc = offset / half;
    let size = Vec2::splat(blob.half_size()) / half;
    let outside = ndc.x - size.x > 1.0
        || ndc.x + size.x < -1.0
        || ndc.y - size.y > 1.0
        || ndc.y + size.y < -1.0;
    (!outside && size.x > 0.0).then(|| Vec4::new(ndc.x, ndc.y, size.x, size.y))
}

pub fn fire_blobs(
    map: &TileMap,
    fires: &Fires,
    tiles: (u32, u32, u32, u32),
    gameplay: bool,
    out: &mut Vec<LightBlob>,
) {
    let (x0, y0, x1, y1) = tiles;
    let inside = |index: u32| {
        let x = index % map.width;
        let y = index / map.width;
        (x >= x0 && x <= x1 && y >= y0 && y <= y1).then_some((x, y))
    };
    if gameplay {
        out.extend(
            fires
                .burning
                .keys()
                .filter_map(|&index| inside(index))
                .map(|(x, y)| LightBlob::at_tile(map, x as f32, y as f32, FIRE_LIGHT)),
        );
    }
    out.extend(fires.objects.keys().map(|&index| {
        LightBlob::at_tile(
            map,
            (index % map.width) as f32,
            (index / map.width) as f32,
            BURNING_LIGHT,
        )
    }));
}

pub fn clear_blobs(mut blobs: ResMut<LightBlobs>) {
    blobs.0.clear();
}

pub fn night(ages: Option<Res<Ages>>, mut night: ResMut<Night>) {
    let value = ages.map(|ages| ages.night_mod()).unwrap_or_default();
    night.set_if_neq(Night(value));
}

pub fn forget(
    mut painted: MessageReader<Painted>,
    mut zones: ResMut<LightZones>,
    map: Option<Res<TileMap>>,
) {
    let Some(map) = map else {
        painted.clear();
        return;
    };
    for message in painted.read() {
        for pos in &message.cells {
            if pos.x < map.width && pos.y < map.height && !zones.zones.is_empty() {
                zones.forget(&map, pos.x, pos.y);
            }
        }
    }
}

pub fn gather(
    ages: Option<Res<Ages>>,
    settings: Option<Res<AgeSettings>>,
    map: Option<Res<TileMap>>,
    drawn: Res<Drawn>,
    mode: Res<MapMode>,
    camera: Option<Single<(&Transform, &Projection), With<WorldCamera>>>,
    blobs: Res<LightBlobs>,
    fires: Option<Res<Fires>>,
    mut zones: ResMut<LightZones>,
    mut ambience: ResMut<Ambience>,
) {
    let (Some(ages), Some(map)) = (ages, map) else {
        ambience.set_if_neq(Ambience::default());
        return;
    };
    let settings = settings.map(|settings| *settings).unwrap_or_default();

    let mut next = Ambience {
        light: light_color(&ages),
        overlays: overlay_colors(&ages),
        blobs: Vec::new(),
    };

    if let Some(camera) = camera
        && let (transform, Projection::Orthographic(projection)) = camera.into_inner()
        && settings.night_lights
        && ages.show_lights()
        && map.width > 0
        && map.height > 0
    {
        zones.ensure(&map, drawn.epoch);
        let centre = transform.translation.truncate();
        let area = projection.area;
        let tiles = visible_tiles(&map, centre, area.size());
        let mut found = blobs.0.clone();

        let [zx0, zy0, zx1, zy1] = zones.span(&map, tiles, 0);
        for zy in zy0..=zy1 {
            for zx in zx0..=zx1 {
                let zone = zones.zone(&map, zx, zy);
                zone.lava_blobs(&map, &mut found);
                zone.wall_blobs(&map, &mut found);
            }
        }

        let margin = if mode.render_buildings() { 0 } else { 4 };
        let [zx0, zy0, zx1, zy1] = zones.span(&map, tiles, margin);
        for zy in zy0..=zy1 {
            for zx in zx0..=zx1 {
                zones.zone(&map, zx, zy).thing_blobs(&map, &mut found);
            }
        }

        if let Some(fires) = fires.as_deref()
            && fires.width == map.width
            && fires.height == map.height
        {
            fire_blobs(&map, fires, tiles, mode.tiles_on(), &mut found);
        }

        next.blobs = found
            .iter()
            .filter_map(|blob| blob_quad(blob, centre, area))
            .collect();
    }

    ambience.set_if_neq(next);
}

pub struct AmbiencePlugin;

impl Plugin for AmbiencePlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "ambience.wgsl");

        app.init_resource::<Ambience>()
            .init_resource::<LightBlobs>()
            .init_resource::<LightZones>()
            .add_plugins(ExtractResourcePlugin::<Ambience>::default())
            .add_systems(First, clear_blobs)
            .add_systems(PostUpdate, (forget, gather).chain());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .add_systems(RenderStartup, init_pipelines)
            .add_systems(
                Render,
                prepare_textures.in_set(RenderSystems::PrepareResources),
            )
            .add_systems(
                Core2d,
                ambience
                    .after(tonemapping)
                    .before(crate::sleek::sleek)
                    .in_set(Core2dSystems::PostProcess),
            );
    }
}

#[derive(ShaderType, Clone, Copy, Debug, Default)]
struct AmbienceUniform {
    light: Vec4,
    overlays: [Vec4; 9],
    kinds: [Vec4; 9],
}

#[derive(Resource)]
struct AmbiencePipelines {
    layout: BindGroupLayoutDescriptor,
    blobs: CachedRenderPipelineId,
    compose: Vec<(TextureFormat, CachedRenderPipelineId)>,
}

fn init_pipelines(
    mut commands: Commands,
    fullscreen: Res<FullscreenShader>,
    assets: Res<AssetServer>,
    cache: Res<PipelineCache>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "ambience_layout",
        &BindGroupLayoutEntries::with_indices(
            ShaderStages::FRAGMENT,
            (
                (0, texture_2d(TextureSampleType::Float { filterable: true })),
                (1, texture_2d(TextureSampleType::Float { filterable: true })),
                (2, uniform_buffer::<AmbienceUniform>(false)),
            ),
        ),
    );
    let shader: Handle<Shader> = load_embedded_asset!(assets.as_ref(), "ambience.wgsl");
    let over = BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    };

    let blobs = cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("ambience_blobs".into()),
        layout: vec![],
        vertex: VertexState {
            shader: shader.clone(),
            entry_point: Some("blob_vertex".into()),
            buffers: vec![VertexBufferLayout {
                array_stride: 16,
                step_mode: VertexStepMode::Instance,
                attributes: vec![VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                }],
            }],
            ..default()
        },
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleStrip,
            ..default()
        },
        fragment: Some(FragmentState {
            shader: shader.clone(),
            entry_point: Some("blob_fragment".into()),
            targets: vec![Some(ColorTargetState {
                format: LIGHT_FORMAT,
                blend: Some(BlendState {
                    color: over,
                    alpha: over,
                }),
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        ..default()
    });

    let compose = [
        TextureFormat::Rgba8UnormSrgb,
        TextureFormat::Bgra8UnormSrgb,
        TextureFormat::Rgba16Float,
    ]
    .into_iter()
    .map(|format| {
        let id = cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("ambience_compose".into()),
            layout: vec![layout.clone()],
            vertex: fullscreen.to_vertex_state(),
            fragment: Some(FragmentState {
                shader: shader.clone(),
                entry_point: Some("compose".into()),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            ..default()
        });
        (format, id)
    })
    .collect();

    commands.insert_resource(AmbiencePipelines {
        layout,
        blobs,
        compose,
    });
}

#[derive(Component)]
struct LightTexture(CachedTexture);

pub fn light_size(width: u32, height: u32) -> UVec2 {
    UVec2::new(
        (width / LIGHT_DIVISOR).max(1),
        (height / LIGHT_DIVISOR).max(1),
    )
}

fn prepare_textures(
    mut commands: Commands,
    mut cache: ResMut<TextureCache>,
    device: Res<RenderDevice>,
    views: Query<(Entity, &ExtractedCamera), With<ScreenEffects>>,
) {
    for (entity, camera) in &views {
        let Some(size) = camera.physical_viewport_size else {
            continue;
        };
        let size = light_size(size.x, size.y);
        let texture = cache.get(
            &device,
            TextureDescriptor {
                label: Some("ambience_light"),
                size: Extent3d {
                    width: size.x,
                    height: size.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: LIGHT_FORMAT,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
        );
        commands.entity(entity).insert(LightTexture(texture));
    }
}

fn ambience(
    view: ViewQuery<(&ViewTarget, &LightTexture), With<ScreenEffects>>,
    ambience: Option<Res<Ambience>>,
    pipelines: Option<Res<AmbiencePipelines>>,
    cache: Res<PipelineCache>,
    mut ctx: RenderContext,
) {
    let (Some(ambience), Some(pipelines)) = (ambience, pipelines) else {
        return;
    };
    if !ambience.active() {
        return;
    }
    let (target, light) = view.into_inner();

    let format = target.main_texture_format();
    let Some(compose_id) = pipelines
        .compose
        .iter()
        .find(|(candidate, _)| *candidate == format)
        .map(|(_, id)| *id)
    else {
        return;
    };
    let (Some(blobs), Some(compose)) = (
        cache.get_render_pipeline(pipelines.blobs),
        cache.get_render_pipeline(compose_id),
    ) else {
        return;
    };

    let device = ctx.render_device().clone();
    let instances = (!ambience.blobs.is_empty()).then(|| {
        let bytes: Vec<u8> = ambience
            .blobs
            .iter()
            .flat_map(|quad| quad.to_array())
            .flat_map(f32::to_ne_bytes)
            .collect();
        device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("ambience_blobs"),
            contents: &bytes,
            usage: BufferUsages::VERTEX,
        })
    });

    {
        let mut pass = ctx
            .command_encoder()
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("ambience_lights"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &light.0.default_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(LinearRgba::BLACK.into()),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        if let Some(instances) = &instances {
            pass.set_pipeline(blobs);
            pass.set_vertex_buffer(0, *instances.slice(..));
            pass.draw(0..4, 0..ambience.blobs.len() as u32);
        }
    }

    let uniform = AmbienceUniform {
        light: ambience.light,
        overlays: ambience.overlays,
        kinds: OVERLAY_LOOKS.map(|(_, blend, _)| Vec4::new(blend.code(), 0.0, 0.0, 0.0)),
    };
    let mut bytes = encase::UniformBuffer::new(Vec::<u8>::new());
    bytes.write(&uniform).expect("ambience uniform");
    let buffer = device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("ambience_uniform"),
        contents: bytes.as_ref(),
        usage: BufferUsages::UNIFORM,
    });

    let post = target.post_process_write();
    let layout = cache.get_bind_group_layout(&pipelines.layout);
    let group = device.create_bind_group(
        "ambience",
        &layout,
        &BindGroupEntries::with_indices((
            (0, post.source),
            (1, &light.0.default_view),
            (2, buffer.as_entire_binding()),
        )),
    );

    let mut pass = ctx
        .command_encoder()
        .begin_render_pass(&RenderPassDescriptor {
            label: Some("ambience_compose"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post.destination,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    pass.set_pipeline(compose);
    pass.set_bind_group(0, &group, &[]);
    pass.draw(0..3, 0..1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use econbox_age::{Age, OVERLAYS};
    use econbox_tile::{Ground, Ore, Tile};

    fn sand(size: u32) -> TileMap {
        TileMap::filled(size, size, Tile::bare(Ground::Sand))
    }

    fn up(map: &TileMap, y: u32) -> u32 {
        map.height - 1 - y
    }

    #[test]
    fn zones_are_eight_tiles_counted_from_the_bottom_left() {
        let map = sand(16);
        let zones = {
            let mut zones = LightZones::default();
            zones.reset(&map, 1);
            zones
        };

        assert_eq!((zones.columns, zones.rows), (2, 2));
        assert_eq!(zones.zone_of(&map, 0, 15), (0, 0));
        assert_eq!(zones.zone_of(&map, 9, 0), (1, 1));
        assert_eq!(ZoneLights::scan(&map, 0, 0).centre, Some((4, up(&map, 4))));
    }

    #[test]
    fn a_little_lava_glows_tile_by_tile_and_a_lot_glows_from_the_zone_centre() {
        let mut map = sand(8);
        for x in 0..4 {
            map.set(x, 7, Tile::bare(Ground::Lava2));
        }
        let mut blobs = Vec::new();
        ZoneLights::scan(&map, 0, 0).lava_blobs(&map, &mut blobs);
        assert_eq!(blobs.len(), 4);
        assert!(blobs.iter().all(|blob| blob.radius == LAVA_LIGHT));
        assert_eq!(blobs[0].pos, tile_centre(&map, 0.5, 7.5));

        map.set(4, 7, Tile::bare(Ground::Lava3));
        blobs.clear();
        ZoneLights::scan(&map, 0, 0).lava_blobs(&map, &mut blobs);
        assert_eq!(
            blobs,
            vec![LightBlob::at_tile(&map, 4.0, 3.0, LAVA_ZONE_LIGHT)]
        );
    }

    #[test]
    fn light_walls_and_glowing_minerals_cast_their_own_lights() {
        let mut map = sand(8);
        map.set(
            2,
            2,
            Tile {
                wall: Some(Wall::Light),
                ..Tile::bare(Ground::Sand)
            },
        );
        map.set(
            5,
            5,
            Tile {
                ore: Some(Ore::Mythril),
                ..Tile::bare(Ground::Sand)
            },
        );
        let zone = ZoneLights::scan(&map, 0, 0);
        let mut blobs = Vec::new();
        zone.wall_blobs(&map, &mut blobs);
        zone.thing_blobs(&map, &mut blobs);

        assert_eq!(
            blobs,
            vec![
                LightBlob::at_tile(&map, 2.0, 2.0, WALL_LIGHT),
                LightBlob::at_tile(&map, 5.0, 5.0, 0.1),
            ]
        );
    }

    #[test]
    fn a_light_of_radius_one_is_a_disc_twenty_five_tiles_wide_each_way() {
        let blob = LightBlob {
            pos: Vec2::new(100.0, 0.0),
            radius: 1.0,
        };
        let area = Rect::from_center_size(Vec2::ZERO, Vec2::new(800.0, 400.0));
        let quad = blob_quad(&blob, Vec2::ZERO, area).expect("on screen");

        assert_eq!(blob.half_size(), 25.0 * TILE_SIZE);
        assert!((quad.x - 0.25).abs() < 1e-6);
        assert!((quad.z - 25.0 * TILE_SIZE / 400.0).abs() < 1e-6);
        assert!((quad.w - 25.0 * TILE_SIZE / 200.0).abs() < 1e-6);

        let far = LightBlob {
            pos: Vec2::new(2000.0, 0.0),
            radius: 0.2,
        };
        assert_eq!(blob_quad(&far, Vec2::ZERO, area), None);
    }

    #[test]
    fn the_overlays_stack_in_the_order_and_blend_of_worldbox_era_effects() {
        let order: Vec<&str> = OVERLAY_LOOKS.iter().map(|(o, _, _)| o.id()).collect();
        assert_eq!(
            order,
            [
                "night",
                "chaos",
                "moon",
                "magic",
                "sun",
                "rain_darkness",
                "winter",
                "ash",
                "rain"
            ]
        );
        let blend = |id: &str| {
            OVERLAY_LOOKS
                .iter()
                .find(|(o, _, _)| o.id() == id)
                .map(|(_, blend, _)| *blend)
        };
        assert_eq!(blend("night"), Some(Blend::Multiply));
        assert_eq!(blend("sun"), Some(Blend::Additive));
        assert_eq!(blend("ash"), Some(Blend::Alpha));
        for overlay in OVERLAYS {
            assert!(OVERLAY_LOOKS.iter().any(|(o, _, _)| *o == overlay));
        }
    }

    #[test]
    fn the_light_layer_shows_at_six_tenths_of_the_night_in_the_age_colour() {
        let mut ages = Ages::new(1);
        ages.set_current(Age::Moon, false);
        ages.night = 0.5;
        let light = light_color(&ages);

        assert!((light.w - 0.3).abs() < 1e-6);
        assert!((light.y - 1.0).abs() < 1e-6);
        assert!((light.x - 0x8d as f32 / 255.0).abs() < 1e-6);

        ages.set_current(Age::Hope, false);
        ages.night = 0.0;
        assert_eq!(light_color(&ages).w, 0.0);
        assert!(
            !Ambience {
                light: light_color(&ages),
                overlays: overlay_colors(&ages),
                blobs: Vec::new(),
            }
            .active()
        );
    }

    #[test]
    fn the_light_texture_is_a_third_of_the_screen() {
        assert_eq!(light_size(1920, 1080), UVec2::new(640, 360));
        assert_eq!(light_size(2, 2), UVec2::new(1, 1));
    }

    fn lit_world(night_lights: bool) -> App {
        let mut map = sand(64);
        for y in 30..34 {
            for x in 30..34 {
                map.set(x, y, Tile::bare(Ground::Lava3));
            }
        }
        let mut ages = Ages::new(2);
        ages.set_current(Age::Dark, false);
        ages.night = 0.6;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(map)
            .insert_resource(ages)
            .insert_resource(AgeSettings {
                night_lights,
                ..AgeSettings::default()
            })
            .init_resource::<Drawn>()
            .init_resource::<MapMode>()
            .init_resource::<LightBlobs>()
            .init_resource::<LightZones>()
            .init_resource::<Ambience>()
            .add_message::<Painted>()
            .add_systems(Update, (forget, gather).chain());
        app.world_mut().spawn((
            WorldCamera,
            Transform::default(),
            Projection::Orthographic(OrthographicProjection {
                area: Rect::from_center_size(Vec2::ZERO, Vec2::splat(64.0 * TILE_SIZE)),
                ..OrthographicProjection::default_2d()
            }),
        ));
        app
    }

    #[test]
    fn a_dark_age_lights_the_lava_in_view() {
        let mut app = lit_world(true);
        app.update();
        let ambience = app.world().resource::<Ambience>();

        assert!((ambience.light.w - 0.36).abs() < 1e-6);
        assert_eq!(ambience.blobs.len(), 16);
        assert!(ambience.active());
    }

    #[test]
    fn with_night_lights_off_the_night_still_darkens_but_nothing_glows() {
        let mut app = lit_world(false);
        app.update();
        let ambience = app.world().resource::<Ambience>();

        assert!(ambience.light.w > 0.0);
        assert!(ambience.blobs.is_empty());
    }

    #[test]
    fn painting_a_zone_rescans_its_lights() {
        let mut app = lit_world(true);
        app.update();
        app.world_mut()
            .resource_mut::<TileMap>()
            .set(10, 10, Tile::bare(Ground::Lava2));
        app.world_mut()
            .write_message(Painted::new(vec![econbox_tilemap::TilePos::new(10, 10)]));
        app.update();

        assert_eq!(app.world().resource::<Ambience>().blobs.len(), 17);
    }
}
