use bevy::asset::{embedded_asset, load_embedded_asset};
use bevy::core_pipeline::FullscreenShader;
use bevy::core_pipeline::schedule::{Core2d, Core2dSystems};
use bevy::core_pipeline::tonemapping::tonemapping;
use bevy::prelude::*;
use bevy::render::camera::ExtractedCamera;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_resource::binding_types::{sampler, texture_2d, uniform_buffer};
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderContext, RenderDevice, ViewQuery};
use bevy::render::texture::{CachedTexture, TextureCache};
use bevy::render::view::ViewTarget;
use bevy::render::{Render, RenderApp, RenderStartup, RenderSystems};
use bevy::shader::Shader;
use econbox_camera::WorldCamera;

pub const WORK_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;
pub const PRECOMPOSE_HEIGHT_CAP: u32 = 720;
pub const PRECOMPOSE_DIVISOR: f32 = 5.0;

#[derive(Component, Debug, Clone, Copy, PartialEq, ExtractComponent)]
#[extract_app(RenderApp)]
pub struct ScreenEffects {
    pub bloom: bool,
    pub vignette: bool,
    pub bloom_threshold: f32,
    pub bloom_intensity: f32,
    pub bloom_tint: [f32; 3],
    pub bloom_size: u32,
    pub bloom_luma: [f32; 3],
    pub vignette_begin: f32,
    pub vignette_expand: f32,
    pub vignette_color: [f32; 4],
}

impl Default for ScreenEffects {
    fn default() -> Self {
        Self {
            bloom: true,
            vignette: true,
            bloom_threshold: 0.423,
            bloom_intensity: 0.78,
            bloom_tint: [1.0, 1.0, 1.0],
            bloom_size: 128,
            bloom_luma: [1.0 / 3.0; 3],
            vignette_begin: 0.151,
            vignette_expand: 1.696,
            vignette_color: [0.0, 0.0, 0.0, 1.0],
        }
    }
}

impl ScreenEffects {
    pub fn threshold(&self) -> Vec4 {
        let scale = 1.0 / (1.0 - self.bloom_threshold);
        let [x, y, z] = self.bloom_luma;
        Vec4::new(
            x * scale,
            y * scale,
            z * scale,
            -self.bloom_threshold * scale,
        )
    }

    pub fn vignette_shape(&self) -> Vec2 {
        let begin = self.vignette_begin * self.vignette_begin;
        let end = self.vignette_begin + self.vignette_expand;
        let inverse = 1.0 / (end - begin);
        Vec2::new(4.0 * inverse * inverse, -inverse * begin)
    }

    pub fn vignette_premultiplied(&self) -> Vec4 {
        let [r, g, b, a] = self.vignette_color;
        Vec4::new(r * a, g * a, b * a, a)
    }

    pub fn blur_sizes(&self) -> [u32; 5] {
        let base = self.bloom_size.max(4);
        [base, base / 2, base / 4, base / 2, base]
    }

    pub fn active(&self) -> bool {
        self.bloom || self.vignette
    }
}

pub fn precompose_size(width: u32, height: u32) -> UVec2 {
    let scale = height.min(PRECOMPOSE_HEIGHT_CAP) as f32 / height.max(1) as f32;
    UVec2::new(
        ((width as f32 * scale / PRECOMPOSE_DIVISOR).round() as u32).max(1),
        ((height as f32 * scale / PRECOMPOSE_DIVISOR).round() as u32).max(1),
    )
}

pub fn vignette_fade(effects: &ScreenEffects, uv: Vec2) -> f32 {
    let shape = effects.vignette_shape();
    let centre = uv - Vec2::splat(0.5);
    (centre.dot(centre) * shape.x + shape.y).clamp(0.0, 1.0)
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenOptions {
    pub bloom: bool,
    pub vignette: bool,
    pub transition: bool,
    pub shadows: bool,
}

impl Default for ScreenOptions {
    fn default() -> Self {
        Self {
            bloom: true,
            vignette: true,
            transition: true,
            shadows: true,
        }
    }
}

pub struct SleekPlugin;

impl Plugin for SleekPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "sleek.wgsl");

        app.init_resource::<ScreenOptions>()
            .add_plugins(ExtractComponentPlugin::<ScreenEffects>::default())
            .add_systems(PostUpdate, equip);

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
                sleek.after(tonemapping).in_set(Core2dSystems::PostProcess),
            );
    }
}

