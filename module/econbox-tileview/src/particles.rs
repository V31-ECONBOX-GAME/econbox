use crate::fire_view::{ScorchLayer, scorch};
use crate::quality::MapMode;
use crate::sleek::ScreenOptions;
use bevy::asset::{AssetPath, RenderAssetUsages, embedded_asset, embedded_path};
use bevy::camera::visibility::NoFrustumCulling;
use bevy::mesh::{Indices, MeshVertexBufferLayoutRef, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, BlendState, RenderPipelineDescriptor, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{
    AlphaMode2d, Material2d, Material2dKey, Material2dPipeline, Material2dPlugin,
};
use econbox_age::Dice;
use econbox_camera::TILE_SIZE;
use econbox_noise::perlin;
use econbox_tile::Ground;
use econbox_tilemap::{Emit, Glow, NewWorld, TileMap, TilePos};

pub const PARTICLE_LAYER: f32 = 1.5;
pub const MAX_PARTICLES: usize = 50;
pub const COOLDOWN: f32 = 0.2;
pub const COOLDOWN_RANDOM: f32 = 0.3;
pub const SIZE_BIRTH: f32 = 0.264_160_16;
pub const SIZE_END_SLOPE: f32 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Recipe {
    pub life: (f32, f32),
    pub size: f32,
    pub opacity: f32,
    pub force: Vec2,
    pub drift: f32,
    pub drift_frequency: f32,
    pub spin: f32,
    pub shown: f32,
    pub additive: bool,
    pub colors: &'static [(f32, [f32; 3])],
    pub alphas: &'static [(f32, [f32; 1])],
}

pub const FIRE: Recipe = Recipe {
    life: (4.0, 6.0),
    size: 8.77,
    opacity: 1.0,
    force: Vec2::new(0.0, 1.0),
    drift: 1.0,
    drift_frequency: 0.5,
    spin: 0.0,
    shown: 1.0,
    additive: true,
    colors: &[
        (0.0, [1.0, 0.984_544_75, 0.852_941_16]),
        (18697.0 / 65535.0, [0.946_960_9, 0.532_040_06, 0.240_298_95]),
        (31611.0 / 65535.0, [1.0, 0.051_538_456, 0.007_352_948]),
        (
            49922.0 / 65535.0,
            [0.132_352_95, 0.132_352_95, 0.132_352_95],
        ),
    ],
    alphas: &[
        (0.0, [0.0]),
        (8096.0 / 65535.0, [1.0]),
        (59367.0 / 65535.0, [1.0]),
        (1.0, [0.176_470_6]),
    ],
};

pub const SMOKE: Recipe = Recipe {
    life: (10.0, 10.0),
    size: 5.0,
    opacity: 0.537_254_9,
    force: Vec2::new(0.11, 0.19),
    drift: 0.2,
    drift_frequency: 0.1,
    spin: std::f32::consts::FRAC_PI_4,
    shown: 0.5,
    additive: false,
    colors: &[(0.0, [1.0, 1.0, 1.0]), (1.0, [1.0, 1.0, 1.0])],
    alphas: &[
        (0.0, [0.0]),
        (14456.0 / 65535.0, [0.470_588_24]),
        (43369.0 / 65535.0, [0.470_588_24]),
        (1.0, [0.0]),
    ],
};

pub fn recipe(glow: Glow) -> &'static Recipe {
    match glow {
        Glow::Fire => &FIRE,
        Glow::Smoke => &SMOKE,
    }
}

pub fn size_share(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let (t2, t3) = (t * t, t * t * t);
    (2.0 * t3 - 3.0 * t2 + 1.0) * SIZE_BIRTH + (3.0 * t2 - 2.0 * t3) + (t3 - t2) * SIZE_END_SLOPE
}

pub fn ramp<const N: usize>(keys: &[(f32, [f32; N])], t: f32) -> [f32; N] {
    let Some(&(first, start)) = keys.first() else {
        return [0.0; N];
    };
    if t <= first {
        return start;
    }
    for pair in keys.windows(2) {
        let ((from, low), (to, high)) = (pair[0], pair[1]);
        if t <= to {
            let share = if to > from {
                (t - from) / (to - from)
            } else {
                1.0
            };
            return std::array::from_fn(|i| low[i] + (high[i] - low[i]) * share);
        }
    }
    keys[keys.len() - 1].1
}

