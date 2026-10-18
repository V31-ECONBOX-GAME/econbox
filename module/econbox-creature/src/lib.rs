//! The living things: what can be put on the map, where it can stand, and how it moves.

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use econbox_art::{Mark, SPRITE, Skin, Stance, atlas, sprite};
use econbox_camera::TILE_SIZE;
use econbox_clock::WorldClock;
use econbox_tile::Tile;
use econbox_tilemap::{TileMap, TilePos};

pub const CREATURE_LAYER: f32 = 1.0;
pub const WANDER_REACH: u32 = 6;
pub const STARVING: f32 = 0.0;
pub const DROWN_RATE: f32 = 8.0;
pub const BURN_RATE: f32 = 30.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Kind {
    Folk,
    Farm,
    Beast,
    Aquatic,
    Monster,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Folk => "folk",
            Self::Farm => "farm",
            Self::Beast => "beast",
            Self::Aquatic => "aquatic",
            Self::Monster => "monster",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Species {
    Human,
    Elf,
    Orc,
    Dwarf,
    Cat,
    Dog,
    Chicken,
    Rabbit,
    Sheep,
    Cow,
    Fox,
    Monkey,
    Wolf,
    Bear,
    Boar,
    Crab,
    Frog,
    Fish,
    Zombie,
    Ogre,
    Demon,
}

pub const SPECIES: [Species; 21] = [
    Species::Human,
    Species::Elf,
    Species::Orc,
    Species::Dwarf,
    Species::Cat,
    Species::Dog,
    Species::Chicken,
    Species::Rabbit,
    Species::Sheep,
    Species::Cow,
    Species::Fox,
    Species::Monkey,
    Species::Wolf,
    Species::Bear,
    Species::Boar,
    Species::Crab,
    Species::Frog,
    Species::Fish,
    Species::Zombie,
    Species::Ogre,
    Species::Demon,
];

impl Species {
    pub fn name(self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Elf => "elf",
            Self::Orc => "orc",
            Self::Dwarf => "dwarf",
            Self::Cat => "cat",
            Self::Dog => "dog",
            Self::Chicken => "chicken",
            Self::Rabbit => "rabbit",
            Self::Sheep => "sheep",
            Self::Cow => "cow",
            Self::Fox => "fox",
            Self::Monkey => "monkey",
            Self::Wolf => "wolf",
            Self::Bear => "bear",
            Self::Boar => "boar",
            Self::Crab => "crab",
            Self::Frog => "frog",
            Self::Fish => "fish",
            Self::Zombie => "zombie",
            Self::Ogre => "ogre",
            Self::Demon => "demon",
        }
    }

    pub fn kind(self) -> Kind {
        match self {
            Self::Human | Self::Elf | Self::Orc | Self::Dwarf => Kind::Folk,
            Self::Cat
            | Self::Dog
            | Self::Chicken
            | Self::Rabbit
            | Self::Sheep
            | Self::Cow
            | Self::Fox
            | Self::Monkey => Kind::Farm,
            Self::Wolf | Self::Bear | Self::Boar => Kind::Beast,
            Self::Crab | Self::Frog | Self::Fish => Kind::Aquatic,
            Self::Zombie | Self::Ogre | Self::Demon => Kind::Monster,
        }
    }

    pub fn swims(self) -> bool {
        matches!(self, Self::Fish | Self::Crab | Self::Frog)
    }

    pub fn walks(self) -> bool {
        !matches!(self, Self::Fish)
    }

    pub fn stands_on(self, tile: Tile) -> bool {
        if tile.is_lava() {
            false
        } else if tile.is_water() {
            self.swims()
        } else {
            self.walks()
        }
    }

    pub fn health(self) -> f32 {
        match self.kind() {
            Kind::Folk => 26.0,
            Kind::Farm => 14.0,
            Kind::Beast => 40.0,
            Kind::Aquatic => 10.0,
            Kind::Monster => 60.0,
        }
    }

    pub fn speed(self) -> f32 {
        match self {
            Self::Rabbit | Self::Fox | Self::Cat => 26.0,
            Self::Wolf | Self::Dog | Self::Fish => 22.0,
            Self::Human | Self::Elf | Self::Orc | Self::Dwarf | Self::Monkey => 16.0,
            Self::Bear | Self::Boar | Self::Ogre | Self::Demon => 13.0,
            _ => 9.0,
        }
    }

    pub fn size(self) -> f32 {
        match self.kind() {
            Kind::Beast | Kind::Monster => 6.0,
            Kind::Folk => 5.0,
            _ => 4.0,
        }
    }

    pub fn stance(self) -> Stance {
        match self {
            Self::Human
            | Self::Elf
            | Self::Orc
            | Self::Dwarf
            | Self::Zombie
            | Self::Ogre
            | Self::Demon => Stance::Upright,
            Self::Chicken => Stance::Bird,
            Self::Crab | Self::Frog => Stance::Squat,
            Self::Fish => Stance::Swimmer,
            _ => Stance::Quadruped,
        }
    }

    pub fn mark(self) -> Mark {
        match self {
            Self::Orc | Self::Demon | Self::Ogre | Self::Cow | Self::Boar => Mark::Horns,
            Self::Rabbit | Self::Elf => Mark::LongEars,
            _ => Mark::Plain,
        }
    }

    pub fn skin(self) -> Skin {
        Skin {
            coat: self.color(),
            accent: self.accent(),
        }
    }

    pub fn accent(self) -> [u8; 3] {
        match self {
            Self::Human => [0x46, 0x6a, 0x9e],
            Self::Elf => [0x2f, 0x6b, 0x45],
            Self::Orc => [0x6b, 0x3a, 0x2c],
            Self::Dwarf => [0x8e, 0x4a, 0x2a],
            Self::Zombie => [0x46, 0x52, 0x38],
            Self::Ogre => [0x53, 0x42, 0x2c],
            Self::Demon => [0x2a, 0x12, 0x1e],
            Self::Chicken => [0xe2, 0x8a, 0x22],
            Self::Crab => [0x8e, 0x22, 0x1c],
            Self::Frog => [0xd8, 0xe0, 0x8a],
            Self::Fish => [0x2c, 0x62, 0x90],
            _ => shade_up(self.color()),
        }
    }

    pub fn slot(self) -> usize {
        SPECIES
            .iter()
            .position(|species| *species == self)
            .unwrap_or(0)
    }

    pub fn color(self) -> [u8; 3] {
        match self {
            Self::Human => [0xe6, 0xb2, 0x8a],
            Self::Elf => [0x9f, 0xe0, 0xa8],
            Self::Orc => [0x5f, 0x9e, 0x4a],
            Self::Dwarf => [0xc8, 0x7d, 0x3f],
            Self::Cat => [0xf0, 0xa6, 0x4b],
            Self::Dog => [0x9c, 0x6b, 0x42],
            Self::Chicken => [0xf2, 0xf0, 0xe4],
            Self::Rabbit => [0xe8, 0xe8, 0xf0],
            Self::Sheep => [0xd8, 0xd4, 0xc6],
            Self::Cow => [0xb0, 0xb4, 0xbc],
            Self::Fox => [0xe0, 0x72, 0x2c],
            Self::Monkey => [0x8a, 0x5a, 0x38],
            Self::Wolf => [0x77, 0x7f, 0x92],
            Self::Bear => [0x6b, 0x46, 0x2a],
            Self::Boar => [0x55, 0x45, 0x3c],
            Self::Crab => [0xd8, 0x3c, 0x32],
            Self::Frog => [0x5c, 0xc0, 0x4a],
            Self::Fish => [0x4a, 0x9c, 0xd8],
            Self::Zombie => [0x7a, 0x9c, 0x5a],
            Self::Ogre => [0x86, 0x6e, 0x48],
            Self::Demon => [0xb0, 0x2c, 0x50],
        }
    }
}

