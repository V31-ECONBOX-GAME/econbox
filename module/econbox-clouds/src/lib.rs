//! WorldBox's drifting clouds: spawned by the world age, carried east, faded by zoom, raining what they carry.

pub mod age;
pub mod art;
pub mod cloud;
pub mod dice;
pub mod kind;
pub mod rain;

pub use age::{AGES, Age};
pub use art::{PUFFS, Puff};
pub use cloud::{Cloud, Deeds};
pub use kind::{CloudKind, Drop, KINDS};

use bevy::prelude::*;
use cloud::{SHADOW_OFFSET, over_tile, spawn_spot, spot_tile};
use dice::Dice;
use econbox_brush::{Brush, ForcedBrush, Power, Tool};
use econbox_camera::{PointerOverUi, TILE_SIZE, WorldBounds, WorldCamera};
use econbox_clock::WorldClock;
use econbox_creature::Strike;
use econbox_cursor::Hovered;
use econbox_tile::Ground;
use econbox_tilemap::{Painted, TileMap, TilePos};
use std::collections::HashSet;

pub const SHADOW_LAYER: f32 = 4.0;
pub const CLOUD_LAYER: f32 = 4.5;
pub const STACK: f32 = 0.0002;
pub const STACKED: u64 = 1000;
pub const LIMIT: usize = 200;
pub const FRAME: f32 = 1.0 / 60.0;
pub const STEP: f32 = 0.02;
pub const CATCH_UP: u32 = 4;
pub const SHADOW_SQUASH: f32 = 0.5;
pub const SHADOW_SHARE: f32 = 0.5;
pub const SHADOW_COLOR: [u8; 3] = [62, 66, 82];
pub const SPONGE_REACH: f32 = 10.0;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WorldAge(pub Age);

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloudLaws {
    pub clouds: bool,
    pub disasters: bool,
}

impl Default for CloudLaws {
    fn default() -> Self {
        Self {
            clouds: true,
            disasters: true,
        }
    }
}

impl CloudLaws {
    pub fn allow(self, kind: CloudKind) -> bool {
        (!kind.normal() || self.clouds) && (!kind.disaster() || self.disasters)
    }
}

#[derive(Resource, Debug, Clone)]
pub struct Sky {
    pub timer: f32,
    pub lag: f32,
    pub dice: Dice,
    pub serial: u64,
    pub size: Option<Vec2>,
}

impl Default for Sky {
    fn default() -> Self {
        Self {
            timer: Age::default().cloud_interval(),
            lag: 0.0,
            dice: Dice::new(0x00c1_00d5),
            serial: 0,
            size: None,
        }
    }
}

impl Sky {
    pub fn due(&mut self, dt: f32, age: Age) -> bool {
        if self.timer > 0.0 {
            self.timer -= dt;
            if self.timer > 0.0 {
                return false;
            }
        }

        let interval = age.cloud_interval();
        self.timer += interval + self.dice.range(0.0, interval);
        true
    }

    pub fn choose(&mut self, age: Age, laws: CloudLaws) -> Option<CloudKind> {
        let roll = self.dice.roll();
        age.pick(roll).filter(|kind| laws.allow(*kind))
    }

    pub fn make(&mut self, kind: CloudKind, at: Option<Vec2>, height: f32) -> Cloud {
        let puff = self.dice.pick(kind.puffs()).unwrap_or(Puff::Broad);
        let flip = self.dice.coin();
        let (low, high) = kind.speed();
        let speed = self.dice.range(low, high);
        let at = at.unwrap_or_else(|| spawn_spot(&mut self.dice, height));
        self.serial += 1;

        Cloud::new(kind, puff, at, speed, flip, self.serial)
    }

    pub fn steps(&mut self, seconds: f32) -> u32 {
        self.lag += seconds;
        let due = (self.lag / FRAME) as u32;
        self.lag -= due as f32 * FRAME;

        if due > CATCH_UP {
            self.lag = 0.0;
        }

        due.min(CATCH_UP)
    }
}

#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct SpawnCloud {
    pub kind: CloudKind,
    pub pos: TilePos,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct CloudArt(pub Vec<(Puff, Handle<Image>)>);

