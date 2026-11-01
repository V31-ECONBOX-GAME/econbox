//! WorldBox's drops: what clouds and powers let fall, how it falls, and what it does where it lands.

pub mod art;
pub mod kind;
pub mod land;

pub use kind::{DROPS, DropKind};
pub use land::Landing;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use econbox_camera::{TILE_SIZE, WorldBounds};
use econbox_clock::{Frames, WorldClock};
use econbox_creature::{Afflict, Census, Creature, Harm, Relieve, Spawn, Strike, tile_of};
use econbox_noise::Randy;
use econbox_temperature::Frost;
use econbox_tilemap::{Douse, Flash, Ignite, NewWorld, Painted, TileMap, TilePos};
use kind::{DROP_SCALE, FALLING_HEIGHT, FALLING_SPEED_RANDOM};

pub const DROP_LAYER: f32 = 4.3;
pub const HEIGHT_LIFT: f32 = 0.001;
pub const FLASH_STATE: i32 = 14;
pub const FALL_RATE: f32 = 15.0;

#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct SpawnDrop {
    pub kind: DropKind,
    pub spot: IVec2,
    pub height: Option<f32>,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Landed {
    pub kind: DropKind,
    pub pos: TilePos,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Falling {
    pub kind: DropKind,
    pub at: Vec2,
    pub height: f32,
    pub speed: f32,
    pub scale: f32,
    pub frame: usize,
    pub frames: usize,
    pub flip: bool,
    pub next_frame: f32,
    pub every: f32,
}

impl Falling {
    pub fn launch(kind: DropKind, spot: IVec2, height: Option<f32>, randy: &mut Randy) -> Self {
        let speed = kind.falling_speed() + randy.random_float(0.0, FALLING_SPEED_RANDOM);
        let flip = kind.random_flip() && randy.random_bool();
        let every = kind
            .animation()
            .map_or(0.0, |(base, spread)| base + randy.random_float(0.0, spread));
        let frames = art::frames(kind).len();
        let frame = if kind.random_frame() {
            randy.random_int(0, frames as i32) as usize
        } else {
            0
        };
        let height = height.unwrap_or_else(|| {
            randy.random_float(FALLING_HEIGHT.0, FALLING_HEIGHT.1) as i32 as f32
        });

        Self {
            kind,
            at: spot.as_vec2(),
            height,
            speed,
            scale: DROP_SCALE,
            frame,
            frames,
            flip,
            next_frame: every,
            every,
        }
    }

    pub fn step(&mut self, dt: f32, randy: &mut Randy) -> bool {
        if self.kind.animation().is_some() && self.frames > 0 {
            if self.next_frame > 0.0 {
                self.next_frame -= dt;
            } else {
                self.next_frame = self.every;
                self.frame = (self.frame + 1) % self.frames;
            }
        }

        let mut fall = FALL_RATE * dt;
        fall *= if self.scale < 1.0 {
            self.speed / (self.scale * 2.0)
        } else {
            self.speed
        };

        if self.height < 0.0 {
            fall = 0.0;
        }

        self.height -= fall * self.scale;

        if self.kind.drifts() && fall > 0.0 && randy.random_bool() {
            if randy.random_bool() {
                self.at.x -= self.scale;
            } else {
                self.at.x += self.scale;
            }
        }

        if self.height <= 0.0 {
            self.height = 0.0;
            return true;
        }

        false
    }

    pub fn spot(&self) -> IVec2 {
        IVec2::new(self.at.x as i32, self.at.y as i32)
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct DropArt(pub Vec<(DropKind, Vec<(Handle<Image>, Vec2)>)>);

impl DropArt {
    pub fn frame(&self, kind: DropKind, frame: usize) -> Option<(Handle<Image>, Vec2)> {
        self.0
            .iter()
            .find(|(drawn, _)| *drawn == kind)
            .and_then(|(_, frames)| frames.get(frame).cloned())
    }
}

#[derive(Resource, Debug, Clone)]
pub struct DropDice(pub Randy);

impl Default for DropDice {
    fn default() -> Self {
        Self(Randy::new(0x000d_509e))
    }
}

pub fn spot_tile(spot: IVec2, width: u32, height: u32) -> Option<TilePos> {
    let inside = spot.x >= 0 && spot.y >= 0 && (spot.x as u32) < width && (spot.y as u32) < height;
    inside.then(|| TilePos::new(spot.x as u32, height - 1 - spot.y as u32))
}

fn sweep(
    mut commands: Commands,
    mut fresh: MessageReader<NewWorld>,
    left: Query<Entity, With<Falling>>,
) {
    if fresh.read().count() == 0 {
        return;
    }
    for entity in &left {
        commands.entity(entity).despawn();
    }
}

pub struct DropsPlugin;

impl Plugin for DropsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldClock>()
            .init_resource::<Frames>()
            .init_resource::<DropArt>()
            .init_resource::<DropDice>()
            .add_message::<SpawnDrop>()
            .add_message::<Landed>()
            .add_message::<Flash>()
            .add_message::<Painted>()
            .add_message::<Strike>()
            .add_message::<Ignite>()
            .add_message::<Douse>()
            .add_message::<Afflict>()
            .add_message::<Relieve>()
            .add_message::<Harm>()
            .add_message::<Spawn>()
            .add_message::<NewWorld>()
            .init_resource::<Frost>()
            .add_systems(Startup, draw)
            .add_systems(
                Update,
                (sweep, launch, fall, place)
                    .chain()
                    .run_if(resource_exists::<TileMap>),
            );
    }
}

fn draw(images: Option<ResMut<Assets<Image>>>, mut drawn: ResMut<DropArt>) {
    let Some(mut images) = images else { return };

    drawn.0 = DROPS
        .into_iter()
        .map(|kind| {
            let frames = art::frames(kind)
                .iter()
                .map(|frame| {
                    let size = Vec2::new(frame.width as f32, frame.height as f32);
                    (images.add(art::image(frame)), size)
                })
                .collect();
            (kind, frames)
        })
        .collect();
}

fn launch(mut commands: Commands, mut asked: MessageReader<SpawnDrop>, mut dice: ResMut<DropDice>) {
    for request in asked.read() {
        let falling = Falling::launch(request.kind, request.spot, request.height, &mut dice.0);
        commands.spawn((
            Sprite::default(),
            Transform::from_xyz(0.0, 0.0, DROP_LAYER),
            Visibility::Hidden,
            falling,
        ));
    }
}

#[derive(SystemParam)]
pub struct Aftermath<'w> {
    painted: MessageWriter<'w, Painted>,
    landed: MessageWriter<'w, Landed>,
    flashes: MessageWriter<'w, Flash>,
    ignite: MessageWriter<'w, Ignite>,
    douse: MessageWriter<'w, Douse>,
    afflict: MessageWriter<'w, Afflict>,
    relieve: MessageWriter<'w, Relieve>,
    harm: MessageWriter<'w, Harm>,
    spawn: MessageWriter<'w, Spawn>,
}