fn equip(
    mut commands: Commands,
    options: Res<ScreenOptions>,
    mut cameras: Query<(Entity, Option<&mut ScreenEffects>), With<WorldCamera>>,
) {
    for (entity, effects) in &mut cameras {
        match effects {
            Some(mut effects) => {
                if effects.bloom != options.bloom || effects.vignette != options.vignette {
                    effects.bloom = options.bloom;
                    effects.vignette = options.vignette;
                }
            }
            None => {
                commands.entity(entity).insert(ScreenEffects {
                    bloom: options.bloom,
                    vignette: options.vignette,
                    ..ScreenEffects::default()
                });
            }
        }
    }
}

#[derive(ShaderType, Clone, Copy, Debug, Default)]
struct PassUniform {
    texel: Vec4,
    threshold: Vec4,
    bloom: Vec4,
    shape: Vec4,
    color: Vec4,
}

#[derive(Resource)]
struct SleekPipelines {
    single: BindGroupLayoutDescriptor,
    double: BindGroupLayoutDescriptor,
    sampler: Sampler,
    brightpass: CachedRenderPipelineId,
    downsample: CachedRenderPipelineId,
    precompose: CachedRenderPipelineId,
    compose: Vec<(TextureFormat, CachedRenderPipelineId)>,
}

fn pipeline(
    label: &'static str,
    layout: &BindGroupLayoutDescriptor,
    fullscreen: &FullscreenShader,
    shader: &Handle<Shader>,
    entry: &'static str,
    format: TextureFormat,
) -> RenderPipelineDescriptor {
    RenderPipelineDescriptor {
        label: Some(label.into()),
        layout: vec![layout.clone()],
        vertex: fullscreen.to_vertex_state(),
        fragment: Some(FragmentState {
            shader: shader.clone(),
            entry_point: Some(entry.into()),
            targets: vec![Some(ColorTargetState {
                format,
                blend: None,
                write_mask: ColorWrites::ALL,
            })],
            ..default()
        }),
        ..default()
    }
}

fn init_pipelines(
    mut commands: Commands,
    device: Res<RenderDevice>,
    fullscreen: Res<FullscreenShader>,
    assets: Res<AssetServer>,
    cache: Res<PipelineCache>,
) {
    let single = BindGroupLayoutDescriptor::new(
        "sleek_single_layout",
        &BindGroupLayoutEntries::with_indices(
            ShaderStages::FRAGMENT,
            (
                (0, texture_2d(TextureSampleType::Float { filterable: true })),
                (1, sampler(SamplerBindingType::Filtering)),
                (2, uniform_buffer::<PassUniform>(false)),
            ),
        ),
    );
    let double = BindGroupLayoutDescriptor::new(
        "sleek_double_layout",
        &BindGroupLayoutEntries::with_indices(
            ShaderStages::FRAGMENT,
            (
                (0, texture_2d(TextureSampleType::Float { filterable: true })),
                (1, sampler(SamplerBindingType::Filtering)),
                (2, uniform_buffer::<PassUniform>(false)),
                (3, texture_2d(TextureSampleType::Float { filterable: true })),
            ),
        ),
    );
    let sampler = device.create_sampler(&SamplerDescriptor {
        min_filter: FilterMode::Linear,
        mag_filter: FilterMode::Linear,
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        ..default()
    });
    let shader: Handle<Shader> = load_embedded_asset!(assets.as_ref(), "sleek.wgsl");

    let queue = |label, layout: &BindGroupLayoutDescriptor, entry, format| {
        cache.queue_render_pipeline(pipeline(label, layout, &fullscreen, &shader, entry, format))
    };

    let compose = [
        TextureFormat::Rgba8UnormSrgb,
        TextureFormat::Bgra8UnormSrgb,
        TextureFormat::Rgba16Float,
    ]
    .into_iter()
    .map(|format| (format, queue("sleek_compose", &double, "compose", format)))
    .collect();

    commands.insert_resource(SleekPipelines {
        brightpass: queue("sleek_brightpass", &single, "brightpass", WORK_FORMAT),
        downsample: queue("sleek_downsample", &single, "downsample", WORK_FORMAT),
        precompose: queue("sleek_precompose", &single, "precompose", WORK_FORMAT),
        compose,
        single,
        double,
        sampler,
    });
}