impl CloudArt {
    pub fn get(&self, puff: Puff) -> Handle<Image> {
        self.0
            .iter()
            .find(|(drawn, _)| *drawn == puff)
            .map(|(_, image)| image.clone())
            .unwrap_or_default()
    }
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct CloudShadow;

pub struct CloudsPlugin;

impl Plugin for CloudsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldAge>()
            .init_resource::<CloudLaws>()
            .init_resource::<Sky>()
            .init_resource::<CloudArt>()
            .add_message::<SpawnCloud>()
            .add_message::<Painted>()
            .add_message::<Strike>()
            .add_systems(Startup, draw)
            .add_systems(Update, (renew, weather, cap, sponge, place).chain());
    }
}

fn draw(images: Option<ResMut<Assets<Image>>>, mut art: ResMut<CloudArt>) {
    let Some(mut images) = images else { return };

    art.0 = PUFFS
        .into_iter()
        .map(|puff| (puff, images.add(art::image(puff))))
        .collect();
}

fn renew(
    mut commands: Commands,
    bounds: Res<WorldBounds>,
    age: Res<WorldAge>,
    mut sky: ResMut<Sky>,
    clouds: Query<Entity, With<Cloud>>,
) {
    let size = Vec2::new(bounds.width, bounds.height);

    if sky.size == Some(size) {
        return;
    }

    if sky.size.is_some() {
        for entity in &clouds {
            commands.entity(entity).despawn();
        }
    }

    sky.size = Some(size);
    sky.timer = age.0.cloud_interval();
}

pub fn ortho(projection: &Projection) -> Option<f32> {
    match projection {
        Projection::Orthographic(view) if view.area.height() > 0.0 => {
            Some(view.area.height() / TILE_SIZE * 0.5)
        }
        _ => None,
    }
}

fn weather(
    mut commands: Commands,
    time: Res<Time>,
    clock: Option<Res<WorldClock>>,
    bounds: Res<WorldBounds>,
    age: Res<WorldAge>,
    laws: Res<CloudLaws>,
    art: Res<CloudArt>,
    camera: Option<Single<&Projection, With<WorldCamera>>>,
    mut sky: ResMut<Sky>,
    mut map: Option<ResMut<TileMap>>,
    mut asked: MessageReader<SpawnCloud>,
    mut clouds: Query<(Entity, &mut Cloud)>,
    mut painted: MessageWriter<Painted>,
    mut strikes: MessageWriter<Strike>,
) {
    let width = bounds.width / TILE_SIZE;
    let height = bounds.height / TILE_SIZE;
    let ortho = camera.and_then(|projection| ortho(&projection));
    let (multiplier, running) = clock.map_or((1.0, true), |clock| {
        (clock.speed().multiplier, clock.rate() > 0.0)
    });
    let dt = STEP * multiplier;
    let mut gone = HashSet::new();
    let mut cells = Vec::new();

    for request in asked.read() {
        let at = over_tile(request.pos, height as u32);
        let cloud = sky.make(request.kind, Some(at), height);
        spawn(&mut commands, &art, cloud);
    }

    for _ in 0..sky.steps(time.delta_secs()) {
        for (entity, mut cloud) in &mut clouds {
            if gone.contains(&entity) {
                continue;
            }

            let deeds = cloud.update(dt, running, width, ortho);

            if deeds.gone {
                gone.insert(entity);
                commands.entity(entity).despawn();
                continue;
            }

            let Some(map) = map.as_deref_mut() else {
                continue;
            };

            for _ in 0..deeds.drops {
                let spot = cloud.drop_spot(&mut sky.dice);
                if let Some(pos) = spot_tile(spot, map.width, map.height) {
                    let fall = rain::land(cloud.kind.drop(), map, pos, &mut sky.dice);
                    pour(map, fall, &mut cells, &mut strikes);
                }
            }

            for _ in 0..deeds.bolts {
                if !sky.dice.chance(rain::BOLT_ODDS) {
                    continue;
                }

                let spot = cloud.bolt_spot(&mut sky.dice);
                if let Some(pos) = spot_tile(spot, map.width, map.height) {
                    let fall = rain::bolt(map, pos, &mut sky.dice);
                    pour(map, fall, &mut cells, &mut strikes);
                }
            }
        }

        if running
            && sky.due(dt, age.0)
            && let Some(kind) = sky.choose(age.0, *laws)
        {
            let cloud = sky.make(kind, None, height);
            spawn(&mut commands, &art, cloud);
        }
    }

    if !cells.is_empty() {
        painted.write(Painted::new(cells));
    }
}