impl Aftermath<'_> {
    fn send(&mut self, landing: Landing, cells: &mut Vec<TilePos>) {
        cells.extend(landing.cells);
        for (pos, strength) in landing.flashes {
            self.flashes.write(Flash { pos, strength });
        }
        for pos in landing.ignite {
            self.ignite.write(Ignite { pos });
        }
        for pos in landing.douse {
            self.douse.write(Douse { pos });
        }
        for afflict in landing.afflict {
            self.afflict.write(afflict);
        }
        for relieve in landing.relieve {
            self.relieve.write(relieve);
        }
        for harm in landing.harm {
            self.harm.write(harm);
        }
        for spawn in landing.spawn {
            self.spawn.write(spawn);
        }
    }
}

pub fn crowd(pos: TilePos, others: &[TilePos]) -> usize {
    let chunk = |at: TilePos| (at.x / land::CHUNK_SIZE, at.y / land::CHUNK_SIZE);
    let (cx, cy) = chunk(pos);

    others
        .iter()
        .filter(|at| {
            let (x, y) = chunk(**at);
            x.abs_diff(cx) <= 1 && y.abs_diff(cy) <= 1
        })
        .count()
}

fn fall(
    mut commands: Commands,
    frames: Res<Frames>,
    clock: Res<WorldClock>,
    census: Option<Res<Census>>,
    mut dice: ResMut<DropDice>,
    mut frost: ResMut<Frost>,
    mut map: ResMut<TileMap>,
    mut drops: Query<(Entity, &mut Falling)>,
    creatures: Query<&Transform, With<Creature>>,
    mut aftermath: Aftermath,
) {
    let dt = clock.step();
    let mut cells = Vec::new();
    let mut down = Vec::new();
    let census = census.map(|census| *census).unwrap_or_default();
    let mut others: Option<Vec<TilePos>> = None;

    for _ in 0..frames.due {
        for (entity, mut drop) in &mut drops {
            if down.contains(&entity) {
                continue;
            }

            if !drop.step(dt, &mut dice.0) {
                continue;
            }

            down.push(entity);
            commands.entity(entity).despawn();

            let Some(pos) = spot_tile(drop.spot(), map.width, map.height) else {
                continue;
            };

            let near = if drop.kind == DropKind::LifeSeed {
                let others = others.get_or_insert_with(|| {
                    creatures
                        .iter()
                        .filter_map(|transform| tile_of(&map, transform.translation.truncate()))
                        .collect()
                });
                crowd(pos, others)
            } else {
                0
            };

            let landing = land::land(
                drop.kind,
                &mut map,
                &mut frost,
                pos,
                &census,
                near,
                &mut dice.0,
            );
            aftermath.send(landing, &mut cells);
            aftermath.landed.write(Landed {
                kind: drop.kind,
                pos,
            });
            aftermath.flashes.write(Flash {
                pos,
                strength: FLASH_STATE,
            });
        }
    }

    if !cells.is_empty() {
        aftermath.painted.write(Painted::new(cells));
    }
}