#[derive(Component)]
struct SleekTextures {
    blur: Vec<CachedTexture>,
    pre: CachedTexture,
}

fn work_texture(
    cache: &mut TextureCache,
    device: &RenderDevice,
    label: &'static str,
    size: UVec2,
) -> CachedTexture {
    cache.get(
        device,
        TextureDescriptor {
            label: Some(label),
            size: Extent3d {
                width: size.x.max(1),
                height: size.y.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: WORK_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        },
    )
}

fn prepare_textures(
    mut commands: Commands,
    mut cache: ResMut<TextureCache>,
    device: Res<RenderDevice>,
    views: Query<(Entity, &ExtractedCamera, &ScreenEffects)>,
) {
    for (entity, camera, effects) in &views {
        let Some(size) = camera.physical_viewport_size else {
            continue;
        };

        let blur = effects
            .blur_sizes()
            .into_iter()
            .map(|side| work_texture(&mut cache, &device, "sleek_blur", UVec2::splat(side)))
            .collect();
        let pre = work_texture(
            &mut cache,
            &device,
            "sleek_precompose",
            precompose_size(size.x, size.y),
        );

        commands.entity(entity).insert(SleekTextures { blur, pre });
    }
}

fn uniform_buffer_of(device: &RenderDevice, value: PassUniform) -> Buffer {
    let mut bytes = encase::UniformBuffer::new(Vec::<u8>::new());
    bytes.write(&value).expect("sleek uniform");
    device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("sleek_pass_uniform"),
        contents: bytes.as_ref(),
        usage: BufferUsages::UNIFORM,
    })
}