fn pour(
    map: &mut TileMap,
    fall: rain::Fall,
    cells: &mut Vec<TilePos>,
    strikes: &mut MessageWriter<Strike>,
) {
    for (pos, tile) in fall.changes {
        map.set(pos.x, pos.y, tile);
        cells.push(pos);
    }

    for strike in fall.strikes {
        strikes.write(strike);
    }
}

fn cap(mut commands: Commands, clouds: Query<(Entity, &Cloud)>) {
    let mut sky: Vec<(u64, Entity)> = clouds
        .iter()
        .map(|(entity, cloud)| (cloud.serial, entity))
        .collect();

    if sky.len() <= LIMIT {
        return;
    }

    sky.sort_unstable();
    for (_, entity) in &sky[..sky.len() - LIMIT] {
        commands.entity(*entity).despawn();
    }
}

fn spawn(commands: &mut Commands, art: &CloudArt, cloud: Cloud) {
    let image = art.get(cloud.puff);
    let (width, height) = cloud.puff.tiles();
    let size = Vec2::new(width, height) * TILE_SIZE;

    commands
        .spawn((
            Sprite {
                image: image.clone(),
                color: Color::NONE,
                flip_x: cloud.flip,
                custom_size: Some(size),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, CLOUD_LAYER),
            Visibility::Hidden,
            cloud,
        ))
        .with_child((
            CloudShadow,
            Sprite {
                image,
                color: Color::NONE,
                custom_size: Some(size),
                ..default()
            },
            Transform {
                translation: (SHADOW_OFFSET * TILE_SIZE).extend(SHADOW_LAYER - CLOUD_LAYER),
                scale: Vec3::new(1.0, SHADOW_SQUASH, 1.0),
                ..default()
            },
        ));
}

fn sponge(
    buttons: Option<Res<ButtonInput<MouseButton>>>,
    brush: Option<Res<Brush>>,
    forced: Option<Res<ForcedBrush>>,
    hovered: Option<Res<Hovered>>,
    over_ui: Option<Res<PointerOverUi>>,
    map: Option<Res<TileMap>>,
    mut armed: Local<bool>,
    mut clouds: Query<&mut Cloud>,
) {
    let (Some(buttons), Some(brush), Some(hovered), Some(map)) = (buttons, brush, hovered, map)
    else {
        return;
    };
    let brush = forced.map_or(*brush, |forced| forced.over(*brush));

    if brush.tool != Tool::Power(Power::Sponge) {
        return;
    }

    let over_ui = over_ui.is_some_and(|over| over.0);

    if buttons.just_pressed(MouseButton::Left) {
        *armed = !over_ui;
    }

    if !buttons.pressed(MouseButton::Left) || !*armed || over_ui {
        return;
    }

    let Some(pos) = hovered.pos else { return };
    let centre = Vec2::new(pos.x as f32, map.height.saturating_sub(1 + pos.y) as f32);

    soak(&mut clouds, centre, brush.radius);
}

pub fn sponge_reach(radius: u32) -> f32 {
    SPONGE_REACH * (radius + 1) as f32
}

fn soak(clouds: &mut Query<&mut Cloud>, centre: Vec2, radius: u32) {
    let reach = sponge_reach(radius);

    for mut cloud in clouds.iter_mut() {
        if cloud.at.distance_squared(centre) <= reach * reach && !cloud.dying {
            cloud.dying = true;
        }
    }
}

fn srgb_to_linear(channel: f32) -> f32 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

fn channel(value: u8) -> f32 {
    srgb_to_linear(value as f32 / 255.0)
}