fn shade_up(color: [u8; 3]) -> [u8; 3] {
    color.map(|channel| (f32::from(channel) * 1.35).clamp(0.0, 255.0) as u8)
}

#[derive(Resource, Debug, Clone)]
pub struct CreatureArt {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Creature {
    pub species: Species,
    pub health: f32,
    pub age: f32,
    pub food: f32,
}

impl Creature {
    pub fn new(species: Species) -> Self {
        Self {
            species,
            health: species.health(),
            age: 0.0,
            food: 1.0,
        }
    }

    pub fn alive(&self) -> bool {
        self.health > 0.0
    }

    pub fn starving(&self) -> bool {
        self.food <= STARVING
    }

    pub fn suffer(&mut self, effect: Effect) {
        match effect {
            Effect::Kill => self.health = 0.0,
            Effect::Hurt(amount) => self.health -= amount,
            Effect::Heal => {
                self.health = self.species.health();
                self.food = 1.0;
            }
            Effect::Weaken => self.health *= 0.5,
            Effect::Purge if self.species.kind() == Kind::Monster => self.health = 0.0,
            Effect::Purge => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Effect {
    Kill,
    Hurt(f32),
    Heal,
    Weaken,
    Purge,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct Strike {
    pub centre: TilePos,
    pub reach: f32,
    pub effect: Effect,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Wander {
    pub target: Vec2,
    pub rest: f32,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct Spawn {
    pub species: Species,
    pub pos: TilePos,
}

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct Census {
    pub alive: usize,
    pub deaths: u64,
    pub per_species: [usize; SPECIES.len()],
}

impl Census {
    pub fn of(&self, species: Species) -> usize {
        self.per_species[species.slot()]
    }

    pub fn of_kind(&self, kind: Kind) -> usize {
        SPECIES
            .iter()
            .filter(|species| species.kind() == kind)
            .map(|species| self.of(*species))
            .sum()
    }
}

pub fn centre_of(map: &TileMap, pos: TilePos) -> Vec2 {
    let half = Vec2::new(map.width as f32, map.height as f32) * TILE_SIZE * 0.5;

    Vec2::new(
        -half.x + (pos.x as f32 + 0.5) * TILE_SIZE,
        half.y - (pos.y as f32 + 0.5) * TILE_SIZE,
    )
}

pub fn tile_of(map: &TileMap, world: Vec2) -> Option<TilePos> {
    let half = Vec2::new(map.width as f32, map.height as f32) * TILE_SIZE * 0.5;
    let local = world + half;

    if local.x < 0.0 || local.y < 0.0 {
        return None;
    }

    let x = (local.x / TILE_SIZE) as u32;
    let y = map
        .height
        .checked_sub(1 + (local.y / TILE_SIZE) as u32)
        .unwrap_or(u32::MAX);

    (x < map.width && y < map.height).then_some(TilePos::new(x, y))
}

fn drift(seed: u32) -> Vec2 {
    let mut h = seed.wrapping_mul(0x9e37_79b9);
    h ^= h >> 15;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;

    let angle = (h % 3600) as f32 * std::f32::consts::TAU / 3600.0;

    Vec2::new(angle.cos(), angle.sin())
}

pub fn sheet() -> Image {
    let frames: Vec<_> = SPECIES
        .iter()
        .map(|species| sprite(species.stance(), species.mark(), species.skin()))
        .collect();
    let (data, width, height) = atlas(&frames);

    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        mipmap_filter: ImageFilterMode::Nearest,
        ..default()
    });

    image
}

pub struct CreaturePlugin;

impl Plugin for CreaturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Census>()
            .add_message::<Spawn>()
            .add_message::<Strike>()
            .add_systems(Startup, draw)
            .add_systems(
                Update,
                (place, roam, live, smite, strand, reap, count)
                    .chain()
                    .run_if(resource_exists::<TileMap>),
            );
    }
}

fn draw(
    mut commands: Commands,
    images: Option<ResMut<Assets<Image>>>,
    layouts: Option<ResMut<Assets<TextureAtlasLayout>>>,
) {
    let (Some(mut images), Some(mut layouts)) = (images, layouts) else {
        return;
    };

    commands.insert_resource(CreatureArt {
        image: images.add(sheet()),
        layout: layouts.add(TextureAtlasLayout::from_grid(
            UVec2::splat(SPRITE as u32),
            SPECIES.len() as u32,
            1,
            None,
            None,
        )),
    });
}

fn place(
    mut commands: Commands,
    mut asked: MessageReader<Spawn>,
    map: Res<TileMap>,
    art: Option<Res<CreatureArt>>,
) {
    for request in asked.read() {
        let Some(tile) = map.get(request.pos) else {
            continue;
        };

        if !request.species.stands_on(tile) {
            continue;
        }

        let at = centre_of(&map, request.pos);
        let [r, g, b] = request.species.color();
        let size = request.species.size();

        let look = match &art {
            Some(art) => Sprite {
                image: art.image.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: art.layout.clone(),
                    index: request.species.slot(),
                }),
                custom_size: Some(Vec2::splat(size * 2.0)),
                ..default()
            },
            None => Sprite {
                color: Color::srgb_u8(r, g, b),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
        };

        commands.spawn((
            Creature::new(request.species),
            Wander {
                target: at,
                rest: 0.0,
            },
            look,
            Transform::from_xyz(at.x, at.y, CREATURE_LAYER),
        ));
    }
}

fn roam(
    time: Res<Time>,
    clock: Res<WorldClock>,
    map: Res<TileMap>,
    mut wanderers: Query<(&Creature, &mut Wander, &mut Transform)>,
) {
    let rate = clock.rate();

    if rate == 0.0 {
        return;
    }

    let step = time.delta_secs() * rate;

    for (creature, mut wander, mut transform) in &mut wanderers {
        let here = transform.translation.truncate();
        wander.rest -= step;

        if wander.rest <= 0.0 || here.distance(wander.target) < TILE_SIZE * 0.5 {
            let seed = (here.x.to_bits() ^ here.y.to_bits()).wrapping_add(clock.ticks as u32);
            let reach = WANDER_REACH as f32 * TILE_SIZE;
            let wish = here + drift(seed) * reach;

            if let Some(pos) = tile_of(&map, wish)
                && map
                    .get(pos)
                    .is_some_and(|tile| creature.species.stands_on(tile))
            {
                wander.target = centre_of(&map, pos);
            }

            wander.rest = 1.0 + (seed % 200) as f32 / 100.0;
        }

        let toward = wander.target - here;

        if toward.length() > f32::EPSILON {
            let move_by = toward.normalize() * creature.species.speed() * step;
            let next = if move_by.length() > toward.length() {
                wander.target
            } else {
                here + move_by
            };

            if tile_of(&map, next)
                .and_then(|pos| map.get(pos))
                .is_some_and(|tile| creature.species.stands_on(tile))
            {
                transform.translation.x = next.x;
                transform.translation.y = next.y;
            } else {
                wander.rest = 0.0;
            }
        }
    }
}

fn live(time: Res<Time>, clock: Res<WorldClock>, mut creatures: Query<&mut Creature>) {
    let rate = clock.rate();

    if rate == 0.0 {
        return;
    }

    let step = time.delta_secs() * rate;

    for mut creature in &mut creatures {
        creature.age += step;
        creature.food = (creature.food - step * 0.01).max(0.0);

        if creature.starving() {
            creature.health -= step;
        }
    }
}

fn smite(
    mut asked: MessageReader<Strike>,
    map: Res<TileMap>,
    mut creatures: Query<(&mut Creature, &Transform)>,
) {
    for strike in asked.read() {
        let centre = centre_of(&map, strike.centre);
        let reach = (strike.reach + 0.5) * TILE_SIZE;

        for (mut creature, transform) in &mut creatures {
            if transform.translation.truncate().distance(centre) <= reach {
                creature.suffer(strike.effect);
            }
        }
    }
}

fn strand(
    time: Res<Time>,
    clock: Res<WorldClock>,
    map: Res<TileMap>,
    mut creatures: Query<(&mut Creature, &Transform)>,
) {
    let rate = clock.rate();

    if rate == 0.0 {
        return;
    }

    let step = time.delta_secs() * rate;

    for (mut creature, transform) in &mut creatures {
        let Some(tile) =
            tile_of(&map, transform.translation.truncate()).and_then(|pos| map.get(pos))
        else {
            continue;
        };

        if tile.is_lava() {
            creature.health -= BURN_RATE * step;
        } else if !creature.species.stands_on(tile) {
            creature.health -= DROWN_RATE * step;
        }
    }
}

fn reap(mut commands: Commands, creatures: Query<(Entity, &Creature)>, mut census: ResMut<Census>) {
    for (entity, creature) in &creatures {
        if !creature.alive() {
            commands.entity(entity).despawn();
            census.deaths += 1;
        }
    }
}

fn count(creatures: Query<&Creature>, mut census: ResMut<Census>) {
    let mut per_species = [0; SPECIES.len()];

    for creature in &creatures {
        per_species[creature.species.slot()] += 1;
    }

    let alive = per_species.iter().sum();

    if census.alive != alive || census.per_species != per_species {
        census.alive = alive;
        census.per_species = per_species;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use econbox_tile::{Ground, Overlay};

    fn map() -> TileMap {
        TileMap::filled(64, 64, Tile::new(Ground::SoilLow, Overlay::Grass))
    }

    #[test]
    fn every_species_has_a_name_of_its_own() {
        let mut names: Vec<&str> = SPECIES.iter().map(|species| species.name()).collect();
        names.sort_unstable();
        let before = names.len();
        names.dedup();

        assert_eq!(names.len(), before);
        assert_eq!(before, SPECIES.len());
    }

    #[test]
    fn every_species_has_a_colour_of_its_own() {
        let mut colours: Vec<[u8; 3]> = SPECIES.iter().map(|species| species.color()).collect();
        colours.sort_unstable();
        let before = colours.len();
        colours.dedup();

        assert_eq!(colours.len(), before);
    }

    #[test]
    fn the_four_folk_are_the_ones_that_could_found_a_kingdom() {
        let folk: Vec<Species> = SPECIES
            .iter()
            .copied()
            .filter(|species| species.kind() == Kind::Folk)
            .collect();

        assert_eq!(
            folk,
            [Species::Human, Species::Elf, Species::Orc, Species::Dwarf]
        );
    }

    #[test]
    fn land_creatures_refuse_the_sea_and_fish_refuse_the_shore() {
        let grass = Tile::new(Ground::SoilLow, Overlay::Grass);
        let sea = Tile::bare(Ground::DeepOcean);

        assert!(Species::Human.stands_on(grass));
        assert!(!Species::Human.stands_on(sea));
        assert!(Species::Fish.stands_on(sea));
        assert!(!Species::Fish.stands_on(grass));
    }

    #[test]
    fn a_frog_is_at_home_on_both_sides_of_the_shore() {
        assert!(Species::Frog.stands_on(Tile::bare(Ground::ShallowWaters)));
        assert!(Species::Frog.stands_on(Tile::new(Ground::SoilLow, Overlay::Grass)));
    }

    #[test]
    fn nothing_will_set_foot_on_lava() {
        let lava = Tile::bare(Ground::Lava3);

        for species in SPECIES {
            assert!(!species.stands_on(lava), "{}", species.name());
        }
    }

    #[test]
    fn a_kill_ends_any_creature_however_tough() {
        let mut demon = Creature::new(Species::Demon);
        demon.suffer(Effect::Kill);

        assert!(!demon.alive());
    }

    #[test]
    fn a_hurt_takes_exactly_what_it_says() {
        let mut ogre = Creature::new(Species::Ogre);
        ogre.suffer(Effect::Hurt(10.0));

        assert_eq!(ogre.health, Species::Ogre.health() - 10.0);
    }

    #[test]
    fn a_blessing_makes_a_creature_whole_and_fed_again() {
        let mut human = Creature::new(Species::Human);
        human.health = 1.0;
        human.food = 0.0;
        human.suffer(Effect::Heal);

        assert_eq!(human.health, Species::Human.health());
        assert!(!human.starving());
    }

    #[test]
    fn a_curse_halves_health_but_never_kills_on_its_own() {
        let mut cow = Creature::new(Species::Cow);

        for _ in 0..40 {
            cow.suffer(Effect::Weaken);
        }

        assert!(cow.alive());
        assert!(cow.health < 1e-3);
    }

    #[test]
    fn a_purge_takes_monsters_and_spares_everyone_else() {
        for species in SPECIES {
            let mut creature = Creature::new(species);
            creature.suffer(Effect::Purge);

            assert_eq!(
                creature.alive(),
                species.kind() != Kind::Monster,
                "{}",
                species.name()
            );
        }
    }

    #[test]
    fn a_fresh_creature_starts_whole_and_fed() {
        let creature = Creature::new(Species::Human);

        assert_eq!(creature.health, Species::Human.health());
        assert_eq!(creature.age, 0.0);
        assert!(creature.alive());
        assert!(!creature.starving());
    }

    #[test]
    fn a_monster_outlasts_a_rabbit() {
        assert!(Species::Demon.health() > Species::Rabbit.health());
        assert!(Species::Rabbit.speed() > Species::Demon.speed());
    }

    #[test]
    fn the_centre_of_a_tile_maps_back_to_that_tile() {
        let map = map();

        for (x, y) in [(0, 0), (31, 17), (63, 63)] {
            let pos = TilePos::new(x, y);

            assert_eq!(tile_of(&map, centre_of(&map, pos)), Some(pos), "{x},{y}");
        }
    }

    #[test]
    fn a_point_off_the_map_belongs_to_no_tile() {
        let map = map();
        let half = 64.0 * TILE_SIZE * 0.5;

        assert!(tile_of(&map, Vec2::new(-half - 1.0, 0.0)).is_none());
        assert!(tile_of(&map, Vec2::new(0.0, half + 1.0)).is_none());
    }

    #[test]
    fn a_wander_step_always_points_somewhere() {
        for seed in 0..64_u32 {
            let step = drift(seed);

            assert!((step.length() - 1.0).abs() < 1e-4, "{seed}");
        }
    }

    #[test]
    fn wandering_does_not_always_head_the_same_way() {
        let mut seen: Vec<(i32, i32)> = (0..32_u32)
            .map(|seed| {
                let step = drift(seed) * 100.0;
                (step.x as i32, step.y as i32)
            })
            .collect();
        seen.sort_unstable();
        seen.dedup();

        assert!(seen.len() > 8, "only {} directions", seen.len());
    }

    #[test]
    fn a_starving_creature_loses_health_until_it_dies() {
        let mut creature = Creature::new(Species::Rabbit);
        creature.food = 0.0;

        assert!(creature.starving());

        creature.health = 0.0;
        assert!(!creature.alive());
    }
}