pub fn paint(recipe: &Recipe, under: [f32; 3], t: f32) -> [f32; 4] {
    let color = ramp(recipe.colors, t);
    let alpha = recipe.opacity * ramp(recipe.alphas, t)[0];
    let cover = if recipe.additive { 0.0 } else { alpha };
    let kept = 1.0 - cover;
    let [r, g, b] = std::array::from_fn(|i| {
        let mixed = (under[i] * kept + color[i] * alpha).min(1.0);
        (Srgba::gamma_function(mixed) - Srgba::gamma_function(under[i]) * kept).max(0.0)
    });
    [r, g, b, cover]
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Particle {
    pub at: Vec2,
    pub velocity: Vec2,
    pub age: f32,
    pub life: f32,
}

impl Particle {
    pub fn progress(self) -> f32 {
        (self.age / self.life).clamp(0.0, 1.0)
    }

    pub fn step(&mut self, recipe: &Recipe, seconds: f32) {
        self.age += seconds;
        self.velocity += recipe.force * seconds;
        let scale = recipe.drift_frequency;
        let wave = perlin(self.at.x * scale, self.at.y * scale) * 2.0 - 1.0;
        let sway = Vec2::new(wave * recipe.drift / scale, 0.0);
        self.at += (self.velocity + sway) * seconds;
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Emitter {
    pub cooldown: f32,
    pub live: Vec<Particle>,
}

impl Emitter {
    pub fn spawn(&mut self, recipe: &Recipe, at: Vec2, dice: &mut Dice) -> bool {
        if self.live.len() > MAX_PARTICLES || self.cooldown > 0.0 {
            return false;
        }
        self.cooldown = COOLDOWN + dice.float(0.0, COOLDOWN_RANDOM);
        self.live.push(Particle {
            at,
            velocity: Vec2::ZERO,
            age: 0.0,
            life: dice.float(recipe.life.0, recipe.life.1),
        });
        true
    }

    pub fn tick(&mut self, recipe: &Recipe, seconds: f32) {
        self.cooldown = (self.cooldown - seconds).max(0.0);
        for particle in &mut self.live {
            particle.step(recipe, seconds);
        }
        self.live.retain(|particle| particle.age < particle.life);
    }
}

#[derive(Resource, Debug, Clone)]
pub struct GlowParticles {
    pub fire: Emitter,
    pub smoke: Emitter,
    pub dice: Dice,
}

impl Default for GlowParticles {
    fn default() -> Self {
        Self {
            fire: Emitter::default(),
            smoke: Emitter::default(),
            dice: Dice::new(0x0067_106e),
        }
    }
}

impl GlowParticles {
    pub fn emit(&mut self, glow: Glow, at: Vec2) -> bool {
        let Self { fire, smoke, dice } = self;
        let emitter = match glow {
            Glow::Fire => fire,
            Glow::Smoke => smoke,
        };
        emitter.spawn(recipe(glow), at, dice)
    }
}

pub fn spot(map: &TileMap, pos: TilePos) -> Vec2 {
    Vec2::new(pos.x as f32, map.height.saturating_sub(1 + pos.y) as f32)
}

pub fn under(map: &TileMap, at: Vec2) -> [f32; 3] {
    let x = at.x.floor() as i64;
    let row = i64::from(map.height) - 1 - at.y.floor() as i64;
    let inside = x >= 0 && row >= 0 && x < i64::from(map.width) && row < i64::from(map.height);
    let color = if inside {
        map.at(x as u32, row as u32).color()
    } else {
        Ground::DeepOcean.color()
    };
    color.map(|channel| f32::from(channel) / 255.0)
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Quads {
    pub positions: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

impl Quads {
    pub fn push(&mut self, recipe: &Recipe, particle: Particle, map: &TileMap) {
        let t = particle.progress();
        let half = Vec2::new(map.width as f32, map.height as f32) * 0.5;
        let centre = (particle.at - half) * TILE_SIZE;
        let reach = recipe.size * size_share(t) * recipe.shown * 0.5 * TILE_SIZE;
        let turn = Vec2::from_angle(-recipe.spin * particle.age);
        let color = paint(recipe, under(map, particle.at), t);
        let base = self.positions.len() as u32;

        for corner in [
            Vec2::new(-reach, -reach),
            Vec2::new(reach, -reach),
            Vec2::new(reach, reach),
            Vec2::new(-reach, reach),
        ] {
            let at = centre + turn.rotate(corner);
            self.positions.push([at.x, at.y, 0.0]);
            self.colors.push(color);
        }
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    pub fn mesh(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
        .with_inserted_indices(Indices::U32(self.indices))
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone, Default)]
pub struct ParticleMaterial {}

impl Material2d for ParticleMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(
            AssetPath::from_path_buf(embedded_path!("particles.wgsl")).with_source("embedded"),
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
        if let Some(fragment) = &mut descriptor.fragment {
            for target in fragment.targets.iter_mut().flatten() {
                target.blend = Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING);
            }
        }
        Ok(())
    }
}

#[derive(Component)]
pub struct ParticleQuads;

fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ParticleMaterial>>,
) {
    commands.spawn((
        ParticleQuads,
        Mesh2d(meshes.add(Quads::default().mesh())),
        MeshMaterial2d(materials.add(ParticleMaterial::default())),
        Transform::from_xyz(0.0, 0.0, PARTICLE_LAYER),
        Visibility::Hidden,
        NoFrustumCulling,
    ));
}

pub fn enabled(options: &ScreenOptions, glow: Glow) -> bool {
    match glow {
        Glow::Fire => options.fire,
        Glow::Smoke => options.smoke,
    }
}

fn run(
    time: Res<Time>,
    map: Option<Res<TileMap>>,
    mode: Res<MapMode>,
    layer: Res<ScorchLayer>,
    options: Res<ScreenOptions>,
    mut fresh: MessageReader<NewWorld>,
    mut asked: MessageReader<Emit>,
    mut particles: ResMut<GlowParticles>,
    mut meshes: ResMut<Assets<Mesh>>,
    quads: Single<(&Mesh2d, &mut Visibility), With<ParticleQuads>>,
) {
    let Some(map) = map else {
        fresh.clear();
        asked.clear();
        return;
    };

    if fresh.read().count() > 0 {
        particles.fire = Emitter::default();
        particles.smoke = Emitter::default();
    }

    let seconds = time.delta_secs();
    particles.fire.tick(&FIRE, seconds);
    particles.smoke.tick(&SMOKE, seconds);

    for emit in asked.read() {
        let flashing = layer
            .flashes
            .contains_key(&(emit.pos.y * map.width + emit.pos.x));
        if enabled(&options, emit.glow) && !mode.low_res && !(emit.unless_flashing && flashing) {
            particles.emit(emit.glow, spot(&map, emit.pos));
        }
    }

    let (mesh, mut visibility) = quads.into_inner();
    let mut built = Quads::default();
    for particle in particles.smoke.live.iter().rev() {
        built.push(&SMOKE, *particle, &map);
    }
    for particle in particles.fire.live.iter().rev() {
        built.push(&FIRE, *particle, &map);
    }

    let wanted = if built.positions.is_empty() {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    visibility.set_if_neq(wanted);

    if wanted == Visibility::Inherited
        && let Some(mut target) = meshes.get_mut(&mesh.0)
    {
        *target = built.mesh();
    }
}

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "particles.wgsl");
        app.add_plugins(Material2dPlugin::<ParticleMaterial>::default())
            .init_resource::<GlowParticles>()
            .add_message::<Emit>()
            .add_message::<NewWorld>()
            .add_systems(Startup, spawn)
            .add_systems(PostUpdate, run.before(scorch));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use econbox_tile::Tile;

    #[test]
    fn a_particle_grows_from_a_quarter_to_its_full_size() {
        assert!((size_share(0.0) - SIZE_BIRTH).abs() < 1e-6);
        assert!((size_share(1.0) - 1.0).abs() < 1e-6);
        assert!(size_share(0.5) > SIZE_BIRTH && size_share(0.5) < 1.0);
    }

    #[test]
    fn the_gradients_hit_their_keys_and_stay_inside_them() {
        assert_eq!(ramp(FIRE.alphas, 0.0), [0.0]);
        assert_eq!(ramp(FIRE.alphas, 0.5), [1.0]);
        assert_eq!(ramp(FIRE.colors, 1.0), [0.132_352_95; 3]);
        assert_eq!(ramp(SMOKE.colors, 0.5), [1.0; 3]);
        assert_eq!(ramp(SMOKE.alphas, 1.0), [0.0]);
    }

    #[test]
    fn fire_only_adds_light_and_smoke_covers_what_is_under_it() {
        let under = [0.2, 0.44, 0.8];

        let fire = paint(&FIRE, under, 0.4);
        assert_eq!(fire[3], 0.0);
        assert!(fire[0] > fire[2]);

        let smoke = paint(&SMOKE, under, 0.5);
        let alpha = SMOKE.opacity * 0.470_588_24;
        assert!((smoke[3] - alpha).abs() < 1e-6);
    }

    #[test]
    fn the_paint_lands_on_unitys_gamma_blend() {
        let under = [0.2, 0.44, 0.8];
        let linear = under.map(Srgba::gamma_function);
        let smoke = paint(&SMOKE, under, 0.5);
        let alpha = smoke[3];
        for i in 0..3 {
            let shown = smoke[i] + linear[i] * (1.0 - alpha);
            let wanted = Srgba::gamma_function(under[i] * (1.0 - alpha) + alpha);
            assert!((shown - wanted).abs() < 1e-5);
        }
    }

    #[test]
    fn fire_rises_faster_than_smoke_and_smoke_leans_right() {
        let start = Particle {
            at: Vec2::new(10.0, 10.0),
            velocity: Vec2::ZERO,
            age: 0.0,
            life: 10.0,
        };
        let (mut fire, mut smoke) = (start, start);
        for _ in 0..240 {
            fire.step(&FIRE, 1.0 / 60.0);
            smoke.step(&SMOKE, 1.0 / 60.0);
        }

        assert!((fire.velocity.y - 4.0).abs() < 1e-3);
        assert!(fire.at.y > smoke.at.y);
        assert!(smoke.velocity.x > 0.0);
    }

    #[test]
    fn an_emitter_waits_out_its_cooldown_and_stops_past_fifty() {
        let mut dice = Dice::new(7);
        let mut emitter = Emitter::default();

        assert!(emitter.spawn(&FIRE, Vec2::ZERO, &mut dice));
        assert!((COOLDOWN..COOLDOWN + COOLDOWN_RANDOM).contains(&emitter.cooldown));
        assert!(!emitter.spawn(&FIRE, Vec2::ZERO, &mut dice));
        emitter.tick(&FIRE, COOLDOWN + COOLDOWN_RANDOM);
        assert!(emitter.spawn(&FIRE, Vec2::ZERO, &mut dice));

        let spare = emitter.live[0];
        emitter.live = vec![spare; MAX_PARTICLES + 1];
        emitter.cooldown = 0.0;
        assert!(!emitter.spawn(&FIRE, Vec2::ZERO, &mut dice));
    }

    #[test]
    fn a_particle_dies_at_the_end_of_its_life() {
        let mut emitter = Emitter {
            cooldown: 0.0,
            live: vec![Particle {
                at: Vec2::ZERO,
                velocity: Vec2::ZERO,
                age: 0.0,
                life: 1.0,
            }],
        };

        emitter.tick(&SMOKE, 0.5);
        assert_eq!(emitter.live.len(), 1);
        emitter.tick(&SMOKE, 0.6);
        assert!(emitter.live.is_empty());
    }

    #[test]
    fn spots_count_rows_from_the_bottom_and_off_map_is_ocean() {
        let map = TileMap::filled(8, 8, Tile::bare(Ground::Sand));

        assert_eq!(spot(&map, TilePos::new(3, 0)), Vec2::new(3.0, 7.0));
        assert_eq!(spot(&map, TilePos::new(3, 7)), Vec2::new(3.0, 0.0));
        let sand = Ground::Sand.color().map(|c| f32::from(c) / 255.0);
        let sea = Ground::DeepOcean.color().map(|c| f32::from(c) / 255.0);
        assert_eq!(under(&map, Vec2::new(3.5, 0.5)), sand);
        assert_eq!(under(&map, Vec2::new(-1.0, 3.0)), sea);
        assert_eq!(under(&map, Vec2::new(3.0, 9.0)), sea);
    }

    #[test]
    fn smoke_shows_only_the_middle_of_its_quad_and_turns() {
        let map = TileMap::filled(8, 8, Tile::bare(Ground::Sand));
        let particle = Particle {
            at: Vec2::new(4.0, 4.0),
            velocity: Vec2::ZERO,
            age: 0.0,
            life: 10.0,
        };
        let mut smoke = Quads::default();
        smoke.push(&SMOKE, particle, &map);
        let mut fire = Quads::default();
        fire.push(&FIRE, particle, &map);

        let side = |quads: &Quads| quads.positions[1][0] - quads.positions[0][0];
        assert!((side(&smoke) - SMOKE.size * SIZE_BIRTH * 0.5 * TILE_SIZE).abs() < 1e-3);
        assert!((side(&fire) - FIRE.size * SIZE_BIRTH * TILE_SIZE).abs() < 1e-3);
        assert_eq!(smoke.indices.len(), 6);

        let mut turned = Quads::default();
        turned.push(
            &SMOKE,
            Particle {
                age: 1.0,
                ..particle
            },
            &map,
        );
        assert!((turned.positions[0][1] - turned.positions[1][1]).abs() > 1.0);

        let mut older = Quads::default();
        older.push(
            &FIRE,
            Particle {
                age: 1.0,
                ..particle
            },
            &map,
        );
        assert_eq!(older.positions[0][1], older.positions[1][1]);
    }
}