fn place(
    bounds: Res<WorldBounds>,
    art: Res<DropArt>,
    mut drops: Query<(&Falling, &mut Sprite, &mut Transform, &mut Visibility)>,
) {
    let origin = -bounds.half();

    for (drop, mut sprite, mut transform, mut visibility) in &mut drops {
        let lifted = Vec2::new(drop.at.x, drop.at.y + drop.height);
        let at = origin + lifted * TILE_SIZE;
        transform.translation = at.extend(DROP_LAYER + drop.height * HEIGHT_LIFT);

        if let Some((image, size)) = art.frame(drop.kind, drop.frame) {
            sprite.image = image;
            sprite.custom_size = Some(size * drop.scale * TILE_SIZE);
        }
        sprite.flip_x = drop.flip;
        *visibility = Visibility::Inherited;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: f32 = 0.02;

    fn fall_time(kind: DropKind, height: f32, seed: i32) -> f32 {
        let mut randy = Randy::new(seed);
        let mut drop = Falling::launch(kind, IVec2::new(50, 50), Some(height), &mut randy);
        let mut time = 0.0;

        while !drop.step(FRAME, &mut randy) {
            time += FRAME;
            assert!(time < 100.0);
        }

        time
    }

    #[test]
    fn a_cloud_drop_starts_ten_up_and_rain_lands_within_half_a_second() {
        for seed in 1..20 {
            let time = fall_time(DropKind::Rain, 10.0, seed);
            assert!((0.3..0.45).contains(&time), "{time}");
        }
    }

    #[test]
    fn snowflakes_float_down_for_seconds() {
        for seed in 1..20 {
            let time = fall_time(DropKind::Snow, 10.0, seed);
            assert!((1.5..4.6).contains(&time), "{time}");
        }
    }

    #[test]
    fn rain_falls_straight_and_snow_drifts_a_fifth_of_a_tile_at_a_time() {
        let mut randy = Randy::new(3);
        let mut rain = Falling::launch(DropKind::Rain, IVec2::new(9, 9), Some(10.0), &mut randy);
        let mut snow = Falling::launch(DropKind::Snow, IVec2::new(9, 9), Some(10.0), &mut randy);

        for _ in 0..20 {
            rain.step(FRAME, &mut randy);
            snow.step(FRAME, &mut randy);
        }

        assert_eq!(rain.at.x, 9.0);
        let drift = (snow.at.x - 9.0) / DROP_SCALE;
        assert!((drift - drift.round()).abs() < 1e-3);
        assert!(drift.abs() <= 20.0);
    }

    #[test]
    fn without_a_height_a_drop_starts_thirty_to_forty_five_up() {
        let mut randy = Randy::new(8);

        for _ in 0..200 {
            let drop = Falling::launch(DropKind::Rain, IVec2::ZERO, None, &mut randy);
            assert!((30.0..45.0).contains(&drop.height));
            assert_eq!(drop.height, drop.height.trunc());
        }
    }

    #[test]
    fn fire_cycles_its_frames_and_rain_keeps_one() {
        let mut randy = Randy::new(5);
        let mut fire = Falling::launch(DropKind::Fire, IVec2::ZERO, Some(40.0), &mut randy);
        let mut rain = Falling::launch(DropKind::Rain, IVec2::ZERO, Some(40.0), &mut randy);
        let (fire_start, rain_start) = (fire.frame, rain.frame);
        let mut seen = vec![fire_start];

        for _ in 0..40 {
            fire.step(FRAME, &mut randy);
            rain.step(FRAME, &mut randy);
            seen.push(fire.frame);
        }

        seen.dedup();
        assert!(seen.len() > 3);
        assert_eq!(rain.frame, rain_start);
    }

    #[test]
    fn spots_map_to_tiles_counted_from_the_top() {
        assert_eq!(spot_tile(IVec2::new(3, 0), 8, 8), Some(TilePos::new(3, 7)));
        assert_eq!(spot_tile(IVec2::new(-1, 0), 8, 8), None);
        assert_eq!(spot_tile(IVec2::new(0, 8), 8, 8), None);
    }
}