fn draw(
    ctx: &mut RenderContext,
    pipeline: &RenderPipeline,
    bind_group: &BindGroup,
    target: &TextureView,
    label: &'static str,
) {
    let mut pass = ctx
        .command_encoder()
        .begin_render_pass(&RenderPassDescriptor {
            label: Some(label),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, bind_group, &[]);
    pass.draw(0..3, 0..1);
}

fn sleek(
    view: ViewQuery<(&ViewTarget, &ScreenEffects, &SleekTextures)>,
    pipelines: Option<Res<SleekPipelines>>,
    cache: Res<PipelineCache>,
    mut ctx: RenderContext,
) {
    let Some(pipelines) = pipelines else { return };
    let (target, effects, textures) = view.into_inner();

    if !effects.active() {
        return;
    }

    let format = target.main_texture_format();
    let Some(compose_id) = pipelines
        .compose
        .iter()
        .find(|(candidate, _)| *candidate == format)
        .map(|(_, id)| *id)
    else {
        return;
    };

    let (Some(brightpass), Some(downsample), Some(precompose), Some(compose)) = (
        cache.get_render_pipeline(pipelines.brightpass),
        cache.get_render_pipeline(pipelines.downsample),
        cache.get_render_pipeline(pipelines.precompose),
        cache.get_render_pipeline(compose_id),
    ) else {
        return;
    };

    let device = ctx.render_device().clone();
    let single = cache.get_bind_group_layout(&pipelines.single);
    let double = cache.get_bind_group_layout(&pipelines.double);
    let flags = Vec4::new(
        0.0,
        0.0,
        if effects.bloom { 1.0 } else { 0.0 },
        if effects.vignette { 1.0 } else { 0.0 },
    );
    let shape = effects.vignette_shape();
    let base = PassUniform {
        threshold: effects.threshold(),
        bloom: Vec4::new(
            effects.bloom_tint[0],
            effects.bloom_tint[1],
            effects.bloom_tint[2],
            effects.bloom_intensity,
        ),
        shape: Vec4::new(shape.x, shape.y, flags.z, flags.w),
        color: effects.vignette_premultiplied(),
        ..default()
    };

    let single_pass = |source: &TextureView, size: UVec2| {
        let buffer = uniform_buffer_of(
            &device,
            PassUniform {
                texel: Vec4::new(1.0 / size.x as f32, 1.0 / size.y as f32, 0.0, 0.0),
                ..base
            },
        );
        device.create_bind_group(
            "sleek_single",
            &single,
            &BindGroupEntries::with_indices((
                (0, source),
                (1, &pipelines.sampler),
                (2, buffer.as_entire_binding()),
            )),
        )
    };

    if effects.bloom {
        let sizes = effects.blur_sizes();
        let first = single_pass(target.main_texture_view(), UVec2::splat(sizes[0]));
        draw(
            &mut ctx,
            brightpass,
            &first,
            &textures.blur[0].default_view,
            "sleek_brightpass",
        );

        for (pair, side) in textures.blur.windows(2).zip(sizes.into_iter().skip(1)) {
            let group = single_pass(&pair[0].default_view, UVec2::splat(side));
            draw(
                &mut ctx,
                downsample,
                &group,
                &pair[1].default_view,
                "sleek_downsample",
            );
        }
    }

    let pre_source = if effects.bloom {
        &textures.blur[textures.blur.len() - 1].default_view
    } else {
        target.main_texture_view()
    };
    let pre_size = UVec2::new(textures.pre.texture.width(), textures.pre.texture.height());
    let group = single_pass(pre_source, pre_size);
    draw(
        &mut ctx,
        precompose,
        &group,
        &textures.pre.default_view,
        "sleek_precompose",
    );

    let post = target.post_process_write();
    let buffer = uniform_buffer_of(&device, base);
    let group = device.create_bind_group(
        "sleek_double",
        &double,
        &BindGroupEntries::with_indices((
            (0, &textures.pre.default_view),
            (1, &pipelines.sampler),
            (2, buffer.as_entire_binding()),
            (3, post.source),
        )),
    );
    draw(&mut ctx, compose, &group, post.destination, "sleek_compose");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_defaults_are_worldbox_camera_renderer_settings() {
        let effects = ScreenEffects::default();

        assert!(effects.bloom && effects.vignette);
        assert_eq!(effects.blur_sizes(), [128, 64, 32, 64, 128]);
        assert!((effects.bloom_threshold - 0.423).abs() < 1e-6);
        assert!((effects.bloom_intensity - 0.78).abs() < 1e-6);
    }

    #[test]
    fn the_threshold_vector_maps_the_threshold_to_zero_and_white_to_one() {
        let threshold = ScreenEffects::default().threshold();
        let weight = |level: f32| Vec4::new(level, level, level, 1.0).dot(threshold);

        assert!(weight(0.423).abs() < 1e-5);
        assert!((weight(1.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn the_vignette_is_clear_in_the_middle_and_dark_in_the_corners() {
        let effects = ScreenEffects::default();

        assert_eq!(vignette_fade(&effects, Vec2::splat(0.5)), 0.0);
        let corner = vignette_fade(&effects, Vec2::ZERO);
        assert!((corner - 0.5885).abs() < 1e-3, "{corner}");
        let side = vignette_fade(&effects, Vec2::new(0.0, 0.5));
        assert!(side > 0.25 && side < corner, "{side}");
    }

    #[test]
    fn the_precompose_texture_is_a_fifth_of_a_720_line_screen() {
        assert_eq!(precompose_size(2480, 1438), UVec2::new(248, 144));
        assert_eq!(precompose_size(1280, 720), UVec2::new(256, 144));
        assert_eq!(precompose_size(800, 600), UVec2::new(160, 120));
    }
}