pub fn gamma_alpha(face: [u8; 3], under: [u8; 3], alpha: f32) -> f32 {
    let alpha = alpha.clamp(0.0, 1.0);
    let mut lean = 0.0;
    let mut spread = 0.0;

    for (top, bottom) in face.into_iter().zip(under) {
        let below = bottom as f32 / 255.0;
        let mixed = below + (top as f32 / 255.0 - below) * alpha;
        let gap = channel(top) - channel(bottom);
        lean += gap * (srgb_to_linear(mixed) - channel(bottom));
        spread += gap * gap;
    }

    if spread < 1e-6 {
        alpha
    } else {
        (lean / spread).clamp(0.0, 1.0)
    }
}

fn tinted(color: [u8; 3], tone: [u8; 3]) -> [u8; 3] {
    [0, 1, 2].map(|i| ((color[i] as u32 * tone[i] as u32) / 255) as u8)
}

fn place(
    bounds: Res<WorldBounds>,
    mut clouds: Query<(&Cloud, &mut Transform, &mut Sprite, &mut Visibility), Without<CloudShadow>>,
    mut shadows: Query<(&ChildOf, &mut Sprite), With<CloudShadow>>,
) {
    let origin = -bounds.half();
    let sea = Ground::DeepOcean.color();

    for (cloud, mut transform, mut sprite, mut visibility) in &mut clouds {
        let lift = (cloud.serial % STACKED) as f32 * STACK;
        let at = origin + cloud.at * TILE_SIZE;
        transform.translation = at.extend(CLOUD_LAYER + lift);

        let [r, g, b] = cloud.kind.color();
        let face = tinted(cloud.kind.color(), art::TONES[1]);
        let alpha = gamma_alpha(face, sea, cloud.alpha);
        sprite.color = Color::srgba_u8(r, g, b, 255).with_alpha(alpha);
        sprite.flip_x = cloud.flip;
        *visibility = Visibility::Inherited;
    }

    for (parent, mut sprite) in &mut shadows {
        let Ok((cloud, ..)) = clouds.get(parent.parent()) else {
            continue;
        };

        let [r, g, b] = SHADOW_COLOR;
        let face = tinted(SHADOW_COLOR, art::TONES[1]);
        let alpha = gamma_alpha(face, sea, cloud.alpha * SHADOW_SHARE);
        sprite.color = Color::srgba_u8(r, g, b, 255).with_alpha(alpha);
        sprite.flip_x = cloud.flip;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use econbox_tile::Tile;
    use std::time::Duration;

    fn app(tiles: u32) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                FRAME,
            )))
            .insert_resource(WorldBounds::from_tiles(tiles, tiles))
            .insert_resource(TileMap::filled(tiles, tiles, Tile::bare(Ground::Sand)))
            .init_resource::<WorldClock>()
            .add_plugins(CloudsPlugin);
        app
    }

    fn seconds(app: &mut App, seconds: f32) {
        for _ in 0..(seconds / FRAME).round() as u32 {
            app.update();
        }
    }

    fn clouds(app: &mut App) -> Vec<Cloud> {
        let mut query = app.world_mut().query::<&Cloud>();
        query.iter(app.world()).cloned().collect()
    }

    #[test]
    fn a_new_world_waits_one_age_interval_for_its_first_cloud() {
        let mut app = app(256);
        seconds(&mut app, 20.0);
        assert!(clouds(&mut app).is_empty());

        seconds(&mut app, 2.0);
        let sky = clouds(&mut app);
        assert_eq!(sky.len(), 1);
        assert_eq!(sky[0].kind, CloudKind::Normal);
    }

    #[test]
    fn worldbox_time_runs_a_fifth_faster_than_the_wall_clock() {
        let mut sky = Sky::default();
        let mut stepped = 0;

        for _ in 0..600 {
            stepped += sky.steps(FRAME);
        }

        assert_eq!(stepped, 600);
        assert!((stepped as f32 * STEP - 12.0).abs() < 1e-3);
    }

    #[test]
    fn clouds_come_one_to_two_intervals_apart() {
        let mut sky = Sky::default();
        let mut last = 0.0;
        let mut clock = 0.0;
        let mut gaps = Vec::new();

        while gaps.len() < 200 {
            clock += STEP;
            if sky.due(STEP, Age::Hope) {
                gaps.push(clock - last);
                last = clock;
            }
        }

        assert!((gaps[0] - 25.0).abs() < 0.05, "{}", gaps[0]);
        for gap in &gaps[1..] {
            assert!(*gap >= 25.0 - 0.05 && *gap <= 50.0 + 0.05, "{gap}");
        }

        let mean = gaps[1..].iter().sum::<f32>() / (gaps.len() - 1) as f32;
        assert!((mean - 37.5).abs() < 2.5, "{mean}");
    }

    #[test]
    fn a_paused_world_spawns_and_moves_nothing() {
        let mut app = app(256);
        seconds(&mut app, 22.0);
        let before = clouds(&mut app);
        assert_eq!(before.len(), 1);

        app.world_mut().resource_mut::<WorldClock>().paused = true;
        seconds(&mut app, 60.0);
        let after = clouds(&mut app);

        assert_eq!(after.len(), 1);
        assert_eq!(after[0].at, before[0].at);
        assert_eq!(after[0].alpha, CloudKind::Normal.max_alpha());
    }

    #[test]
    fn an_open_window_holds_the_sky_like_a_pause() {
        let mut app = app(256);
        app.world_mut().resource_mut::<WorldClock>().hold = true;
        seconds(&mut app, 60.0);

        assert!(clouds(&mut app).is_empty());
    }

    #[test]
    fn faster_speeds_bring_clouds_sooner() {
        let mut app = app(256);
        app.world_mut().resource_mut::<WorldClock>().select("x5");
        seconds(&mut app, 5.0);

        assert_eq!(clouds(&mut app).len(), 1);
    }

    #[test]
    fn a_256_map_keeps_one_or_two_clouds_in_the_sky() {
        let mut app = app(256);
        app.world_mut().resource_mut::<WorldClock>().select("x20");
        let mut counts = Vec::new();

        for _ in 0..400 {
            seconds(&mut app, 0.5);
            counts.push(clouds(&mut app).len());
        }

        let mean = counts[100..].iter().sum::<usize>() as f32 / (counts.len() - 100) as f32;
        assert!((1.0..3.0).contains(&mean), "{mean}");
    }

    #[test]
    fn a_wider_map_holds_more_clouds_at_once() {
        let crowd = |tiles: u32| {
            let mut app = app(tiles);
            app.world_mut().resource_mut::<WorldClock>().select("x20");
            let mut total = 0;

            for _ in 0..400 {
                seconds(&mut app, 0.5);
                total += clouds(&mut app).len();
            }

            total
        };

        assert!(crowd(1024) > crowd(256) * 2);
    }

    #[test]
    fn the_cloud_law_keeps_plain_clouds_away() {
        let mut app = app(256);
        app.world_mut().resource_mut::<CloudLaws>().clouds = false;
        app.world_mut().resource_mut::<WorldClock>().select("x20");
        seconds(&mut app, 30.0);

        assert!(clouds(&mut app).is_empty());
    }

    #[test]
    fn the_disaster_law_keeps_rain_but_not_thunder() {
        let laws = CloudLaws {
            clouds: true,
            disasters: false,
        };

        assert!(laws.allow(CloudKind::Rain));
        assert!(laws.allow(CloudKind::Normal));
        assert!(!laws.allow(CloudKind::Lightning));
        assert!(!laws.allow(CloudKind::Acid));
    }

    #[test]
    fn a_called_cloud_appears_over_its_tile_even_while_paused() {
        let mut app = app(64);
        app.world_mut().resource_mut::<WorldClock>().paused = true;
        app.world_mut().write_message(SpawnCloud {
            kind: CloudKind::Rain,
            pos: TilePos::new(10, 20),
        });
        app.update();

        let sky = clouds(&mut app);
        assert_eq!(sky.len(), 1);
        assert_eq!(sky[0].at, Vec2::new(10.0, 43.0 + 20.0));
    }

    #[test]
    fn rain_clouds_soak_the_sand_under_their_shadow() {
        let mut app = app(64);
        app.world_mut().write_message(SpawnCloud {
            kind: CloudKind::Rain,
            pos: TilePos::new(5, 32),
        });
        seconds(&mut app, 6.0);

        let map = app.world().resource::<TileMap>();
        let wet: Vec<u32> = (0..map.height)
            .filter(|&y| (0..map.width).any(|x| map.at(x, y).ground == Ground::SoilLow))
            .collect();

        assert!(!wet.is_empty());
        assert!(wet.iter().all(|y| y.abs_diff(32) <= 11), "{wet:?}");
    }

    #[test]
    fn the_sky_holds_at_most_two_hundred_clouds() {
        let mut app = app(256);
        app.world_mut().resource_mut::<WorldClock>().paused = true;

        for i in 0..230 {
            app.world_mut().write_message(SpawnCloud {
                kind: CloudKind::Normal,
                pos: TilePos::new(i % 200, 100),
            });
        }
        app.update();
        app.update();

        let sky = clouds(&mut app);
        assert_eq!(sky.len(), LIMIT);
        assert_eq!(sky.iter().map(|cloud| cloud.serial).min(), Some(31));
    }

    #[test]
    fn a_new_map_clears_the_sky() {
        let mut app = app(256);
        app.world_mut().write_message(SpawnCloud {
            kind: CloudKind::Normal,
            pos: TilePos::new(1, 1),
        });
        app.update();
        assert_eq!(clouds(&mut app).len(), 1);

        app.insert_resource(WorldBounds::from_tiles(128, 128));
        app.update();
        app.update();

        assert!(clouds(&mut app).is_empty());
        assert!(app.world().resource::<Sky>().timer > 24.0);
    }

    #[test]
    fn the_sponge_reaches_ten_tiles_per_brush_step() {
        assert_eq!(sponge_reach(0), 10.0);
        assert_eq!(sponge_reach(4), 50.0);
    }

    fn linear_to_srgb(value: f32) -> f32 {
        if value <= 0.003_130_8 {
            value * 12.92
        } else {
            1.055 * value.powf(1.0 / 2.4) - 0.055
        }
    }

    fn worst_miss(face: [u8; 3], under: [u8; 3], alpha: f32) -> f32 {
        let fitted = gamma_alpha(face, Ground::DeepOcean.color(), alpha);

        (0..3)
            .map(|i| {
                let below = under[i] as f32 / 255.0;
                let wanted = below + (face[i] as f32 / 255.0 - below) * alpha;
                let bottom = channel(under[i]);
                let drawn = linear_to_srgb(bottom + (channel(face[i]) - bottom) * fitted);
                (drawn - wanted).abs() * 255.0
            })
            .fold(0.0, f32::max)
    }

    #[test]
    fn clouds_and_shadows_stay_close_to_worldbox_gamma_blending_over_sea_and_land() {
        let grounds = [
            Ground::DeepOcean.color(),
            Ground::CloseOcean.color(),
            Ground::ShallowWaters.color(),
            Ground::Sand.color(),
            Ground::Hills.color(),
            [70, 140, 50],
        ];
        let cloud = tinted(CloudKind::Normal.color(), art::TONES[1]);
        let shadow = tinted(SHADOW_COLOR, art::TONES[1]);

        for under in grounds {
            for alpha in [0.1, 0.25, 0.5] {
                assert!(worst_miss(cloud, under, alpha) < 10.0, "{under:?} {alpha}");
                assert!(
                    worst_miss(shadow, under, alpha * SHADOW_SHARE) < 7.0,
                    "{under:?}"
                );
            }
        }
    }

    #[test]
    fn a_gamma_blend_of_a_colour_with_itself_keeps_the_alpha() {
        assert_eq!(gamma_alpha([10, 20, 30], [10, 20, 30], 0.4), 0.4);
    }

    #[test]
    fn light_over_dark_needs_less_alpha_in_linear_space_and_dark_over_light_more() {
        let sea = Ground::DeepOcean.color();
        let cloud = gamma_alpha([250, 250, 250], sea, 0.5);
        let shadow = gamma_alpha(SHADOW_COLOR, sea, 0.25);

        assert!(cloud < 0.5 && cloud > 0.25, "{cloud}");
        assert!(shadow > 0.25 && shadow < 0.5, "{shadow}");
    }

    #[test]
    fn zoom_is_read_as_worldbox_half_height_in_tiles() {
        let mut view = OrthographicProjection::default_2d();
        view.area = Rect::new(0.0, 0.0, 300.0 * TILE_SIZE, 200.0 * TILE_SIZE);

        assert_eq!(ortho(&Projection::Orthographic(view)), Some(100.0));
    }
}
